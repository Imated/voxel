use wgpu::BindGroup;
use crate::material::MaterialId;
use crate::mesh::MeshId;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RenderObject {
    pub mesh: MeshId,
    pub material: MaterialId,
    pub model_bind_group: Option<BindGroup>,
    pub transparent: bool,
}
