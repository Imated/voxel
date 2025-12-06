use crate::rendering::camera::Camera;
use glam::{IVec2, Vec2};
use winit::keyboard::KeyCode;

pub struct CameraController {
    speed: f32,
    direction: IVec2,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            direction: IVec2::ZERO,
        }
    }

    pub fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.direction.y = i32::from(is_pressed);
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.direction.x = -i32::from(is_pressed);
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.direction.y = -i32::from(is_pressed);
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.direction.x = i32::from(is_pressed);
                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        let forward = camera.target - camera.eye;
        let forward_normalized = forward.normalize();
        let forward_magnitude = forward.length();

        if forward_magnitude > self.speed {
            camera.eye += forward_normalized * self.speed * self.direction.y as f32;
        }

        let right = forward.cross(camera.up);
        let forward = camera.target - camera.eye;
        let forward_magnitude = forward.length();

        camera.eye = camera.target
            - (forward + right * self.speed * self.direction.x as f32).normalize()
                * forward_magnitude;
    }
}
