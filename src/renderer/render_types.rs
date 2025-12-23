use glam::Mat4;

use super::mesh::Mesh;

pub struct FrameGlobals {
    pub view_proj: Mat4,
}

pub struct RenderItem<'a> {
    pub mesh: &'a Mesh,
    pub model: Mat4,
}
