use crate::{engine::camera::Camera, renderer::render_types::RenderItem};

use super::{
    mesh_store::{MeshId, MeshStore},
    transform::Transform,
};

pub struct Object {
    pub mesh: MeshId,
    pub transform: Transform,
}

pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<Object>,
    pub character: usize, // index into objects
}

impl Scene {
    pub fn character_mut(&mut self) -> &mut Object {
        &mut self.objects[self.character]
    }

    pub fn render_items<'a>(&'a self, meshes: &'a MeshStore) -> Vec<RenderItem<'a>> {
        self.objects
            .iter()
            .map(|obj| RenderItem {
                mesh: meshes.get(obj.mesh),
                model: obj.transform.model_matrix(),
            })
            .collect()
    }
}
