use crate::rendering::material::Material;
use crate::rendering::mesh::Mesh;

#[derive(Clone, Debug, PartialEq)]
pub struct RenderObject {
    pub mesh: Mesh,
    pub material: Material,
    pub pass: PassType,
    pub instances: wgpu::Buffer,
    pub instances_len: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PassType {
    Opaque,
    Transparent,
}
