use crate::rendering::material::Material;
use crate::rendering::mesh::Mesh;
use wgpu::Buffer;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderObject {
    pub mesh: Mesh,
    pub material: Material,
    pub pass: PassType,
    pub instances: InstanceBuffer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceBuffer {
    pub buffer: Buffer,
    pub len: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PassType {
    Opaque,
    Transparent,
}
