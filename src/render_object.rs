use crate::material::Material;
use crate::mesh::Mesh;
use wgpu::BindGroup;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderObject {
    pub mesh: Mesh,
    pub material: Material,
    pub model_bind_group: Option<BindGroup>,
    pub transparent: bool,
}
