use crate::render_object::RenderObject;
use wgpu::{
    Color, CommandEncoder, LoadOp, Operations,
    RenderPassColorAttachment, RenderPassDescriptor, StoreOp, TextureView,
};

pub struct FrameData<'a> {
    pub color: &'a TextureView,
}

pub struct MainRenderPass {

}

impl MainRenderPass {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn record(
        &mut self,
        encoder: &mut CommandEncoder,
        data: &FrameData,
        objects: Vec<&RenderObject>,
    ) {
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

        for object in objects {

            //render_pass.set_pipeline();

            //render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            //render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);

            //render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        drop(render_pass);
    }
}
