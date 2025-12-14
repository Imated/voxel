use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Vec4};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols(
    Vec4::new(1.0, 0.0, 0.0, 0.0),
    Vec4::new(0.0, 1.0, 0.0, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 1.0),
);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct CameraBufferContext {
    view_proj: Mat4,
}

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub aspect: f32,
    pub fov: f32,
    pub near_clip: f32,
    pub far_clip: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = Mat4::perspective_rh(self.fov, self.aspect, self.near_clip, self.far_clip);

        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    pub fn fill_buffer_context(&self) -> CameraBufferContext {
        CameraBufferContext {
            view_proj: self.build_view_projection_matrix(),
        }
    }
}
