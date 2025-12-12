use crate::rendering::buffer::Buffer;
use crate::rendering::vertex::Vertex;

#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    pub(crate) vertices: Buffer<Vertex>,
    pub(crate) indices: Buffer<u16>,
    pub(crate) num_indices: u32,
    pub(crate) start_index: u32,
}
