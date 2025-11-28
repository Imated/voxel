use crate::abort;
use crate::rendering::renderer::Renderer;
use crate::rendering::shader::{Shader, Shaders};
use crate::{ShaderType, fatal};
use legion::{Schedule, system, World, Resources};
use wgpu::{
    BindGroupLayoutEntry, BindingType, ShaderStages, TextureSampleType, TextureViewDimension,
};
use crate::rendering::texture::Textures;
use crate::TextureType::Atlas;

pub fn launch_startup_systems(mut world: &mut World, mut resources: &mut Resources) {
    let mut startup_schedule = Schedule::builder()
        .add_system(load_shaders_system())
        .add_system(load_textures_system())
        .build();

    startup_schedule.execute(&mut world, &mut resources);
}

#[system]
fn load_shaders(#[resource] shaders: &mut Shaders, #[resource] renderer: &mut Renderer) {
    let default_shader_layout = renderer.create_shader_layout(vec![BindGroupLayoutEntry {
        binding: 0,
        visibility: ShaderStages::FRAGMENT,
        ty: BindingType::Texture {
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }]);

    let default_shader = renderer
        .create_shader("/res/shaders/default.wgsl", default_shader_layout)
        .unwrap_or_else(|err| {
            fatal!("Failed to load default shader: {}", err);
        });
    shaders.add(ShaderType::Opaque as u32, default_shader);
}

#[system]
fn load_textures(#[resource] textures: &mut Textures, #[resource] renderer: &mut Renderer) {
    let atlas = renderer
        .load_texture("/res/textures/atlas.png")
        .unwrap_or_else(|err| {
            fatal!("Failed to load texture atlas: {}", err);
        });
    textures.add(Atlas as u32, atlas);
}