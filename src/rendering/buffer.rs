use crate::rendering::wgpu_context::WGPUContext;
use bytemuck::{cast_slice, Pod, Zeroable};
use std::any::type_name;
use std::marker::PhantomData;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BindingResource, BufferUsages};

// A generic wgpu buffer implementation.
// Inspired by https://github.com/Wumpf/blub/blob/master/src/wgpu_utils/uniformbuffer.rs.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Buffer<Content>
where
    Content: Pod + Zeroable,
{
    buffer: wgpu::Buffer,
    content: Vec<Content>,
    len: u32,
}
impl<Content> Buffer<Content>
where
    Content: Pod + Zeroable,
{
    fn name() -> &'static str {
        let type_name = type_name::<Content>();
        let pos = type_name.rfind(':').unwrap_or_default();
        &type_name[(pos + 1)..]
    }

    pub fn new(context: &WGPUContext, content: Option<&[Content]>, usage: BufferUsages) -> Self {
        let contents = match content {
            None => &[],
            Some(content) => content,
        };

        let buffer = context.device.create_buffer_init(&BufferInitDescriptor {
            label: Some(&format!("Uniform Buffer: {:?}", Self::name())),
            contents: cast_slice(contents),
            usage,
        });

        Self {
            buffer,
            content: Vec::from(contents),
            len: contents.len() as u32,
        }
    }

    pub fn new_uniform(context: &WGPUContext, content: Option<&[Content]>) -> Self {
        Self::new(context, content, BufferUsages::UNIFORM | BufferUsages::COPY_DST)
    }

    pub fn new_instance(context: &WGPUContext, content: Option<&[Content]>) -> Self {
        Self::new(context, content, BufferUsages::VERTEX | BufferUsages::COPY_DST)
    }

    pub fn new_vertex(context: &WGPUContext, content: Option<&[Content]>) -> Self {
        Self::new(context, content, BufferUsages::VERTEX)
    }

    pub fn new_index(context: &WGPUContext, content: Option<&[Content]>) -> Self {
        Self::new(context, content, BufferUsages::INDEX)
    }

    pub fn upload(&mut self, context: &WGPUContext, content: Vec<Content>) {
        self.content = content;
        context.queue.write_buffer(&self.buffer, 0, cast_slice(&self.content));
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn binding_resource(&self) -> BindingResource {
        self.buffer.as_entire_binding()
    }

    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn content(&mut self) -> &mut [Content] {
        &mut self.content
    }
}
