use wgpu::Buffer;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mesh {
    pub(crate) vertices: Buffer,
    pub(crate) indices: Buffer,
    pub(crate) num_indices: u32,
    pub(crate) start_index: u32,
}
