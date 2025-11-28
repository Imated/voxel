mod macros;
mod rendering;
mod startup;

use crate::TextureType::Atlas;
use crate::rendering::material::{Material, Materials};
use crate::rendering::mesh::Meshes;
use crate::rendering::render_object::RenderObject;
use crate::rendering::renderer::Renderer;
use crate::rendering::shader::Shaders;
use crate::rendering::texture::Textures;
use crate::rendering::vertex::Vertex;
use legion::{Resources, World};
use log::*;
use std::process::abort;
use std::sync::Arc;
use std::time::{Duration, Instant};
use wgpu::{
    BindGroupEntry, BindGroupLayoutEntry, BindingResource, BindingType, ShaderStages, SurfaceError,
    TextureSampleType, TextureViewDimension,
};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{DeviceId, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};
use crate::rendering::bind_groups::bind_group_builder::BindGroupBuilder;
use crate::startup::launch_startup_systems;

const TRIANGLE_VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.625, 0.0],
        tex_coords: [1.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [0.0, -0.5, 0.0],
        tex_coords: [0.0, 0.5],
    },
    Vertex {
        position: [-0.25, 0.125, 0.0],
        tex_coords: [0.5, 0.5],
    },
    Vertex {
        position: [0.25, 0.125, 0.0],
        tex_coords: [0.0, 0.5],
    },
];

const TRIANGLE_INDICES: &[u16] = &[0, 4, 5, 1, 3, 4, 2, 5, 3];

#[repr(u32)]
pub enum ShaderType {
    Opaque = 0,
}

#[repr(u32)]
pub enum MeshType {
    Triangle = 0,
}

#[repr(u32)]
pub enum MaterialType {
    BlockOpaque = 0,
}

#[repr(u32)]
pub enum TextureType {
    Atlas = 0,
}

struct App {
    resources: Resources,
    world: World,
    last_frame_time: Instant,
    frame_count: u64,
}

impl App {
    pub fn new(_event_loop: &EventLoop<()>) -> Self {
        Self {
            resources: Resources::default(),
            world: World::default(),
            last_frame_time: Instant::now(),
            frame_count: 0,
        }
    }
}

impl App {
    pub fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }

    pub fn handle_mouse_moved(
        &self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        _position: PhysicalPosition<f64>,
    ) {
    }

    pub fn load_assets(&mut self) -> anyhow::Result<()> {
        let shaders = self.resources.get_mut::<Shaders>().unwrap();
        let mut materials = self.resources.get_mut::<Materials>().unwrap();
        let textures = self.resources.get_mut::<Textures>().unwrap();
        let mut meshes = self.resources.get_mut::<Meshes>().unwrap();
        let renderer = self.resources.get::<Renderer>().unwrap();

        let default_shader = shaders.get(ShaderType::Opaque as u32).unwrap();
        let default_material_bind_group = BindGroupBuilder::new()
            .with_texture2d(&&textures.get(Atlas as u32).unwrap().view).
            with_sampler(&renderer)
            .build(&renderer, &shaders.get(ShaderType::Opaque as u32).unwrap().bind_group_layout);
        materials.add(MaterialType::BlockOpaque as u32, Material {
            shader: default_shader.clone(),
            bind_group: default_material_bind_group,
        });

        let mesh = renderer.create_mesh::<Vertex, u16>(TRIANGLE_VERTICES, TRIANGLE_INDICES);
        meshes.add(MeshType::Triangle as u32, mesh);

        Ok(())
    }

    pub fn render(&mut self) {
        let mut renderer = self.resources.get_mut::<Renderer>().unwrap();
        let meshes = self.resources.get::<Meshes>().unwrap();
        let materials = self.resources.get::<Materials>().unwrap();

        renderer.push_object(RenderObject {
            mesh: meshes.get(MeshType::Triangle as u32).unwrap().clone(),
            material: materials
                .get(MaterialType::BlockOpaque as u32)
                .unwrap()
                .clone(),
            model_bind_group: None,
            transparent: false,
        });

        match renderer.render() {
            Ok(_) => {}
            Err(SurfaceError::Lost) => {}
            Err(SurfaceError::Outdated) => {}
            Err(SurfaceError::OutOfMemory) => {
                fatal!("Out of memory!!");
            }
            Err(SurfaceError::Timeout) => {
                warn!("Surface timed out!");
            }
            Err(err) => error!("Failed to render! Error: {:?}", err),
        }

        self.frame_count += 1;
        let elapsed = self.last_frame_time.elapsed();
        if elapsed >= Duration::from_secs(1) {
            let fps = self.frame_count as f32 / elapsed.as_secs_f32();
            println!("FPS: {:.1}", fps);

            self.frame_count = 0;
            self.last_frame_time = Instant::now();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Voxel Engine")
            .with_inner_size(LogicalSize::new(800, 600))
            .with_resizable(true);
        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .unwrap_or_else(|err| fatal!("Failed to create window! Error: {:?}", err)),
        );
        let renderer = pollster::block_on(Renderer::new(window))
            .unwrap_or_else(|err| fatal!("Failed to create renderer! Error: {:?}", err));

        self.resources.insert(renderer);
        self.resources.insert(Meshes::new());
        self.resources.insert(Materials::new());
        self.resources.insert(Shaders::new());
        self.resources.insert(Textures::new());

        launch_startup_systems(&mut self.world, &mut self.resources);

        self.load_assets()
            .unwrap_or_else(|err| fatal!("Failed to load assets! Error: {:?}", err));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                let mut renderer = self.resources.get_mut::<Renderer>().unwrap();
                renderer.resize(size.width, size.height);
            }
            WindowEvent::RedrawRequested => self.render(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => self.handle_key(event_loop, code, key_state.is_pressed()),
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => self.handle_mouse_moved(event_loop, device_id, position),
            _ => {}
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    pretty_env_logger::init();
    let event_loop = EventLoop::new()?;
    let mut app = App::new(&event_loop);
    event_loop.run_app(&mut app)?;

    Ok(())
}

fn main() {
    run().unwrap_or_else(|err| fatal!("Failed to run application! Error: {:?}", err))
}
