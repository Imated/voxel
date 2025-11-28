use crate::rendering::main_pass::{FrameData, MainRenderPass};
use crate::rendering::material::Material;
use crate::rendering::mesh::Mesh;
use crate::rendering::render_object::*;
use crate::rendering::shader::Shader;
use crate::rendering::texture::Texture;
use std::sync::Arc;
use wgpu::PresentMode::Mailbox;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::wgt::SamplerDescriptor;
use wgpu::{
    AddressMode, Backends, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
    BufferUsages, Device, DeviceDescriptor, Features, FilterMode, Instance, InstanceDescriptor,
    Limits, PresentMode, Queue, RequestAdapterOptions, Sampler, SamplerBindingType, ShaderStages,
    Surface, SurfaceConfiguration, SurfaceError, TextureUsages, TextureViewDescriptor, Trace,
};
use winit::window::Window;
use crate::rendering::vertex::Vertex;

pub struct Renderer {
    window: Arc<Window>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    surface: Surface<'static>,
    is_surface_configured: bool,

    main_pass: MainRenderPass,
    sampler: Sampler,

    render_objects: Vec<RenderObject>,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: Default::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                required_limits: Limits::default(),
                memory_hints: Default::default(),
                trace: Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|format| format.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps // Mailbox if supported, otherwise FIFO (guaranteed to be supported)
                .present_modes
                .contains(&Mailbox)
                .then(|| Mailbox)
                .unwrap_or(PresentMode::Fifo),
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let main_pass = MainRenderPass::new();

        Ok(Self {
            window,
            device,
            queue,
            config,
            surface,
            is_surface_configured: false,
            main_pass,
            sampler,
            render_objects: vec![],
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width <= 0 && height <= 0 {
            self.config.width = width;
            self.config.height = height;
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.is_surface_configured = true;
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        if self.config.width <= 0 && self.config.height <= 0 {
            return Ok(());
        }
        if !self.is_surface_configured {
            return Ok(());
        }

        self.window.request_redraw();

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&Default::default());

        let frame_data = FrameData { color: &view };

        let main_objects: Vec<&RenderObject> = self
            .render_objects
            .iter()
            .filter(|obj| !obj.transparent)
            .collect(); // pass all non-transparent objects into the main pass.

        self.main_pass
            .record(&mut encoder, &frame_data, &main_objects);

        self.render_objects.clear();
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        output.present();

        Ok(())
    }

    pub fn push_object(&mut self, obj: RenderObject) {
        self.render_objects.push(obj);
    }

    pub fn create_shader(&self, path: &str, layout: BindGroupLayout) -> anyhow::Result<Shader> {
        Shader::new(&self.device, &self.config, path, layout)
    }

    pub fn create_shader_layout(&self, mut entries: Vec<BindGroupLayoutEntry>) -> BindGroupLayout {
        entries.push(BindGroupLayoutEntry {
            binding: entries.len() as u32,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
        }); // add universal sampler

        self.device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &entries,
            })
    }

    pub fn create_material(&self, shader: &Shader, entries: Vec<BindGroupEntry>) -> Material {
        let mut entries = entries;
        entries.push(BindGroupEntry {
            binding: entries.len() as u32,
            resource: BindingResource::Sampler(&self.sampler),
        }); // add universal sampler

        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &shader.bind_group_layout,
            entries: &entries,
        });

        Material {
            shader: shader.clone(), // only clones pointer to internal gpu resources
            bind_group,
        }
    }

    pub fn load_texture(&self, path: &str) -> anyhow::Result<Texture> {
        Texture::new(&self.device, &self.queue, path)
    }

    pub fn create_buffer(&self, contents: &[u8], usage: BufferUsages) -> Buffer {
        self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents,
            usage,
        })
    }

    pub fn create_mesh(&self, vertices: &[Vertex], indices: &[u16]) -> Mesh {
        let vertex_buffer =
            self.create_buffer(bytemuck::cast_slice(vertices), BufferUsages::VERTEX);
        let index_buffer = self.create_buffer(bytemuck::cast_slice(indices), BufferUsages::INDEX);
        let num_indices = indices.len() as u32;

        Mesh::new(vertex_buffer, index_buffer, num_indices)
    }
}
