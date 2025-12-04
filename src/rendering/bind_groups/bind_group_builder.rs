use crate::rendering::renderer::Renderer;
use wgpu::{BindGroup, BindGroupEntry, BindGroupLayout, BindingResource, TextureView};

pub struct BindGroupBuilder<'a> {
    pub(crate) entries: Vec<BindGroupEntry<'a>>,
}

impl<'a> BindGroupBuilder<'a> {
    pub fn new() -> Self {
        Self {
            entries: vec![],
        }
    }

    pub fn with_texture2d(mut self, view: &'a TextureView) -> Self {
        self.entries.push(BindGroupEntry {
            binding: self.entries.len() as u32,
            resource: BindingResource::TextureView(view),
        });

        self
    }

    pub fn with_sampler(mut self, renderer: &'a Renderer) -> Self {
        self.entries.push(BindGroupEntry {
            binding: self.entries.len() as u32,
            resource: BindingResource::Sampler(renderer.universal_sampler()),
        });

        self
    }

    pub fn build(self, renderer: &'a Renderer, layout: &BindGroupLayout) -> BindGroup {
        renderer.create_bind_group(self.entries, layout)
    }
}