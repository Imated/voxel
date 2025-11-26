use wgpu::naga::FastHashMap;
use wgpu::Buffer;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct MeshId(pub u32);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mesh {
    vertices: Buffer,
    indices: Buffer,
    num_indices: u32,
}

impl Mesh {
    pub fn new(vertices: Buffer, indices: Buffer, num_indices: u32) -> Self {
        Self {
            vertices,
            indices,
            num_indices,
        }
    }
}

pub struct Meshes {
    pub meshes: FastHashMap<MeshId, Mesh>,
}

impl Meshes {
    pub fn new() -> Self {
        Self {
            meshes: FastHashMap::default(),
        }
    }

    pub fn add(&mut self, id: u32, mesh: Mesh) {
        self.meshes.insert(MeshId(id), mesh);
    }

    pub fn get(&self, id: u32) -> Option<&Mesh>  {
        self.meshes.get(&MeshId(id))
    }
}