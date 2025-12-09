use std::time::Instant;
use crate::rendering::material::Material;
use crate::rendering::render_object::{PassType, RenderObject};
use crate::rendering::renderer::Renderer;
use crate::rendering::vertex::{InstanceData, Vertex};
use glam::{Mat4, Quat, Vec3};

pub struct Cubes {
    render_object: RenderObject,
    instances: Vec<InstanceData>,
    start_time: Instant,
}

impl Cubes {
    const NUM_INSTANCES_PER_ROW: u32 = 10;
    const INSTANCE_DISPLACEMENT: Vec3 = Vec3::new(Self::NUM_INSTANCES_PER_ROW as f32 * 0.5, 0.0, Self::NUM_INSTANCES_PER_ROW as f32 * 0.5);

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
        let instances: Vec<InstanceData> = (0..Self::NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (0..Self::NUM_INSTANCES_PER_ROW).map(move |x| {
                let position = Vec3::new(x as f32, 0.0, z as f32) - Self::INSTANCE_DISPLACEMENT;
                let rotation = Quat::from_axis_angle(Vec3::Z, 0f32.to_degrees());
                let model = Mat4::from_translation(position) * Mat4::from_quat(rotation);

                InstanceData {
                    model
                }
            })
        }).collect();

        let mesh = renderer.create_mesh(&Self::TRIANGLE_VERTICES, &Self::TRIANGLE_INDICES, 0);
        let instance_buffer = renderer.create_instance_buffer(&instances);

        Self {
            render_object: RenderObject {
                mesh,
                material: material.clone(),
                pass: PassType::Opaque,
                instances: instance_buffer,
            },
            instances,
            start_time: Instant::now(),
        }
    }

    pub fn render(&mut self, renderer: &mut Renderer) {
        let time = Instant::now() - self.start_time;
        self.start_time = Instant::now();
        let rotation = Quat::from_axis_angle(Vec3::Y, time.as_secs_f32().to_degrees() * 0.01f32);
        self.instances = self.instances.iter().map(|x| {
            InstanceData {
                model: x.model * Mat4::from_quat(rotation),
            }
        }).collect();

        self.update_instances(renderer);

        renderer.push_object(&self.render_object);
    }

    fn update_instances(&self, renderer: &Renderer) {
        renderer.update_instance_buffer_with(&self.render_object.instances, &self.instances);
    }
}