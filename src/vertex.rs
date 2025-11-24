use bytemuck::{Pod, Zeroable};
use wgpu::{
    BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode,
    vertex_attr_array,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 3],
}

impl Vertex {
    pub(crate) fn desc() -> VertexBufferLayout<'static> {
        const ATTRIBS: [VertexAttribute; 2] = vertex_attr_array![0 => Float32x3, 1 => Float32x3];

        VertexBufferLayout {
            array_stride: size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}
