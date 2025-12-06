use crate::rendering::renderer::Renderer;
use crate::rendering::wgpu_context::WGPUContext;
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferBindingType, SamplerBindingType, ShaderStages, TextureSampleType, TextureViewDimension,
};

pub struct BindGroupLayoutBuilder {
    pub(crate) entries: Vec<BindGroupLayoutEntry>,
}

impl BindGroupLayoutBuilder {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn with_texture2d(mut self, visibility: ShaderStages) -> Self {
        self.entries.push(BindGroupLayoutEntry {
            binding: self.entries.len() as u32,
            visibility,
            ty: BindingType::Texture {
                sample_type: TextureSampleType::Float { filterable: true },
                view_dimension: TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        });

        self
    }

    pub fn with_sampler(mut self, visibility: ShaderStages) -> Self {
        self.entries.push(BindGroupLayoutEntry {
            binding: self.entries.len() as u32,
            visibility,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
        });

        self
    }

    pub fn with_buffer(mut self, visibility: ShaderStages, buffer_type: BufferBindingType) -> Self {
        self.entries.push(BindGroupLayoutEntry {
            binding: self.entries.len() as u32,
            visibility,
            ty: BindingType::Buffer {
                ty: buffer_type,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });

        self
    }

    pub fn build(self, context: &WGPUContext) -> BindGroupLayout {
        context
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &self.entries,
            })
    }
}
