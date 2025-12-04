use crate::rendering::main_pass::{FrameData, MainRenderPass};
use crate::rendering::mesh::Mesh;
use crate::rendering::render_object::*;
use crate::rendering::shader::Shader;
use crate::rendering::texture::Texture;
use crate::rendering::vertex::Vertex;
use bytemuck::{cast_slice, Pod, Zeroable};
use std::sync::Arc;
use glam::{Mat4, Vec3};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::wgt::SamplerDescriptor;
use wgpu::PresentMode::Mailbox;
use wgpu::{AddressMode, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, Device, DeviceDescriptor, Features, FilterMode, Instance, InstanceDescriptor, Limits, PresentMode, Queue, RequestAdapterOptions, Sampler, ShaderStages, Surface, SurfaceConfiguration, SurfaceError, TextureUsages, TextureViewDescriptor, Trace};
use winit::window::Window;
use crate::rendering::camera::Camera;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct SceneData {
    view_proj: Mat4,
}

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
    pub camera: Camera,

    scene_bind_group: BindGroup,
    scene_bind_group_layout: BindGroupLayout,
    scene_data_buffer: Buffer,
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
        let camera = Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: Vec3::Y,
            aspect: config.width as f32 / config.height as f32,
            fov: 45.0,
            near_clip: 0.1,
            far_clip: 100.0,
        };

        let scene_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Scene Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
        });

        let scene_data = SceneData {
            view_proj: Mat4::IDENTITY,
        };

        let scene_data_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Scene Data Buffer"),
            contents: cast_slice(&[scene_data]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let scene_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Scene Bind Group"),
            layout: &scene_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: scene_data_buffer.as_entire_binding(),
            }],
        });

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
        self.queue.write_buffer(&self.scene_data_buffer, 0, cast_slice(&[scene_data]));
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

        self.camera.aspect = width as f32 / height as f32;
        self.update_scene_data();

        self.is_surface_configured = true;
    }

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

        let frame_data = FrameData { color: &view, scene_bind_group: (&self.scene_bind_group).clone() };

        let main_objects: Vec<&RenderObject> = self
            .render_objects
            .iter()
            .filter(|obj| obj.pass == self.main_pass.pass_type())
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
        Shader::new(&self.device, &self.config, path, [&self.scene_bind_group_layout, &layout])
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

    pub fn create_mesh<V, I>(&self, vertices: &[Vertex], indices: &[u16], start_index: u32) -> Mesh
    where
        V: Pod + Zeroable,
        I: Pod + Zeroable,
    {
        let vertex_buffer =
            self.create_buffer(bytemuck::cast_slice(vertices), BufferUsages::VERTEX);
        let index_buffer = self.create_buffer(bytemuck::cast_slice(indices), BufferUsages::INDEX);
        let num_indices = indices.len() as u32;

        Mesh {
            vertices: vertex_buffer,
            indices: index_buffer,
            num_indices,
            start_index,
        }
    }

    pub(crate) fn create_bind_group_layout(&self, entries: Vec<BindGroupLayoutEntry>) -> BindGroupLayout {
        self.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &entries,
        })
    }

    pub(crate) fn create_bind_group(&self, entries: Vec<BindGroupEntry>, layout: &BindGroupLayout) -> BindGroup {
        self.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout,
            entries: &entries,
        })
    }

    pub fn universal_sampler(&self) -> &Sampler {
        &self.sampler
    }
}
