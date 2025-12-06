use crate::rendering::vertex::Vertex;
use crate::rendering::wgpu_context::WGPUContext;
use std::fs;
use wgpu::naga::FastHashMap;
use wgpu::{
    BindGroupLayout, BlendState, ColorTargetState, ColorWrites, Device, Face, FragmentState,
    FrontFace, MultisampleState, PipelineCompilationOptions, PipelineLayout,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline,
    RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource,
    SurfaceConfiguration, VertexState,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ShaderId(pub u32);

pub struct Shaders {
    pub shaders: FastHashMap<ShaderId, Shader>,
}

impl Shaders {
    pub fn new() -> Self {
        Self {
            shaders: FastHashMap::default(),
        }
    }

    pub fn add(&mut self, id: u32, shader: Shader) {
        self.shaders.insert(ShaderId(id), shader);
    }

    pub fn get(&self, id: u32) -> Option<&Shader> {
        self.shaders.get(&ShaderId(id))
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Shader {
    pub(crate) module: ShaderModule,
    pub(crate) pipeline: RenderPipeline,
    pub(crate) global_layout: BindGroupLayout,
    pub(crate) material_layout: BindGroupLayout,
}
