mod macros;
mod renderer;
mod shader;
mod main_pass;
mod vertex;

use crate::renderer::Renderer;
use log::*;
use std::process::abort;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{DeviceId, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

struct App {
    renderer: Option<Renderer>,
}

impl App {
    pub fn new(_event_loop: &EventLoop<()>) -> Self {
        Self { renderer: None }
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
            .unwrap_or_else(|err| fatal!("Failed to initialize WGPU! Error: {:?}", err));
        self.renderer = Some(renderer);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let renderer = match &mut self.renderer {
            Some(renderer) => renderer,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => renderer.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                renderer
                    .render()
                    .unwrap_or_else(|err| error!("Failed to render! Error: {:?}", err));
            }
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
