use wgpu::{Buffer, BufferUsages, Color, CommandEncoder, Device, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp, SurfaceConfiguration, TextureView};
use wgpu::util::{BufferInitDescriptor, DeviceExt, RenderEncoder};
use crate::shader::Shader;
use crate::vertex::Vertex;

pub struct FrameData<'a> {
    pub color: &'a TextureView,
}

pub struct MainRenderPass {
    default_shader: Shader,
    vertex_buffer: Buffer,
    num_vertices: u32,
}

const TRIANGLE_VERTICES: &[Vertex] = &[
    Vertex { position: [0.0, 0.5, 0.0], uv: [1.0, 0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, 0.0], uv: [0.0, 1.0, 0.0] },
    Vertex { position: [0.5, -0.5, 0.0], uv: [0.0, 0.0, 1.0] },
];

impl MainRenderPass {
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> anyhow::Result<Self> {

        let shader = Shader::new(device, config, "/res/shaders/default.wgsl")?;

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(TRIANGLE_VERTICES),
            usage: BufferUsages::VERTEX,
        });

        let num_vertices = TRIANGLE_VERTICES.len() as u32;

        Ok(Self {
            default_shader: shader,
            vertex_buffer,
            num_vertices
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

        render_pass.draw(0..self.num_vertices, 0..1);

        drop(render_pass);
    }
}