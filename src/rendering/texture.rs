use wgpu::{Extent3d, TextureView};

pub struct Texture {
    pub size: Extent3d,
    pub(crate) data: wgpu::Texture,
    pub view: TextureView,
}
