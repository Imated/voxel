use wgpu::{BindGroupLayout, BindGroupLayoutEntry, BindingType, SamplerBindingType, ShaderStages, TextureSampleType, TextureViewDimension};
use crate::rendering::renderer::Renderer;

pub struct BindGroupLayoutBuilder {
    pub(crate) entries: Vec<BindGroupLayoutEntry>,
}

impl BindGroupLayoutBuilder {
    pub fn new() -> Self {
        Self {
            entries: vec![],
        }
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

    pub fn build(self, renderer: &Renderer) -> BindGroupLayout {
        renderer.create_bind_group_layout(self.entries)
    }
}