use crate::rendering::wgpu_context::WGPUContext;
use wgpu::{AddressMode, FilterMode, Label, Sampler, SamplerDescriptor};

pub struct SamplerBuilder<'a> {
    desc: SamplerDescriptor<'a>,
}

impl<'a> SamplerBuilder<'a> {
    pub fn new() -> Self {
        Self {
            desc: Default::default(),
        }
    }

    pub fn with_mode(mut self, mode: AddressMode) -> Self {
        self.desc.address_mode_u = mode;
        self.desc.address_mode_v = mode;
        self.desc.address_mode_w = mode;

        self
    }

    pub fn with_filtering(mut self, mode: FilterMode) -> Self {
        self.desc.min_filter = mode;
        self.desc.mag_filter = mode;
        self.desc.mipmap_filter = mode;

        self
    }

    pub fn build(mut self, context: &WGPUContext, label: Label<'a>) -> Sampler {
        self.desc.label = label;
        context.device.create_sampler(&self.desc)
    }
}
