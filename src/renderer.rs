use crate::main_pass::{FrameData, MainRenderPass};
use anyhow::Error;
use std::sync::Arc;
use wgpu::PresentMode::Mailbox;
use wgpu::{AddressMode, Backends, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Device, DeviceDescriptor, Features, FilterMode, Instance, InstanceDescriptor, Limits, PresentMode, Queue, RequestAdapterOptions, Sampler, SamplerBindingType, ShaderStages, Surface, SurfaceConfiguration, SurfaceError, TextureSampleType, TextureUsages, TextureViewDescriptor, TextureViewDimension, Trace};
use wgpu::wgt::SamplerDescriptor;
use winit::window::Window;

pub struct Renderer {
    window: Arc<Window>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    surface: Surface<'static>,

    main_pass: MainRenderPass,
    sampler: Sampler
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

        let main_pass = MainRenderPass::new(&device, &config)?;

        Ok(Self {
            window,
            device,
            queue,
            config,
            surface,
            main_pass,
            sampler,
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
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        if self.config.width <= 0 && self.config.height <= 0 {
            return Ok(());
        }

        self.window.request_redraw();

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&Default::default());

        let frame_data = FrameData { color: &view };

        self.main_pass.record(&mut encoder, &frame_data);

        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        output.present();

        Ok(())
    }
}
