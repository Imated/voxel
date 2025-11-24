use wgpu::{Color, CommandEncoder, Device, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp, SurfaceConfiguration, TextureView};
use crate::shader::Shader;

pub struct FrameData<'a> {
    pub color: &'a TextureView,
}

pub struct MainRenderPass {
    default_shader: Shader,
}

impl MainRenderPass {
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> anyhow::Result<Self> {
        let shader = Shader::new(device, config, "/res/shaders/default.wgsl")?;

        Ok(Self {
            default_shader: shader
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
        render_pass.draw(0..3, 0..1);

        drop(render_pass);
    }
}