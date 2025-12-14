use crate::rendering::render_object::{PassType, RenderObject};
use wgpu::{
    BindGroup, Color, CommandEncoder, IndexFormat, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp, TextureView,
};

pub struct FrameData<'a> {
    pub color: &'a TextureView,
    pub global_bind_group: &'a BindGroup,
}

pub struct MainRenderPass;

impl MainRenderPass {
    pub fn record(
        &mut self,
        encoder: &mut CommandEncoder,
        data: &FrameData,
        objects: &[&RenderObject],
    ) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: data.color,
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

        for &object in objects {
            let material = &object.material;
            let shader = &material.shader;
            let mesh = &object.mesh;

            render_pass.set_pipeline(&shader.pipeline);

            render_pass.set_bind_group(0, data.global_bind_group, &[]);
            render_pass.set_bind_group(1, &material.bind_group, &[]);

            render_pass.set_vertex_buffer(0, mesh.vertices.buffer().slice(..));
            render_pass.set_vertex_buffer(1, object.instances.slice(..));
            render_pass.set_index_buffer(mesh.indices.buffer().slice(..), IndexFormat::Uint16);

            render_pass.draw_indexed(
                mesh.start_index..mesh.num_indices,
                0,
                0..object.instances_len,
            );
        }
    }

    pub fn pass_type(&self) -> PassType {
        PassType::Opaque
    }
}
