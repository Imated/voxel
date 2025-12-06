use crate::rendering::renderer::Renderer;
use crate::rendering::wgpu_context::WGPUContext;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Buffer,
    TextureView,
};

pub struct BindGroupBuilder<'a> {
    entries: Vec<BindGroupEntry<'a>>,
}

impl<'a> BindGroupBuilder<'a> {
    pub fn new() -> Self {
        Self { entries: vec![] }
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

    pub fn with_buffer(mut self, buffer: &'a Buffer) -> Self {
        self.entries.push(BindGroupEntry {
            binding: self.entries.len() as u32,
            resource: BindingResource::Buffer(buffer.as_entire_buffer_binding()),
        });

        self
    }

    pub fn build(self, context: &WGPUContext, layout: &BindGroupLayout) -> BindGroup {
        context.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout,
            entries: &self.entries,
        })
    }
}
