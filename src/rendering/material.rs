use crate::rendering::shader::Shader;
use wgpu::BindGroup;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Material {
    pub shader: Shader,
    pub bind_group: BindGroup,
}
