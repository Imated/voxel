mod camera_controller;
mod macros;
mod rendering;
mod cubes;

use crate::camera_controller::CameraController;
use crate::rendering::material::Material;
use crate::rendering::renderer::Renderer;
use crate::rendering::shader::Shader;
use crate::rendering::texture::Texture;
use crate::rendering::utils::bind_group_builder::BindGroupBuilder;
use crate::rendering::utils::bind_group_layout_builder::BindGroupLayoutBuilder;
use log::*;
use std::process::abort;
use std::sync::Arc;
use std::time::{Duration, Instant};
use wgpu::{ShaderStages, SurfaceError};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{DeviceId, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};
use crate::cubes::Cubes;

struct App {
    last_frame_time: Instant,
    frame_count: u64,
    cam_controller: CameraController,

    renderer: Option<Renderer>,

    atlas: Option<Texture>,
    default_opaque_shader: Option<Shader>,
    default_opaque: Option<Material>,

    cubes: Option<Cubes>,
}

impl App {
    pub fn new(_event_loop: &EventLoop<()>) -> Self {
        Self {
            last_frame_time: Instant::now(),
            frame_count: 0,
            cam_controller: CameraController::new(0.002),
            renderer: None,
            atlas: None,
            default_opaque_shader: None,
            default_opaque: None,
            cubes: None,
        }
    }
}

impl App {
    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        self.cam_controller.handle_key(code, is_pressed);
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
        let renderer = self.renderer.as_mut().unwrap();

        let atlas = renderer
            .create_texture("/res/textures/atlas.png")
            .unwrap_or_else(|err| {
                fatal!("Failed to load texture atlas: {}", err);
            });

        self.atlas = Some(atlas);

        let default_shader_layout = BindGroupLayoutBuilder::new()
            .with_texture2d(ShaderStages::FRAGMENT)
            .with_sampler(ShaderStages::FRAGMENT)
            .build(renderer.context());

        let default_shader = renderer
            .create_shader("/res/shaders/default.wgsl", default_shader_layout)
            .unwrap_or_else(|err| {
                fatal!("Failed to load default shader: {}", err);
            });

        self.default_opaque_shader = Some(default_shader);

        let default_material_bind_group = BindGroupBuilder::new()
            .with_texture2d(&self.atlas.as_ref().unwrap().view)
            .with_sampler(renderer)
            .build(
                renderer.context(),
                &self.default_opaque_shader.as_ref().unwrap().material_layout,
            );

        let default_opaque = Material {
            shader: self.default_opaque_shader.as_ref().unwrap().clone(),
            bind_group: default_material_bind_group,
        };

        self.cubes = Some(Cubes::new(self.renderer.as_ref().unwrap(), &default_opaque));

        self.default_opaque = Some(default_opaque);

        Ok(())
    }

    pub fn render(&mut self) {
        let renderer = self.renderer.as_mut().unwrap();

        self.cam_controller.update_camera(&mut renderer.camera);
        renderer.update_scene_data();

        let mut cubes = self.cubes.as_mut().unwrap();
        cubes.render(renderer);

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
        window.set_cursor_grab(CursorGrabMode::Confined).unwrap_or_else(|_| error!("Failed to set cursor grab mode!"));
        window.set_cursor_visible(false);
        let renderer = pollster::block_on(Renderer::new(window))
            .unwrap_or_else(|err| fatal!("Failed to create renderer! Error: {:?}", err));
        self.renderer = Some(renderer);

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
                let renderer = self.renderer.as_mut().unwrap();
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
