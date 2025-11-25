use crate::shader::Shader;
use crate::vertex::Vertex;
use wgpu::util::{BufferInitDescriptor, DeviceExt, RenderEncoder};
use wgpu::{BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferUsages, Color, CommandEncoder, Device, IndexFormat, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, Sampler, SamplerBindingType, ShaderStages, StoreOp, SurfaceConfiguration, TextureSampleType, TextureView, TextureViewDimension};

pub struct FrameData<'a> {
    pub color: &'a TextureView,
}

pub struct MainRenderPass {
    default_shader: Shader,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,
}

const TRIANGLE_VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.625, 0.0],
        uv: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        uv: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        uv: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.0, -0.5, 0.0],
        uv: [0.0, 0.5, 0.5],
    },
    Vertex {
        position: [-0.25, 0.125, 0.0],
        uv: [0.5, 0.5, 0.0],
    },
    Vertex {
        position: [0.25, 0.125, 0.0],
        uv: [0.0, 0.5, 0.0],
    },
];

const TRIANGLE_INDICES: &[u16] = &[0, 4, 5, 1, 3, 4, 2, 5, 3];

impl MainRenderPass {
    pub fn new(device: &Device, config: &SurfaceConfiguration, sampler: Sampler) -> anyhow::Result<Self> {
        let shader_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let shader_bind_group = BindGroupDescriptor {
            label: None,
            layout: &shader_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        };
        let shader = Shader::new(device, config, "/res/shaders/default.wgsl", shader_layout, shader_bind_group)?;

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(TRIANGLE_VERTICES),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(TRIANGLE_INDICES),
            usage: BufferUsages::INDEX,
        });

        let num_indices = TRIANGLE_INDICES.len() as u32;

        Ok(Self {
            default_shader: shader,
            vertex_buffer,
            index_buffer,
            num_indices,
        })
    }

    pub fn record(&mut self, encoder: &mut CommandEncoder, data: &FrameData) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &data.color,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.default_shader.pipeline);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);

        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

        drop(render_pass);
    }
}
