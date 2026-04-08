use std::{
    ffi::CString,
    sync::{Arc, Mutex},
};

use ash::{Device, Instance, khr, vk};
use winit::{
    raw_window_handle::HasDisplayHandle, raw_window_handle::HasWindowHandle, window::Window,
};

use crate::{
    constants::MAX_FRAMES_IN_FLIGHT,
    error::{EngineError, Result},
    shared::load_file_as_vec_u32,
    utils::compiled_spirv_path_for_source,
};

#[derive(Clone, Copy)]
struct QueueFamilies {
    graphics: u32,
    present: u32,
}

struct FrameSync {
    image_available: vk::Semaphore,
    render_finished: vk::Semaphore,
    in_flight: vk::Fence,
}

struct RuntimeState {
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
    swapchain_format: vk::Format,
    extent: vk::Extent2D,
    depth_image: vk::Image,
    depth_memory: vk::DeviceMemory,
    depth_view: vk::ImageView,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    frames: Vec<FrameSync>,
    current_frame: usize,
    zero_sized: bool,
}

pub(crate) struct VContext {
    pub instance: Instance,
    pub device: Device,
    pub physical_device: vk::PhysicalDevice,
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub graphics_queue_family: u32,
    pub present_queue_family: u32,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    upload_command_pool: vk::CommandPool,
    surface_loader: khr::surface::Instance,
    surface: vk::SurfaceKHR,
    swapchain_loader: khr::swapchain::Device,
    depth_format: vk::Format,
    state: Mutex<RuntimeState>,
}

impl VContext {
    pub fn new(window: &Window) -> Result<Arc<Self>> {
        let entry = ash::Entry::linked();

        let app_name = CString::new("benzene").expect("static app name should be valid");
        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .engine_name(&app_name)
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let extension_names =
            ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw())
                .map_err(|result| {
                    EngineError::vk("enumerating required instance extensions", result)
                })?;
        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(extension_names);

        let instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .map_err(|result| EngineError::vk("creating instance", result))?
        };

        let surface = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None,
            )
            .map_err(|result| EngineError::vk("creating window surface", result))?
        };
        let surface_loader = khr::surface::Instance::new(&entry, &instance);

        let (physical_device, queue_families, memory_properties) =
            select_physical_device(&instance, &surface_loader, surface)?;

        let unique_queue_families = if queue_families.graphics == queue_families.present {
            vec![queue_families.graphics]
        } else {
            vec![queue_families.graphics, queue_families.present]
        };

        let queue_priorities = [1.0f32];
        let queue_infos: Vec<_> = unique_queue_families
            .iter()
            .map(|queue_family| {
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(*queue_family)
                    .queue_priorities(&queue_priorities)
            })
            .collect();
        let device_extensions = [khr::swapchain::NAME.as_ptr()];
        let features = vk::PhysicalDeviceFeatures::default().sampler_anisotropy(true);
        let device_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&device_extensions)
            .enabled_features(&features);

        let device = unsafe {
            instance
                .create_device(physical_device, &device_info, None)
                .map_err(|result| EngineError::vk("creating logical device", result))?
        };

        let graphics_queue = unsafe { device.get_device_queue(queue_families.graphics, 0) };
        let present_queue = unsafe { device.get_device_queue(queue_families.present, 0) };

        let upload_command_pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_families.graphics)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let upload_command_pool = unsafe {
            device
                .create_command_pool(&upload_command_pool_info, None)
                .map_err(|result| EngineError::vk("creating upload command pool", result))?
        };

        let swapchain_loader = khr::swapchain::Device::new(&instance, &device);
        let command_pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_families.graphics)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let command_pool = unsafe {
            device
                .create_command_pool(&command_pool_info, None)
                .map_err(|result| EngineError::vk("creating frame command pool", result))?
        };
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32);
        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&allocate_info)
                .map_err(|result| EngineError::vk("allocating frame command buffers", result))?
        };

        let mut frames = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let semaphore_info = vk::SemaphoreCreateInfo::default();
            let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
            let image_available = unsafe {
                device
                    .create_semaphore(&semaphore_info, None)
                    .map_err(|result| EngineError::vk("creating acquire semaphore", result))?
            };
            let render_finished = unsafe {
                device
                    .create_semaphore(&semaphore_info, None)
                    .map_err(|result| EngineError::vk("creating present semaphore", result))?
            };
            let in_flight = unsafe {
                device
                    .create_fence(&fence_info, None)
                    .map_err(|result| EngineError::vk("creating in-flight fence", result))?
            };
            frames.push(FrameSync {
                image_available,
                render_finished,
                in_flight,
            });
        }

        let depth_format = select_depth_format(&instance, physical_device)?;
        let context = Arc::new(Self {
            instance,
            device,
            physical_device,
            memory_properties,
            graphics_queue_family: queue_families.graphics,
            present_queue_family: queue_families.present,
            graphics_queue,
            present_queue,
            upload_command_pool,
            surface_loader,
            surface,
            swapchain_loader,
            depth_format,
            state: Mutex::new(RuntimeState {
                swapchain: vk::SwapchainKHR::null(),
                swapchain_images: Vec::new(),
                swapchain_image_views: Vec::new(),
                swapchain_format: vk::Format::UNDEFINED,
                extent: vk::Extent2D::default(),
                depth_image: vk::Image::null(),
                depth_memory: vk::DeviceMemory::null(),
                depth_view: vk::ImageView::null(),
                command_pool,
                command_buffers,
                frames,
                current_frame: 0,
                zero_sized: false,
            }),
        });
        context.recreate_swapchain(window)?;
        Ok(context)
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn depth_format(&self) -> vk::Format {
        self.depth_format
    }

    pub fn swapchain_format(&self) -> vk::Format {
        self.state
            .lock()
            .expect("vulkan context mutex poisoned")
            .swapchain_format
    }

    pub fn extent(&self) -> vk::Extent2D {
        self.state
            .lock()
            .expect("vulkan context mutex poisoned")
            .extent
    }

    pub fn is_zero_sized(&self) -> bool {
        self.state
            .lock()
            .expect("vulkan context mutex poisoned")
            .zero_sized
    }

    pub fn framebuffer_views(&self) -> (Vec<vk::ImageView>, vk::ImageView, vk::Extent2D) {
        let state = self.state.lock().expect("vulkan context mutex poisoned");
        (
            state.swapchain_image_views.clone(),
            state.depth_view,
            state.extent,
        )
    }

    pub fn wait_idle(&self) {
        unsafe {
            let _ = self.device.device_wait_idle();
        }
    }

    pub fn create_buffer(
        &self,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<(vk::Buffer, vk::DeviceMemory)> {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            self.device
                .create_buffer(&buffer_info, None)
                .map_err(|result| EngineError::vk("creating buffer", result))?
        };

        let requirements = unsafe { self.device.get_buffer_memory_requirements(buffer) };
        let memory_type_index = find_memory_type_index(
            &self.memory_properties,
            requirements.memory_type_bits,
            properties,
        )
        .ok_or_else(|| {
            EngineError::Message("failed to find compatible buffer memory type".into())
        })?;
        let allocation_info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type_index);

        let memory = unsafe {
            self.device
                .allocate_memory(&allocation_info, None)
                .map_err(|result| EngineError::vk("allocating buffer memory", result))?
        };

        unsafe {
            self.device
                .bind_buffer_memory(buffer, memory, 0)
                .map_err(|result| EngineError::vk("binding buffer memory", result))?;
        }

        Ok((buffer, memory))
    }

    pub fn destroy_buffer(&self, buffer: vk::Buffer, memory: vk::DeviceMemory) {
        unsafe {
            self.device.destroy_buffer(buffer, None);
            self.device.free_memory(memory, None);
        }
    }

    pub fn write_buffer_bytes(&self, memory: vk::DeviceMemory, bytes: &[u8]) -> Result<()> {
        let mapped = unsafe {
            self.device
                .map_memory(memory, 0, bytes.len() as u64, vk::MemoryMapFlags::empty())
                .map_err(|result| EngineError::vk("mapping buffer memory", result))?
        };
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), mapped as *mut u8, bytes.len());
            self.device.unmap_memory(memory);
        }
        Ok(())
    }

    pub fn upload_device_buffer(&self, buffer: vk::Buffer, bytes: &[u8]) -> Result<()> {
        let (staging_buffer, staging_memory) = self.create_buffer(
            bytes.len() as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        self.write_buffer_bytes(staging_memory, bytes)?;

        self.with_one_time_commands(|device, command_buffer| unsafe {
            let copy = vk::BufferCopy::default().size(bytes.len() as u64);
            device.cmd_copy_buffer(command_buffer, staging_buffer, buffer, &[copy]);
        })?;

        self.destroy_buffer(staging_buffer, staging_memory);
        Ok(())
    }

    pub fn create_image_2d(
        &self,
        extent: vk::Extent3D,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<(vk::Image, vk::DeviceMemory)> {
        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(extent)
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1);

        let image = unsafe {
            self.device
                .create_image(&image_info, None)
                .map_err(|result| EngineError::vk("creating image", result))?
        };

        let requirements = unsafe { self.device.get_image_memory_requirements(image) };
        let memory_type_index = find_memory_type_index(
            &self.memory_properties,
            requirements.memory_type_bits,
            properties,
        )
        .ok_or_else(|| {
            EngineError::Message("failed to find compatible image memory type".into())
        })?;

        let allocation_info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type_index);
        let memory = unsafe {
            self.device
                .allocate_memory(&allocation_info, None)
                .map_err(|result| EngineError::vk("allocating image memory", result))?
        };

        unsafe {
            self.device
                .bind_image_memory(image, memory, 0)
                .map_err(|result| EngineError::vk("binding image memory", result))?;
        }

        Ok((image, memory))
    }

    pub fn destroy_image(&self, image: vk::Image, memory: vk::DeviceMemory) {
        unsafe {
            self.device.destroy_image(image, None);
            self.device.free_memory(memory, None);
        }
    }

    pub fn create_image_view(
        &self,
        image: vk::Image,
        format: vk::Format,
        aspect: vk::ImageAspectFlags,
    ) -> Result<vk::ImageView> {
        let subresource_range = vk::ImageSubresourceRange::default()
            .aspect_mask(aspect)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let view_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .format(format)
            .view_type(vk::ImageViewType::TYPE_2D)
            .subresource_range(subresource_range);

        unsafe {
            self.device
                .create_image_view(&view_info, None)
                .map_err(|result| EngineError::vk("creating image view", result))
        }
    }

    pub fn create_sampler(&self) -> Result<vk::Sampler> {
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(true)
            .max_anisotropy(16.0)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR);

        unsafe {
            self.device
                .create_sampler(&sampler_info, None)
                .map_err(|result| EngineError::vk("creating sampler", result))
        }
    }

    pub fn upload_rgba_texture(
        &self,
        image: vk::Image,
        extent: vk::Extent3D,
        bytes: &[u8],
    ) -> Result<()> {
        let (staging_buffer, staging_memory) = self.create_buffer(
            bytes.len() as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        self.write_buffer_bytes(staging_memory, bytes)?;

        self.with_one_time_commands(|device, command_buffer| unsafe {
            transition_image_layout(
                device,
                command_buffer,
                image,
                vk::ImageAspectFlags::COLOR,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::AccessFlags::empty(),
                vk::AccessFlags::TRANSFER_WRITE,
            );

            let subresource = vk::ImageSubresourceLayers::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(0)
                .base_array_layer(0)
                .layer_count(1);
            let region = vk::BufferImageCopy::default()
                .image_subresource(subresource)
                .image_extent(extent);

            device.cmd_copy_buffer_to_image(
                command_buffer,
                staging_buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );

            transition_image_layout(
                device,
                command_buffer,
                image,
                vk::ImageAspectFlags::COLOR,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::AccessFlags::TRANSFER_WRITE,
                vk::AccessFlags::SHADER_READ,
            );
        })?;

        self.destroy_buffer(staging_buffer, staging_memory);
        Ok(())
    }

    pub fn with_one_time_commands(
        &self,
        record: impl FnOnce(&Device, vk::CommandBuffer),
    ) -> Result<()> {
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.upload_command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer = unsafe {
            self.device
                .allocate_command_buffers(&allocate_info)
                .map_err(|result| EngineError::vk("allocating upload command buffer", result))?[0]
        };

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            self.device
                .begin_command_buffer(command_buffer, &begin_info)
                .map_err(|result| EngineError::vk("beginning upload command buffer", result))?;
        }

        record(&self.device, command_buffer);

        unsafe {
            self.device
                .end_command_buffer(command_buffer)
                .map_err(|result| EngineError::vk("ending upload command buffer", result))?;

            let submit_info =
                vk::SubmitInfo::default().command_buffers(std::slice::from_ref(&command_buffer));
            self.device
                .queue_submit(
                    self.graphics_queue,
                    std::slice::from_ref(&submit_info),
                    vk::Fence::null(),
                )
                .map_err(|result| EngineError::vk("submitting upload command buffer", result))?;
            self.device
                .queue_wait_idle(self.graphics_queue)
                .map_err(|result| EngineError::vk("waiting for upload queue idle", result))?;
            self.device.free_command_buffers(
                self.upload_command_pool,
                std::slice::from_ref(&command_buffer),
            );
        }

        Ok(())
    }

    pub fn load_shader_module(&self, source_path: &str) -> Result<vk::ShaderModule> {
        let compiled_path = compiled_spirv_path_for_source(source_path);
        let code = load_file_as_vec_u32(&compiled_path);
        let create_info = vk::ShaderModuleCreateInfo::default().code(&code);
        unsafe {
            self.device
                .create_shader_module(&create_info, None)
                .map_err(|result| EngineError::vk("creating shader module", result))
        }
    }

    pub fn recreate_swapchain(&self, window: &Window) -> Result<()> {
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            self.state
                .lock()
                .expect("vulkan context mutex poisoned")
                .zero_sized = true;
            return Ok(());
        }

        self.wait_idle();

        let capabilities = unsafe {
            self.surface_loader
                .get_physical_device_surface_capabilities(self.physical_device, self.surface)
                .map_err(|result| EngineError::vk("querying surface capabilities", result))?
        };
        let formats = unsafe {
            self.surface_loader
                .get_physical_device_surface_formats(self.physical_device, self.surface)
                .map_err(|result| EngineError::vk("querying surface formats", result))?
        };
        let present_modes = unsafe {
            self.surface_loader
                .get_physical_device_surface_present_modes(self.physical_device, self.surface)
                .map_err(|result| EngineError::vk("querying present modes", result))?
        };

        let surface_format = choose_surface_format(&formats);
        let present_mode = choose_present_mode(&present_modes);
        let extent = choose_extent(size.width, size.height, &capabilities);
        let mut image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count > 0 && image_count > capabilities.max_image_count {
            image_count = capabilities.max_image_count;
        }

        let queue_family_indices = [self.graphics_queue_family, self.present_queue_family];
        let mut create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(self.surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);
        if self.graphics_queue_family != self.present_queue_family {
            create_info = create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&queue_family_indices);
        } else {
            create_info = create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
        }

        let swapchain = unsafe {
            self.swapchain_loader
                .create_swapchain(&create_info, None)
                .map_err(|result| EngineError::vk("creating swapchain", result))?
        };
        let swapchain_images = unsafe {
            self.swapchain_loader
                .get_swapchain_images(swapchain)
                .map_err(|result| EngineError::vk("retrieving swapchain images", result))?
        };
        let swapchain_image_views = swapchain_images
            .iter()
            .map(|image| {
                self.create_image_view(*image, surface_format.format, vk::ImageAspectFlags::COLOR)
            })
            .collect::<Result<Vec<_>>>()?;

        let depth_extent = vk::Extent3D {
            width: extent.width,
            height: extent.height,
            depth: 1,
        };
        let (depth_image, depth_memory) = self.create_image_2d(
            depth_extent,
            self.depth_format,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        let depth_view =
            self.create_image_view(depth_image, self.depth_format, vk::ImageAspectFlags::DEPTH)?;

        let mut state = self.state.lock().expect("vulkan context mutex poisoned");
        state.zero_sized = false;
        self.destroy_swapchain_resources(&mut state);
        state.swapchain = swapchain;
        state.swapchain_images = swapchain_images;
        state.swapchain_image_views = swapchain_image_views;
        state.swapchain_format = surface_format.format;
        state.extent = extent;
        state.depth_image = depth_image;
        state.depth_memory = depth_memory;
        state.depth_view = depth_view;
        Ok(())
    }

    pub fn begin_frame(&self) -> Result<(usize, usize, vk::CommandBuffer)> {
        let state = self.state.lock().expect("vulkan context mutex poisoned");
        if state.zero_sized {
            return Err(EngineError::ZeroSizedWindow);
        }

        let frame_index = state.current_frame;
        let frame = &state.frames[frame_index];

        unsafe {
            self.device
                .wait_for_fences(std::slice::from_ref(&frame.in_flight), true, u64::MAX)
                .map_err(|result| EngineError::vk("waiting for in-flight fence", result))?;
        }

        let acquired = unsafe {
            self.swapchain_loader.acquire_next_image(
                state.swapchain,
                u64::MAX,
                frame.image_available,
                vk::Fence::null(),
            )
        };
        let (image_index, is_suboptimal) = match acquired {
            Ok(result) => result,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => return Err(EngineError::SwapchainOutOfDate),
            Err(result) => return Err(EngineError::vk("acquiring swapchain image", result)),
        };
        if is_suboptimal {
            return Err(EngineError::SwapchainOutOfDate);
        }

        let command_buffer = state.command_buffers[frame_index];
        unsafe {
            self.device
                .reset_fences(std::slice::from_ref(&frame.in_flight))
                .map_err(|result| EngineError::vk("resetting in-flight fence", result))?;
            self.device
                .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                .map_err(|result| EngineError::vk("resetting frame command buffer", result))?;
            let begin_info = vk::CommandBufferBeginInfo::default();
            self.device
                .begin_command_buffer(command_buffer, &begin_info)
                .map_err(|result| EngineError::vk("beginning frame command buffer", result))?;
        }

        Ok((frame_index, image_index as usize, command_buffer))
    }

    pub fn end_frame(&self, image_index: usize, command_buffer: vk::CommandBuffer) -> Result<()> {
        let mut state = self.state.lock().expect("vulkan context mutex poisoned");
        let frame_index = state.current_frame;
        let frame = &state.frames[frame_index];

        unsafe {
            self.device
                .end_command_buffer(command_buffer)
                .map_err(|result| EngineError::vk("ending frame command buffer", result))?;

            let wait_stage = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(std::slice::from_ref(&frame.image_available))
                .wait_dst_stage_mask(&wait_stage)
                .command_buffers(std::slice::from_ref(&command_buffer))
                .signal_semaphores(std::slice::from_ref(&frame.render_finished));

            self.device
                .queue_submit(
                    self.graphics_queue,
                    std::slice::from_ref(&submit_info),
                    frame.in_flight,
                )
                .map_err(|result| EngineError::vk("submitting graphics work", result))?;

            let image_indices = [image_index as u32];
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(std::slice::from_ref(&frame.render_finished))
                .swapchains(std::slice::from_ref(&state.swapchain))
                .image_indices(&image_indices);
            match self
                .swapchain_loader
                .queue_present(self.present_queue, &present_info)
            {
                Ok(is_suboptimal) => {
                    if is_suboptimal {
                        return Err(EngineError::SwapchainOutOfDate);
                    }
                }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    return Err(EngineError::SwapchainOutOfDate);
                }
                Err(result) => return Err(EngineError::vk("presenting swapchain image", result)),
            }
        }

        state.current_frame = (state.current_frame + 1) % state.frames.len();
        Ok(())
    }

    fn destroy_swapchain_resources(&self, state: &mut RuntimeState) {
        unsafe {
            if state.depth_view != vk::ImageView::null() {
                self.device.destroy_image_view(state.depth_view, None);
                state.depth_view = vk::ImageView::null();
            }
            if state.depth_image != vk::Image::null()
                && state.depth_memory != vk::DeviceMemory::null()
            {
                self.destroy_image(state.depth_image, state.depth_memory);
                state.depth_image = vk::Image::null();
                state.depth_memory = vk::DeviceMemory::null();
            }
            for image_view in state.swapchain_image_views.drain(..) {
                self.device.destroy_image_view(image_view, None);
            }
            if state.swapchain != vk::SwapchainKHR::null() {
                self.swapchain_loader
                    .destroy_swapchain(state.swapchain, None);
                state.swapchain = vk::SwapchainKHR::null();
            }
        }
        state.swapchain_images.clear();
    }
}

impl Drop for VContext {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();

            let mut state = self.state.lock().expect("vulkan context mutex poisoned");
            self.destroy_swapchain_resources(&mut state);
            for frame in state.frames.drain(..) {
                self.device.destroy_semaphore(frame.image_available, None);
                self.device.destroy_semaphore(frame.render_finished, None);
                self.device.destroy_fence(frame.in_flight, None);
            }
            self.device
                .free_command_buffers(state.command_pool, &state.command_buffers);
            self.device.destroy_command_pool(state.command_pool, None);
            drop(state);

            self.surface_loader.destroy_surface(self.surface, None);
            self.device
                .destroy_command_pool(self.upload_command_pool, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

fn transition_image_layout(
    device: &Device,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    aspect_mask: vk::ImageAspectFlags,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    src_stage: vk::PipelineStageFlags,
    dst_stage: vk::PipelineStageFlags,
    src_access: vk::AccessFlags,
    dst_access: vk::AccessFlags,
) {
    let subresource_range = vk::ImageSubresourceRange::default()
        .aspect_mask(aspect_mask)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1);
    let barrier = vk::ImageMemoryBarrier::default()
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_access_mask(src_access)
        .dst_access_mask(dst_access)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(subresource_range);

    unsafe {
        device.cmd_pipeline_barrier(
            command_buffer,
            src_stage,
            dst_stage,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            std::slice::from_ref(&barrier),
        );
    }
}

fn select_physical_device(
    instance: &Instance,
    surface_loader: &khr::surface::Instance,
    surface: vk::SurfaceKHR,
) -> Result<(
    vk::PhysicalDevice,
    QueueFamilies,
    vk::PhysicalDeviceMemoryProperties,
)> {
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .map_err(|result| EngineError::vk("enumerating physical devices", result))?
    };

    let mut selected = None;
    let mut best_score = 0usize;

    for physical_device in physical_devices {
        let queue_families =
            find_queue_families(instance, surface_loader, surface, physical_device)?;
        let Some(queue_families) = queue_families else {
            continue;
        };

        let extension_properties = unsafe {
            instance
                .enumerate_device_extension_properties(physical_device)
                .map_err(|result| EngineError::vk("enumerating device extensions", result))?
        };
        let swapchain_supported = extension_properties
            .iter()
            .any(|extension| extension.extension_name_as_c_str() == Ok(khr::swapchain::NAME));
        if !swapchain_supported {
            continue;
        }

        let surface_formats = unsafe {
            surface_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .map_err(|result| EngineError::vk("querying device surface formats", result))?
        };
        let present_modes = unsafe {
            surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface)
                .map_err(|result| EngineError::vk("querying device present modes", result))?
        };
        if surface_formats.is_empty() || present_modes.is_empty() {
            continue;
        }

        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let mut score = surface_formats.len() + present_modes.len();
        if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
            score += 1000;
        }

        if score > best_score {
            let memory_properties =
                unsafe { instance.get_physical_device_memory_properties(physical_device) };
            best_score = score;
            selected = Some((physical_device, queue_families, memory_properties));
        }
    }

    selected.ok_or(EngineError::NoCompatibleDevice)
}

fn find_queue_families(
    instance: &Instance,
    surface_loader: &khr::surface::Instance,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
) -> Result<Option<QueueFamilies>> {
    let queue_family_properties =
        unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

    let mut graphics = None;
    let mut present = None;

    for (index, family) in queue_family_properties.iter().enumerate() {
        if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            graphics = Some(index as u32);
        }

        let supports_present = unsafe {
            surface_loader
                .get_physical_device_surface_support(physical_device, index as u32, surface)
                .map_err(|result| EngineError::vk("querying surface queue support", result))?
        };
        if supports_present {
            present = Some(index as u32);
        }
    }

    Ok(match (graphics, present) {
        (Some(graphics), Some(present)) => Some(QueueFamilies { graphics, present }),
        _ => None,
    })
}

fn choose_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
    formats
        .iter()
        .copied()
        .find(|format| {
            format.format == vk::Format::B8G8R8A8_SRGB
                && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or(formats[0])
}

fn choose_present_mode(present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
    if present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
        vk::PresentModeKHR::MAILBOX
    } else {
        vk::PresentModeKHR::FIFO
    }
}

fn choose_extent(
    width: u32,
    height: u32,
    capabilities: &vk::SurfaceCapabilitiesKHR,
) -> vk::Extent2D {
    vk::Extent2D {
        width: width.clamp(
            capabilities.min_image_extent.width,
            capabilities.max_image_extent.width,
        ),
        height: height.clamp(
            capabilities.min_image_extent.height,
            capabilities.max_image_extent.height,
        ),
    }
}

fn select_depth_format(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Result<vk::Format> {
    let candidates = [
        vk::Format::D32_SFLOAT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D24_UNORM_S8_UINT,
    ];

    for format in candidates {
        let properties =
            unsafe { instance.get_physical_device_format_properties(physical_device, format) };
        if properties
            .optimal_tiling_features
            .contains(vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT)
        {
            return Ok(format);
        }
    }

    Err(EngineError::Message(
        "failed to find supported depth format".into(),
    ))
}

fn find_memory_type_index(
    memory_properties: &vk::PhysicalDeviceMemoryProperties,
    type_filter: u32,
    required_properties: vk::MemoryPropertyFlags,
) -> Option<u32> {
    (0..memory_properties.memory_type_count).find(|index| {
        let is_supported = type_filter & (1 << index) != 0;
        let has_properties = memory_properties.memory_types[*index as usize]
            .property_flags
            .contains(required_properties);
        is_supported && has_properties
    })
}
