use crate::cubes::CubeData;
use crate::rendering::shader::Shader;
use crate::rendering::texture::Texture;
use crate::rendering::vertex::Vertex;
use bytemuck::checked::cast_slice;
use bytemuck::{Pod, Zeroable};
use image::{ImageError, ImageReader};
use std::fmt::Debug;
use std::sync::Arc;
use std::{fs, io};
use thiserror::Error;
use wgpu::MemoryHints::Performance;
use wgpu::PowerPreference::HighPerformance;
use wgpu::PresentMode::{Fifo, Mailbox};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    Adapter, Backends, BindGroupLayout, BlendState, Buffer, BufferUsages, ColorTargetState,
    ColorWrites, CreateSurfaceError, Device, DeviceDescriptor, Extent3d, Face, Features,
    FragmentState, FrontFace, Instance, InstanceDescriptor, Limits, MultisampleState, Origin3d,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode, PrimitiveState,
    PrimitiveTopology, Queue, RenderPipeline, RenderPipelineDescriptor, RequestAdapterError,
    RequestAdapterOptions, RequestDeviceError, ShaderModule, ShaderModuleDescriptor, ShaderSource,
    Surface, SurfaceConfiguration, TexelCopyBufferLayout, TexelCopyTextureInfo, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor,
    Trace, VertexState,
};
use winit::window::Window;

#[derive(Error, Debug)]
pub enum CreateWGPUContextError {
    #[error("Failed to create window surface due to {0:?}.")]
    CreateSurface(#[from] CreateSurfaceError),
    #[error("Failed to request a viable adapter due to {0:?}.")]
    RequestAdapter(#[from] RequestAdapterError),
    #[error("Failed to request a viable device due to {0:?}.")]
    RequestDevice(#[from] RequestDeviceError),
}

#[derive(Error, Debug)]
pub enum CreateShaderError {
    #[error("Failed to read shader file due to {0:?}.")]
    IoError(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum CreateTextureError {
    #[error("Failed to read texture file due to {0:?}.")]
    IoError(#[from] io::Error),
    #[error("Failed to decode texture due to {0:?}.")]
    DecodeError(#[from] ImageError),
}

pub struct WGPUContext {
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) config: SurfaceConfiguration,
    pub(crate) surface: Surface<'static>,
    pub(crate) is_surface_configured: bool, // MacOS/Metal support
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

        let surface_config = Self::setup_surface_config(&adapter, &surface, window.clone());

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
        adapter: &Adapter,
        surface: &Surface,
        window: Arc<Window>,
    ) -> SurfaceConfiguration {
        let size = window.inner_size();

        let surface_caps = surface.get_capabilities(adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|format| format.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: if surface_caps // Mailbox if supported, otherwise FIFO (guaranteed to be supported)
                .present_modes
                .contains(&Mailbox)
            {
                Mailbox
            } else {
                Fifo
            },
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        }
    }

    pub(crate) fn create_texture(&self, path: &str) -> Result<Texture, CreateTextureError> {
        let image = ImageReader::open(env!("CARGO_MANIFEST_DIR").to_owned() + path)?.decode()?;
        let image_rgba = image.to_rgba8();

        use image::GenericImageView;
        let (width, height) = image.dimensions();

        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1, // 1 layer for 2d texture
        };

        let texture = self.device.create_texture(&TextureDescriptor {
            label: Some(path),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST, // texture_binding = use in shaders, copy_dst = copy data to texture
            view_formats: &[],
        });

        self.queue.write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &image_rgba,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&TextureViewDescriptor::default());

        Ok(Texture {
            size,
            data: texture,
            view,
        })
    }

    pub(crate) fn create_shader(
        &self,
        path: &str,
        global_layout: &BindGroupLayout,
        material_layout: &BindGroupLayout,
    ) -> Result<Shader, CreateShaderError> {
        let src = fs::read_to_string(env!("OUT_DIR").to_owned() + path)?;
        let shader = self.device.create_shader_module(ShaderModuleDescriptor {
            label: Some(path),
            source: ShaderSource::Wgsl(src.into()),
        });

        let pipeline = self.create_render_pipeline(&shader, [global_layout, material_layout]);

        Ok(Shader {
            module: shader,
            pipeline,
            global_layout: global_layout.clone(),
            material_layout: material_layout.clone(),
        })
    }

    pub(crate) fn create_render_pipeline(
        &self,
        shader: &ShaderModule,
        layouts: [&BindGroupLayout; 2],
    ) -> RenderPipeline {
        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &layouts,
                    push_constant_ranges: &[],
                });

        self.device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&render_pipeline_layout),
                vertex: VertexState {
                    module: shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Vertex::desc(), CubeData::desc()],
                    compilation_options: PipelineCompilationOptions::default(),
                },
                fragment: Some(FragmentState {
                    module: shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(ColorTargetState {
                        format: self.config.format,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    })],
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: Some(Face::Back),
                    unclipped_depth: false,
                    polygon_mode: PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            })
    }
}
