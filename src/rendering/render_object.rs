use crate::rendering::material::Material;
use crate::rendering::mesh::Mesh;
use wgpu::{BindGroup, Buffer};

#[derive(Clone, Debug, PartialEq)]
pub struct RenderObject {
    pub mesh: Mesh,
    pub material: Material,
    pub model_bind_group: Option<BindGroup>,
    pub pass: PassType,
    pub instances: Buffer,
    pub num_instances: u32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PassType {
    Opaque,
    Transparent,
}
