mod macros;

use log::*;
use std::process::abort;
use std::rc::Rc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

pub struct State {
    window: Rc<Window>,
}

impl State {
    pub async fn new(window: Rc<Window>) -> anyhow::Result<Self> {
        Ok(Self { window })
    }

    pub fn resize(&mut self, width: u32, height: u32) {

    }

    pub fn render(&mut self) {
        self.window.request_redraw();
    }
}

struct App {
    state: Option<State>,
}

impl App {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        Self {
            state: None,
        }
    }
}

impl ApplicationHandler<> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Voxel Engine")
            .with_inner_size(LogicalSize::new(800, 600))
            .with_resizable(true);
        let window = Rc::new(
            event_loop
                .create_window(window_attributes)
                .unwrap_or_else(|err| fatal!("Failed to create window! Error: {:?}", err)),
        );
        let state = pollster::block_on(State::new(window))
            .unwrap_or_else(|err| fatal!("Failed to initialize WGPU! Error: {:?}", err));
        self.state = Some(state);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(state) => state,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => state.render(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => match (code, key_state.is_pressed()) {
                (KeyCode::Escape, true) => event_loop.exit(),
                _ => {}
            },
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
