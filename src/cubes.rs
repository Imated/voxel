use crate::rendering::material::Material;
use crate::rendering::render_object::{PassType, RenderObject};
use crate::rendering::renderer::Renderer;
use glam::{Mat4, Quat, Vec3};
use std::time::Instant;
use bytemuck::{Pod, Zeroable};
use wgpu::{vertex_attr_array, BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};
use crate::rendering::buffer::Buffer;
use crate::rendering::mesh::Mesh;
use crate::rendering::vertex::Vertex;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct CubeData {
    pub model: Mat4,
}

impl CubeData {
    pub(crate) fn desc() -> VertexBufferLayout<'static> {
        const ATTRIBS: [VertexAttribute; 4] =
            vertex_attr_array![5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4];

        VertexBufferLayout {
            array_stride: size_of::<CubeData>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &ATTRIBS,
        }
    }
}

pub struct Cubes {
    render_object: RenderObject,
    instance_buffer: Buffer<CubeData>, // gpu side
    instance_data: Vec<CubeData>, // cpu side
    start_time: Instant,
}

impl Cubes {
    const NUM_INSTANCES_PER_ROW: u32 = 10;
    const INSTANCE_DISPLACEMENT: Vec3 = Vec3::new(
        Self::NUM_INSTANCES_PER_ROW as f32 * 0.5,
        0.0,
        Self::NUM_INSTANCES_PER_ROW as f32 * 0.5,
    );

    const TRIANGLE_VERTICES: [Vertex; 6] = [
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

    const TRIANGLE_INDICES: [u16; 9] = [0, 4, 5, 1, 3, 4, 2, 5, 3];

    pub fn new(renderer: &Renderer, material: &Material) -> Self {
        let instances: Vec<CubeData> = (0..Self::NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..Self::NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position = Vec3::new(x as f32, 0.0, z as f32) - Self::INSTANCE_DISPLACEMENT;
                    let rotation = Quat::from_axis_angle(Vec3::Z, 0f32.to_degrees());
                    let model = Mat4::from_translation(position) * Mat4::from_quat(rotation);

                    CubeData { model }
                })
            })
            .collect();

        let mesh = Mesh {
            vertices: Buffer::new_vertex(&renderer.context(), Some(&Self::TRIANGLE_VERTICES)),
            indices: Buffer::new_index(&renderer.context(), Some(&Self::TRIANGLE_INDICES)),
            num_indices: Self::TRIANGLE_INDICES.len() as u32,
            start_index: 0,
        };

        let instance_buffer = Buffer::new_instance(&renderer.context(), Some(&instances));

        Self {
            render_object: RenderObject {
                mesh,
                material: material.clone(),
                pass: PassType::Opaque,
                instances: instance_buffer.buffer().clone(),
                instances_len: instance_buffer.len(),
            },
            instance_buffer,
            instance_data: instances,
            start_time: Instant::now(),
        }
    }

    pub fn render(&mut self, renderer: &mut Renderer) {
        let dt = (Instant::now() - self.start_time).as_secs_f32();
        self.start_time = Instant::now();
        let rotation = Quat::from_axis_angle(Vec3::Y, dt * 2f32);
        for instance in &mut self.instance_data {
            instance.model *= Mat4::from_quat(rotation);
        }

        self.instance_buffer.upload(&renderer.context(), &self.instance_data);

        renderer.push_object(&self.render_object);
    }
}
