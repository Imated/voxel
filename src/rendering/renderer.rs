use crate::rendering::camera::Camera;
use crate::rendering::main_pass::{FrameData, MainRenderPass};
use crate::rendering::mesh::Mesh;
use crate::rendering::render_object::*;
use crate::rendering::shader::Shader;
use crate::rendering::texture::Texture;
use crate::rendering::utils::bind_group_builder::BindGroupBuilder;
use crate::rendering::utils::bind_group_layout_builder::BindGroupLayoutBuilder;
use crate::rendering::utils::sampler_builder::SamplerBuilder;
use crate::rendering::wgpu_context::{CreateShaderError, CreateTextureError, WGPUContext};
use bytemuck::{Pod, Zeroable, cast_slice};
use glam::{Mat4, Vec3};
use std::sync::Arc;
use wgpu::AddressMode::ClampToEdge;
use wgpu::BufferBindingType::Uniform;
use wgpu::FilterMode::Nearest;
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, BufferUsages, Sampler, ShaderStages, SurfaceError,
    TextureViewDescriptor,
};
use winit::window::Window;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct SceneData {
    view_proj: Mat4,
}

pub struct Renderer {
    window: Arc<Window>,

    context: WGPUContext,

    main_pass: MainRenderPass,
    sampler: Sampler,

    render_objects: Vec<RenderObject>,
    pub camera: Camera,

    scene_bind_group: BindGroup,
    scene_bind_group_layout: BindGroupLayout,
    scene_data_buffer: Buffer,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let context = WGPUContext::new(window.clone()).await?;

        let sampler = SamplerBuilder::new()
            .with_mode(ClampToEdge)
            .with_filtering(Nearest)
            .build(&context);

        let main_pass = MainRenderPass::new();
        let camera = Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: Vec3::Y,
            aspect: context.config.width as f32 / context.config.height as f32,
            fov: 45.0,
            near_clip: 0.1,
            far_clip: 100.0,
        };

        let scene_data = SceneData {
            view_proj: Mat4::IDENTITY,
        };

        let scene_data_buffer = context.create_buffer(
            &[scene_data],
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let scene_bind_group_layout = BindGroupLayoutBuilder::new()
            .with_buffer(ShaderStages::VERTEX_FRAGMENT, Uniform)
            .build(&context);

        let scene_bind_group = BindGroupBuilder::new()
            .with_buffer(&scene_data_buffer)
            .build(&context, &scene_bind_group_layout);

        Ok(Self {
            window,
            context,
            main_pass,
            sampler,
            render_objects: vec![],
            camera,
            scene_bind_group,
            scene_bind_group_layout,
            scene_data_buffer,
        })
    }

    pub fn update_scene_data(&self) {
        self.update_scene_data_with(SceneData {
            view_proj: self.camera.build_view_projection_matrix(),
        });
    }

    pub fn update_scene_data_with(&self, scene_data: SceneData) {
        self.context
            .queue
            .write_buffer(&self.scene_data_buffer, 0, cast_slice(&[scene_data]));
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.context.config.width = width;
        self.context.config.height = height;

        if width == 0 && height == 0 {
            return;
        }

        self.context
            .surface
            .configure(&self.context.device, &self.context.config);

        self.camera.aspect = width as f32 / height as f32;
        self.update_scene_data();

        self.context.is_surface_configured = true;
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let context = &self.context;
        if (context.config.width == 0 && context.config.height == 0)
            || !context.is_surface_configured
        {
            return Ok(());
        }

        self.window.request_redraw();

        let output = context.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = context.device.create_command_encoder(&Default::default());

        let frame_data = FrameData {
            color: &view,
            scene_bind_group: self.scene_bind_group.clone(),
        };

        let main_objects: Vec<&RenderObject> = self
            .render_objects
            .iter()
            .filter(|obj| obj.pass == self.main_pass.pass_type())
            .collect(); // pass all non-transparent objects into the main pass.

        self.main_pass
            .record(&mut encoder, &frame_data, &main_objects);

        self.render_objects.clear();
        context.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        output.present();

        Ok(())
    }

    pub fn push_object(&mut self, obj: RenderObject) {
        self.render_objects.push(obj);
    }

    pub fn create_shader(
        &self,
        path: &str,
        material_layout: BindGroupLayout,
    ) -> Result<Shader, CreateShaderError> {
        self.context
            .create_shader(path, &self.scene_bind_group_layout, &material_layout)
    }

    pub fn create_texture(&self, path: &str) -> Result<Texture, CreateTextureError> {
        self.context.create_texture(path)
    }

    pub fn create_mesh<V, I>(&self, vertices: &[V], indices: &[I], start_index: u32) -> Mesh
    where
        V: Pod + Zeroable,
        I: Pod + Zeroable,
    {
        let vertex_buffer = self.context.create_buffer(vertices, BufferUsages::VERTEX);
        let index_buffer = self.context.create_buffer(indices, BufferUsages::INDEX);
        let num_indices = indices.len() as u32;

        Mesh {
            vertices: vertex_buffer,
            indices: index_buffer,
            num_indices,
            start_index,
        }
    }

    pub fn universal_sampler(&self) -> &Sampler {
        &self.sampler
    }

    pub fn context(&self) -> &WGPUContext {
        &self.context
    }
}
