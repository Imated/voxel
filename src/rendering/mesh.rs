use wgpu::Buffer;
use wgpu::naga::FastHashMap;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct MeshId(pub u32);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mesh {
    pub(crate) vertices: Buffer,
    pub(crate) indices: Buffer,
    pub(crate) num_indices: u32,
    pub(crate) start_index: u32,
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

    pub fn get(&self, id: u32) -> Option<&Mesh> {
        self.meshes.get(&MeshId(id))
    }
}
