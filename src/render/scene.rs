use std::{
    cell::RefCell,
    sync::Arc,
    time::{Duration, SystemTime},
};

use crate::{
    backend::{render_loop::RenderContext, vcontext::Vcontext},
    core::{
        camera::{Camera, CameraBufferObject},
        mesh::Mesh,
    },
    render::{geometry::RenderGeometry, vertex_3d::Vertex3D},
};

pub struct SceneFrameState {
    last_fps_save_time: SystemTime,
    count_since_fps_save: usize,
    pub fps: usize,
}

pub struct Scene {
    vcontext: Arc<Vcontext>,
    camera: Camera,
    meshes: Vec<Mesh>,

    render_geometry: RenderGeometry,
    descriptor_pool: ash::vk::DescriptorPool,
    cbo_set: ash::vk::DescriptorSet,
    pub frame_state: RefCell<SceneFrameState>,
}

impl Scene {
    pub fn new(vcontext: Arc<Vcontext>) -> Self {
        let camera = Camera::new(&vcontext);

        let a_vertices = vec![
            Vertex3D {
                pos: glam::Vec3 {
                    x: -0.5,
                    y: 0.5,
                    z: 0.0,
                },
                color: glam::Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            Vertex3D {
                pos: glam::Vec3 {
                    x: 0.0,
                    y: -0.5,
                    z: 0.0,
                },
                color: glam::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
            },
            Vertex3D {
                pos: glam::Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 0.0,
                },
                color: glam::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                },
            },
        ];

        let b_vertices = vec![
            Vertex3D {
                pos: glam::Vec3 {
                    x: -0.5,
                    y: 0.2,
                    z: 1.0,
                },
                color: glam::Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            Vertex3D {
                pos: glam::Vec3 {
                    x: 0.0,
                    y: -0.8,
                    z: 1.0,
                },
                color: glam::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
            },
            Vertex3D {
                pos: glam::Vec3 {
                    x: 0.5,
                    y: 0.2,
                    z: 1.0,
                },
                color: glam::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                },
            },
        ];

        let a_mesh = Mesh::new(vcontext.clone(), a_vertices);
        let b_mesh = Mesh::new(vcontext.clone(), b_vertices);

        let render_geometry = RenderGeometry::new(vcontext.clone());

        let descriptor_pool_size = ash::vk::DescriptorPoolSize::default()
            .descriptor_count(1)
            .ty(ash::vk::DescriptorType::UNIFORM_BUFFER);
        let descriptor_pool_info = ash::vk::DescriptorPoolCreateInfo::default()
            .max_sets(1)
            .pool_sizes(std::slice::from_ref(&descriptor_pool_size));
        let descriptor_pool = unsafe {
            vcontext
                .device
                .create_descriptor_pool(&descriptor_pool_info, None)
                .expect("unable to create descriptor pool")
        };
        let cbo_set_info = ash::vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(std::slice::from_ref(&render_geometry.ubo_set_layout));
        let cbo_set = unsafe {
            vcontext
                .device
                .allocate_descriptor_sets(&cbo_set_info)
                .expect("unable to create set")[0]
        };

        let cbo_buffer_info = ash::vk::DescriptorBufferInfo::default()
            .buffer(camera.buffer)
            .offset(0)
            .range(size_of::<CameraBufferObject>() as u64);

        let write_cbo_set = ash::vk::WriteDescriptorSet::default()
            .descriptor_count(1)
            .descriptor_type(ash::vk::DescriptorType::UNIFORM_BUFFER)
            .dst_binding(0)
            .dst_set(cbo_set)
            .buffer_info(std::slice::from_ref(&cbo_buffer_info));

        unsafe {
            vcontext
                .device
                .update_descriptor_sets(std::slice::from_ref(&write_cbo_set), &[]);
        }

        let frame_state = SceneFrameState {
            last_fps_save_time: SystemTime::now(),
            count_since_fps_save: 0,
            fps: 0,
        };

        Self {
            vcontext,
            camera,
            meshes: vec![a_mesh, b_mesh],

            render_geometry,
            descriptor_pool,
            cbo_set,
            frame_state: RefCell::new(frame_state),
        }
    }

    pub fn render(&self, context: &RenderContext) {
        //Track fps
        let mut frame_state = self.frame_state.borrow_mut();
        frame_state.count_since_fps_save += 1;
        if SystemTime::now()
            .duration_since(frame_state.last_fps_save_time)
            .unwrap()
            >= Duration::from_secs(1)
        {
            frame_state.fps = frame_state.count_since_fps_save;
            frame_state.count_since_fps_save = 0;
            frame_state.last_fps_save_time = SystemTime::now();
        }

        let device = &self.vcontext.device;
        let cmd = context.cmd;
        unsafe {
            device.cmd_bind_pipeline(
                cmd,
                ash::vk::PipelineBindPoint::GRAPHICS,
                self.render_geometry.pipeline,
            );

            device.cmd_bind_descriptor_sets(
                cmd,
                ash::vk::PipelineBindPoint::GRAPHICS,
                self.render_geometry.pipeline_layout,
                0,
                std::slice::from_ref(&self.cbo_set),
                &[],
            );

            for mesh in &self.meshes {
                mesh.draw(cmd);
            }
        };
    }
}
