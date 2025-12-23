use crate::renderer::mesh::Mesh;
use crate::{assets::mesh::MeshData, core::device::Device, renderer::renderer::Renderer};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeshId(pub usize);

pub struct MeshStore {
    meshes: Vec<Mesh>,
}

impl MeshStore {
    pub fn new() -> Self {
        Self { meshes: Vec::new() }
    }

    pub fn add(&mut self, mesh: Mesh) -> MeshId {
        let id = MeshId(self.meshes.len());
        self.meshes.push(mesh);
        id
    }

    pub fn get(&self, id: MeshId) -> &Mesh {
        &self.meshes[id.0]
    }

    pub fn destroy_all(&mut self, device: &ash::Device) {
        for m in self.meshes.drain(..) {
            m.destroy(device);
        }
    }
}

impl MeshStore {
    pub fn upload(
        &mut self,
        renderer: &Renderer,
        dev: &Device,
        mesh: &MeshData,
    ) -> anyhow::Result<MeshId> {
        let gpu = renderer.upload_mesh(dev, mesh)?;
        Ok(self.add(gpu))
    }
}
