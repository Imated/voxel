use image::{GenericImageView, ImageReader};
use wgpu::naga::FastHashMap;
use wgpu::wgt::{TextureDescriptor, TextureViewDescriptor};
use wgpu::{
    Device, Extent3d, Origin3d, Queue, TexelCopyBufferLayout, TexelCopyTextureInfo, TextureAspect,
    TextureDimension, TextureFormat, TextureUsages, TextureView,
};
use crate::shader::{Shader, ShaderId};

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
    data: wgpu::Texture,
    pub view: TextureView,
}

impl Texture {
    pub fn new(device: &Device, queue: &Queue, path: &str) -> anyhow::Result<Self> {
        let image = ImageReader::open(env!("CARGO_MANIFEST_DIR").to_owned() + path)?.decode()?;
        let image_rgba = image.to_rgba8();

        use image::GenericImageView;
        let dimensions = image.dimensions();

        let size = Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1, // 1 layer for 2d texture
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: Some(path),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST, // texture_binding = use in shaders, copy_dst = copy data to texture
            view_formats: &[],
        });

        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &image_rgba,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&TextureViewDescriptor::default());

        Ok(Self {
            size,
            data: texture,
            view,
        })
    }
}
