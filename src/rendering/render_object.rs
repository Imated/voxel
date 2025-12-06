use crate::rendering::material::Material;
use crate::rendering::mesh::Mesh;
use wgpu::BindGroup;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderObject {
    pub mesh: Mesh,
    pub material: Material,
    pub model_bind_group: Option<BindGroup>,
    pub pass: PassType,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PassType {
    Opaque,
    Transparent,
}
