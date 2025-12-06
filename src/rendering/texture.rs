use image::ImageReader;
use wgpu::naga::FastHashMap;
use wgpu::wgt::{TextureDescriptor, TextureViewDescriptor};
use wgpu::{
    Device, Extent3d, Origin3d, Queue, TexelCopyBufferLayout, TexelCopyTextureInfo, TextureAspect,
    TextureDimension, TextureFormat, TextureUsages, TextureView,
};
use crate::rendering::wgpu_context::WGPUContext;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct TextureId(pub u32);

pub struct Textures {
    pub textures: FastHashMap<TextureId, Texture>,
}

impl Textures {
    pub fn new() -> Self {
        Self {
            textures: FastHashMap::default(),
        }
    }

    pub fn add(&mut self, id: u32, texture: Texture) {
        self.textures.insert(TextureId(id), texture);
    }

    pub fn get(&self, id: u32) -> Option<&Texture> {
        self.textures.get(&TextureId(id))
    }
}

pub struct Texture {
    pub size: Extent3d,
    pub(crate) data: wgpu::Texture,
    pub view: TextureView,
}
