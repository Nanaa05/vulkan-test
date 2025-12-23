use crate::engine::camera::Camera;
use crate::engine::camera_rig::{CameraRig, CameraTargetMode};
use glam::Vec3;

pub struct CameraSystem;

impl CameraSystem {
    pub fn update(camera: &mut Camera, rig: &mut CameraRig, character_pos: Vec3) {
        rig.target = match rig.mode {
            CameraTargetMode::Origin => Vec3::ZERO,
            CameraTargetMode::FollowCharacter => character_pos,
        };

        rig.pitch = rig.pitch.clamp(-89.0, 89.0);

        let yaw = rig.yaw.to_radians();
        let pitch = rig.pitch.to_radians();

        let front = Vec3::new(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        )
        .normalize();

        camera.yaw = rig.yaw;
        camera.pitch = rig.pitch;
        camera.target = rig.target;
        camera.pos = rig.target - front * rig.radius;
    }
}
