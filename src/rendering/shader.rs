use wgpu::{BindGroupLayout, RenderPipeline, ShaderModule};

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Shader {
    pub(crate) module: ShaderModule,
    pub(crate) pipeline: RenderPipeline,
    pub(crate) global_layout: BindGroupLayout,
    pub(crate) material_layout: BindGroupLayout,
}
