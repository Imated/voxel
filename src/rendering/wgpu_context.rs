use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use thiserror::Error;
use wgpu::MemoryHints::Performance;
use wgpu::PowerPreference::HighPerformance;
use wgpu::PresentMode::Mailbox;
use wgpu::{
    Adapter, Backends, CreateSurfaceError, Device, DeviceDescriptor, Features, Instance,
    InstanceDescriptor, Limits, PresentMode, Queue, RequestAdapterError, RequestAdapterOptions,
    RequestDeviceError, Surface, SurfaceConfiguration, TextureUsages, Trace,
};
use winit::window::Window;

#[derive(Error, Debug)]
pub enum CreateWGPUContextError {
    #[error("Failed to create window surface due to {0:?}.")]
    CreateSurfaceError(#[from] CreateSurfaceError),
    #[error("Failed to request a viable adapter due to {0:?}.")]
    RequestAdapterError(#[from] RequestAdapterError),
    #[error("Failed to request a viable device due to {0:?}.")]
    RequestDeviceError(#[from] RequestDeviceError),
}

pub struct WGPUContext {
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) config: SurfaceConfiguration,
    pub(crate) surface: Surface<'static>,
    pub(crate) is_surface_configured: bool,
}

impl WGPUContext {
    pub async fn new(window: Arc<Window>) -> Result<Self, CreateWGPUContextError> {
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await?;

        let surface_config =
            Self::setup_surface_config(&instance, &adapter, &surface, window.clone());

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                required_limits: Limits::default(),
                memory_hints: Performance,
                trace: Trace::Off,
            })
            .await?;

        Ok(Self {
            device,
            queue,
            config: surface_config,
            surface,
            is_surface_configured: false,
        })
    }

    fn setup_surface_config(
        instance: &Instance,
        adapter: &Adapter,
        surface: &Surface,
        window: Arc<Window>,
    ) -> SurfaceConfiguration {
        let size = window.inner_size();

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

        config
    }
}
