use glam::Vec3;

#[derive(Debug, Clone, Copy)]
pub enum CameraTargetMode {
    Origin,
    FollowCharacter,
}

#[derive(Debug, Clone)]
pub struct CameraRig {
    pub yaw: f32,
    pub pitch: f32,
    pub radius: f32,
    pub mode: CameraTargetMode,
    pub target: Vec3, // computed each frame (origin or character pos)
}

impl CameraRig {
    pub fn new(radius: f32) -> Self {
        Self {
            yaw: -90.0,
            pitch: 0.0,
            radius,
            mode: CameraTargetMode::FollowCharacter,
            target: Vec3::ZERO,
        }
    }
}
