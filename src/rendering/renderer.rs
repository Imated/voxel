use crate::rendering::camera::Camera;
use crate::rendering::global_bindings::GlobalBindings;
use crate::rendering::main_pass::{FrameData, MainRenderPass};
use crate::rendering::render_object::*;
use crate::rendering::shader::Shader;
use crate::rendering::texture::Texture;
use crate::rendering::wgpu_context::{CreateShaderError, CreateTextureError, WGPUContext};
use glam::Vec3;
use std::sync::Arc;
use wgpu::{BindGroupLayout, SurfaceError, TextureViewDescriptor};
use winit::window::Window;

pub struct Renderer {
    window: Arc<Window>,

    context: WGPUContext,

    main_pass: MainRenderPass,

    render_objects: Vec<RenderObject>,
    pub camera: Camera,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let context = WGPUContext::new(window.clone()).await?;

        let main_pass = MainRenderPass;
        let camera = Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: Vec3::Y,
            aspect: context.config.width as f32 / context.config.height as f32,
            fov: 45.0,
            near_clip: 0.1,
            far_clip: 100.0,
        };

        Ok(Self {
            window,
            context,
            main_pass,
            render_objects: vec![],
            camera,
        })
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
        self.context.is_surface_configured = true;
    }

    pub fn render(&mut self, global_bindings: &GlobalBindings) -> Result<(), SurfaceError> {
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
            global_bind_group: global_bindings.bind_group(),
        };

        let main_objects: Vec<&RenderObject> = self
            .render_objects
            .iter()
            .filter(|&obj| obj.pass == self.main_pass.pass_type())
            .collect(); // pass all non-transparent objects into the main pass.

        self.main_pass
            .record(&mut encoder, &frame_data, &main_objects);

        self.render_objects.clear();
        context.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        output.present();

        Ok(())
    }

    pub fn push_object(&mut self, obj: &RenderObject) {
        self.render_objects.push(obj.clone());
    }

    pub fn create_shader(
        &self,
        path: &str,
        material_layout: BindGroupLayout,
        global_bindings: &GlobalBindings,
    ) -> Result<Shader, CreateShaderError> {
        self.context
            .create_shader(path, &global_bindings.bind_group_layout(), &material_layout)
    }

    pub fn create_texture(&self, path: &str) -> Result<Texture, CreateTextureError> {
        self.context.create_texture(path)
    }

    pub fn context(&self) -> &WGPUContext {
        &self.context
    }
}
