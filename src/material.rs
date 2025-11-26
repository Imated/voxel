use crate::shader::Shader;
use wgpu::naga::FastHashMap;
use wgpu::BindGroup;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct MaterialId(pub u32);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Material {
    pub shader: Shader,
    pub bind_group: BindGroup,
}

pub struct Materials {
    pub materials: FastHashMap<MaterialId, Material>,
}

impl Materials {
    pub fn new() -> Self {
        Self {
            materials: FastHashMap::default(),
        }
    }

    pub fn add(&mut self, id: u32, mat: Material)  {
        self.materials.insert(MaterialId(id), mat);
    }

    pub fn get(&self, id: u32) -> Option<&Material>  {
        self.materials.get(&MaterialId(id))
    }
}