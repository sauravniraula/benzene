use std::sync::Arc;

use ash::vk;
use image::GenericImageView;
use slotmap::{Key, SlotMap, new_key_type};

use crate::{
    error::{EngineError, Result},
    render::vulkan::VContext,
};

new_key_type! {
    pub struct MeshId;
    pub struct TextureId;
    pub struct MaterialId;
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

pub(crate) struct Mesh {
    context: Arc<VContext>,
    pub vertex_buffer: vk::Buffer,
    vertex_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    index_memory: vk::DeviceMemory,
    pub index_count: u32,
}

impl Mesh {
    fn from_vertices(
        context: Arc<VContext>,
        vertices: &[MeshVertex],
        indices: &[u32],
    ) -> Result<Self> {
        let vertex_bytes = unsafe {
            std::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                std::mem::size_of_val(vertices),
            )
        };
        let index_bytes = unsafe {
            std::slice::from_raw_parts(
                indices.as_ptr() as *const u8,
                std::mem::size_of_val(indices),
            )
        };

        let (vertex_buffer, vertex_memory) = context.create_buffer(
            vertex_bytes.len() as u64,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        context.upload_device_buffer(vertex_buffer, vertex_bytes)?;

        let (index_buffer, index_memory) = context.create_buffer(
            index_bytes.len() as u64,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        context.upload_device_buffer(index_buffer, index_bytes)?;

        Ok(Self {
            context,
            vertex_buffer,
            vertex_memory,
            index_buffer,
            index_memory,
            index_count: indices.len() as u32,
        })
    }

    fn from_obj(context: Arc<VContext>, path: &str) -> Result<Self> {
        let (models, _) =
            tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS).map_err(|source| EngineError::Obj {
                path: path.into(),
                source,
            })?;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut vertex_offset = 0u32;

        for model in models {
            let mesh = model.mesh;
            let total_vertices = mesh.positions.len() / 3;
            for index in 0..total_vertices {
                let normal = if mesh.normals.len() >= (index + 1) * 3 {
                    [
                        mesh.normals[index * 3],
                        mesh.normals[index * 3 + 1],
                        mesh.normals[index * 3 + 2],
                    ]
                } else {
                    [0.0, 1.0, 0.0]
                };
                let uv = if mesh.texcoords.len() >= (index + 1) * 2 {
                    [mesh.texcoords[index * 2], mesh.texcoords[index * 2 + 1]]
                } else {
                    [0.0, 0.0]
                };
                vertices.push(MeshVertex {
                    position: [
                        mesh.positions[index * 3],
                        mesh.positions[index * 3 + 1],
                        mesh.positions[index * 3 + 2],
                    ],
                    color: [1.0, 1.0, 1.0],
                    normal,
                    uv,
                });
            }

            indices.extend(mesh.indices.into_iter().map(|index| index + vertex_offset));
            vertex_offset += total_vertices as u32;
        }

        Self::from_vertices(context, &vertices, &indices)
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        self.context
            .destroy_buffer(self.index_buffer, self.index_memory);
        self.context
            .destroy_buffer(self.vertex_buffer, self.vertex_memory);
    }
}

pub(crate) struct Texture {
    context: Arc<VContext>,
    image: vk::Image,
    memory: vk::DeviceMemory,
    pub image_view: vk::ImageView,
    pub sampler: vk::Sampler,
}

impl Texture {
    fn white(context: Arc<VContext>) -> Result<Self> {
        Self::from_rgba_bytes(
            context,
            vk::Extent3D {
                width: 1,
                height: 1,
                depth: 1,
            },
            &[255, 255, 255, 255],
        )
    }

    fn from_file(context: Arc<VContext>, path: &str) -> Result<Self> {
        let image = image::open(path).map_err(|source| EngineError::Image {
            path: path.into(),
            source,
        })?;
        let rgba = image.to_rgba8();
        let dimensions = image.dimensions();
        Self::from_rgba_bytes(
            context,
            vk::Extent3D {
                width: dimensions.0,
                height: dimensions.1,
                depth: 1,
            },
            rgba.as_raw(),
        )
    }

    fn from_rgba_bytes(context: Arc<VContext>, extent: vk::Extent3D, bytes: &[u8]) -> Result<Self> {
        let (image, memory) = context.create_image_2d(
            extent,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        context.upload_rgba_texture(image, extent, bytes)?;
        let image_view = context.create_image_view(
            image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageAspectFlags::COLOR,
        )?;
        let sampler = context.create_sampler()?;

        Ok(Self {
            context,
            image,
            memory,
            image_view,
            sampler,
        })
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            self.context.device().destroy_sampler(self.sampler, None);
            self.context
                .device()
                .destroy_image_view(self.image_view, None);
        }
        self.context.destroy_image(self.image, self.memory);
    }
}

struct Material {
    descriptor_set: vk::DescriptorSet,
    texture: TextureId,
}

pub struct AssetManager {
    context: Arc<VContext>,
    material_layout: vk::DescriptorSetLayout,
    material_pool: vk::DescriptorPool,
    meshes: SlotMap<MeshId, Mesh>,
    textures: SlotMap<TextureId, Texture>,
    materials: SlotMap<MaterialId, Material>,
    default_material: MaterialId,
}

impl AssetManager {
    pub(crate) fn new(
        context: Arc<VContext>,
        material_layout: vk::DescriptorSetLayout,
    ) -> Result<Self> {
        let pool_sizes = [vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(256)];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(256);
        let material_pool = unsafe {
            context
                .device()
                .create_descriptor_pool(&pool_info, None)
                .map_err(|result| EngineError::vk("creating material descriptor pool", result))?
        };

        let mut assets = Self {
            context: Arc::clone(&context),
            material_layout,
            material_pool,
            meshes: SlotMap::with_key(),
            textures: SlotMap::with_key(),
            materials: SlotMap::with_key(),
            default_material: MaterialId::null(),
        };

        let default_texture = assets.textures.insert(Texture::white(context)?);
        let default_material = assets.create_material(default_texture)?;
        assets.default_material = default_material;

        Ok(assets)
    }

    pub fn load_mesh_obj(&mut self, path: &str) -> Result<MeshId> {
        let mesh = Mesh::from_obj(Arc::clone(&self.context), path)?;
        Ok(self.meshes.insert(mesh))
    }

    pub fn load_texture(&mut self, path: &str) -> Result<TextureId> {
        let texture = Texture::from_file(Arc::clone(&self.context), path)?;
        Ok(self.textures.insert(texture))
    }

    pub fn create_material(&mut self, texture: TextureId) -> Result<MaterialId> {
        let descriptor_set = allocate_descriptor_set(
            self.context.device(),
            self.material_pool,
            self.material_layout,
        )?;
        let texture_ref = self.textures.get(texture).ok_or_else(|| {
            EngineError::Message("invalid texture id passed to create_material".into())
        })?;

        let image_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(texture_ref.image_view)
            .sampler(texture_ref.sampler);
        let write = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&image_info));
        unsafe {
            self.context
                .device()
                .update_descriptor_sets(std::slice::from_ref(&write), &[]);
        }

        Ok(self.materials.insert(Material {
            descriptor_set,
            texture,
        }))
    }

    pub fn default_material(&self) -> MaterialId {
        self.default_material
    }

    pub(crate) fn mesh(&self, id: MeshId) -> Option<&Mesh> {
        self.meshes.get(id)
    }

    pub(crate) fn material_descriptor_set(&self, id: MaterialId) -> Option<vk::DescriptorSet> {
        self.materials
            .get(id)
            .map(|material| material.descriptor_set)
    }

    pub fn material_texture(&self, id: MaterialId) -> Option<TextureId> {
        self.materials.get(id).map(|material| material.texture)
    }
}

impl Drop for AssetManager {
    fn drop(&mut self) {
        unsafe {
            self.context
                .device()
                .destroy_descriptor_pool(self.material_pool, None);
        }
    }
}

fn allocate_descriptor_set(
    device: &ash::Device,
    descriptor_pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
) -> Result<vk::DescriptorSet> {
    let allocate_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(descriptor_pool)
        .set_layouts(std::slice::from_ref(&layout));
    unsafe {
        device
            .allocate_descriptor_sets(&allocate_info)
            .map_err(|result| EngineError::vk("allocating material descriptor set", result))
            .map(|sets| sets[0])
    }
}
