use bytemuck::{Pod, Zeroable};
use wgpu::{BindGroup, BindGroupLayout, BufferBindingType, ShaderStages};
use wgpu::AddressMode::ClampToEdge;
use wgpu::FilterMode::{Linear, Nearest};
use crate::rendering::buffer::Buffer;
use crate::rendering::camera::{Camera, CameraBufferContext};
use crate::rendering::utils::bind_group_builder::BindGroupBuilder;
use crate::rendering::utils::bind_group_layout_builder::BindGroupLayoutBuilder;
use crate::rendering::utils::sampler_builder::SamplerBuilder;
use crate::rendering::wgpu_context::WGPUContext;

// inspired by https://github.com/Wumpf/blub/blob/master/src/global_bindings.rs
pub struct GlobalBindings {
    layout: BindGroupLayout,
    bind_group: BindGroup,
    global_buffer: Buffer<GlobalBufferContext>,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GlobalBufferContext {
    camera: CameraBufferContext
}

impl GlobalBufferContext {
    pub fn new(camera: &Camera) -> Self {
        Self {
            camera: camera.fill_buffer_context(),
        }
    }

    pub fn fill(&mut self, camera: &Camera) {
        self.camera = camera.fill_buffer_context();
    }
}

impl GlobalBindings {
    pub fn new(context: &WGPUContext, global_data: GlobalBufferContext) -> Self {
        let layout = BindGroupLayoutBuilder::new()
            .with_buffer(ShaderStages::VERTEX_FRAGMENT, BufferBindingType::Uniform)
            .with_sampler(ShaderStages::FRAGMENT)
            .with_sampler(ShaderStages::FRAGMENT)
            .build(context, Some("Global Bind Group Layout"));

        let trilinear_sampler = SamplerBuilder::new()
            .with_mode(ClampToEdge)
            .with_filtering(Linear)
            .build(context, Some("Global Trilinear Sampler"));

        let point_sampler = SamplerBuilder::new()
            .with_mode(ClampToEdge)
            .with_filtering(Nearest)
            .build(context, Some("Global Point Sampler"));

        let global_buffer = Buffer::new_uniform(context, Some(&[global_data]));

        let bind_group = BindGroupBuilder::new()
                .with_buffer(&global_buffer.buffer())
                .with_sampler(&trilinear_sampler)
                .with_sampler(&point_sampler)
                .build(context, &layout, Some("Global Bind Group"));

        Self {
            layout,
            bind_group,
            global_buffer,
        }
    }

    pub fn update_global_buffer(&mut self, context: &WGPUContext, global_data: GlobalBufferContext) {
        self.global_buffer.upload(context, vec![global_data]);
    }

    pub fn global_buffer(&mut self) -> GlobalBufferContext {
        self.global_buffer.content()[0]
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.layout
    }
}