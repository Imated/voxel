mod camera_controller;
mod macros;
mod rendering;

use crate::camera_controller::CameraController;
use crate::rendering::material::Material;
use crate::rendering::mesh::Mesh;
use crate::rendering::render_object::PassType::Opaque;
use crate::rendering::render_object::RenderObject;
use crate::rendering::renderer::Renderer;
use crate::rendering::shader::Shader;
use crate::rendering::texture::Texture;
use crate::rendering::utils::bind_group_builder::BindGroupBuilder;
use crate::rendering::utils::bind_group_layout_builder::BindGroupLayoutBuilder;
use crate::rendering::vertex::{InstanceData, Vertex};
use log::*;
use std::process::abort;
use std::sync::Arc;
use std::time::{Duration, Instant};
use glam::{Mat4, Quat, Vec3, Vec4};
use wgpu::{BufferUsages, ShaderStages, SurfaceError};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{DeviceId, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

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

struct App {
    last_frame_time: Instant,
    frame_count: u64,
    cam_controller: CameraController,

    renderer: Option<Renderer>,

    atlas: Option<Texture>,
    default_opaque_shader: Option<Shader>,
    default_opaque: Option<Material>,
    test_mesh: Option<Mesh>,
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
            test_mesh: None,
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

        self.default_opaque = Some(default_opaque);

        let mesh = renderer.create_mesh(TRIANGLE_VERTICES, TRIANGLE_INDICES, 0);
        self.test_mesh = Some(mesh);

        Ok(())
    }

    pub fn render(&mut self) {
        let renderer = self.renderer.as_mut().unwrap();

        self.cam_controller.update_camera(&mut renderer.camera);
        renderer.update_scene_data();

        const NUM_INSTANCES_PER_ROW: u32 = 10;
        const INSTANCE_DISPLACEMENT: Vec3 = Vec3::new(NUM_INSTANCES_PER_ROW as f32 * 0.5, 0.0, NUM_INSTANCES_PER_ROW as f32 * 0.5);

        let instances: Vec<InstanceData> = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let position = Vec3::new(x as f32, 0.0, z as f32);
                let rotation = Quat::from_axis_angle(Vec3::Z, 45f32.to_degrees());
                let model = Mat4::from_translation(position) * Mat4::from_quat(rotation);

                InstanceData {
                    model
                }
            })
        }).collect();

        renderer.push_object(RenderObject {
            mesh: self.test_mesh.as_ref().unwrap().clone(),
            material: self.default_opaque.as_ref().unwrap().clone(),
            model_bind_group: None,
            pass: Opaque,
            instances: renderer.context().create_buffer(&instances, BufferUsages::VERTEX),
            num_instances: instances.len() as u32,
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
