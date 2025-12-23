use anyhow::Result;
use glfw::Key;

use crate::assets::mesh;
use crate::engine::camera::Camera;
use crate::engine::camera_rig::{CameraRig, CameraTargetMode};
use crate::engine::camera_system::CameraSystem;
use crate::engine::engine::Engine;
use crate::engine::game_loop::GameLoop;

use crate::game::character_controller::CharacterControllerSystem;
use crate::renderer::render_types::FrameGlobals;
use crate::scene::{
    mesh_store::MeshStore,
    scene::{Object, Scene},
    transform::Transform,
};

pub struct Game {
    pub scene: Scene,
    pub meshes: MeshStore,
    pub rig: CameraRig,
    pub motor: crate::game::character_controller::CharacterMotor,
}

impl Game {
    pub fn new(engine: &mut Engine) -> Result<Self> {
        let mut meshes = MeshStore::new();

        let floor_cpu = mesh::plane(engine.config.game.arena_size);
        let floor_id = meshes.upload(&engine.renderer, &engine.context.device, &floor_cpu)?;

        let cube_cpu = mesh::cube();
        let cube_gpu = engine
            .renderer
            .upload_mesh(&engine.context.device, &cube_cpu)?;
        let cube_id = meshes.add(cube_gpu);

        let camera = Camera {
            yaw: -90.0,
            pitch: 0.0,
            pos: glam::vec3(0.0, 0.0, engine.config.camera.orbit_radius),
            target: glam::vec3(0.0, 0.0, 0.0),
            fov_deg: engine.config.camera.fov_deg,
            near: engine.config.camera.near,
            far: engine.config.camera.far,
        };

        let floor = Object {
            mesh: floor_id,
            transform: Transform::identity(),
        };

        let mut cube_tf = Transform::identity();
        cube_tf.position = glam::vec3(0.0, -0.5, 0.0);

        let cube = Object {
            mesh: cube_id,
            transform: cube_tf,
        };

        let scene = Scene {
            camera,
            objects: vec![floor, cube],
            character: 1,
        };

        let rig = CameraRig {
            yaw: -90.0,
            pitch: 0.0,
            radius: engine.config.camera.orbit_radius,
            mode: CameraTargetMode::FollowCharacter,
            target: glam::Vec3::ZERO,
        };
        let motor = crate::game::character_controller::CharacterMotor::new();

        Ok(Self {
            scene,
            meshes,
            rig,
            motor,
        })
    }
}

impl GameLoop for Game {
    fn update(
        &mut self,
        engine: &mut Engine,
        input: &crate::input::input_state::InputState,
        dt: f32,
    ) -> Result<()> {
        // character move
        CharacterControllerSystem::update(
            &mut self.scene,
            &mut self.motor,
            input,
            &engine.config.controls,
            dt,
        );

        // camera rig controls (HJKL still)
        let speed_deg = engine.config.camera.orbit_speed_deg;

        if engine.window.key_down(Key::H) {
            self.rig.yaw -= speed_deg * dt;
        }
        if engine.window.key_down(Key::L) {
            self.rig.yaw += speed_deg * dt;
        }
        if engine.window.key_down(Key::J) {
            self.rig.pitch += speed_deg * dt;
        }
        if engine.window.key_down(Key::K) {
            self.rig.pitch -= speed_deg * dt;
        }

        // toggle follow/origin quickly (optional)
        if engine.window.key_down(Key::O) {
            self.rig.mode = CameraTargetMode::Origin;
        }
        if engine.window.key_down(Key::P) {
            self.rig.mode = CameraTargetMode::FollowCharacter;
        }

        // apply rig to camera
        let character_pos = self.scene.objects[self.scene.character].transform.position;

        CameraSystem::update(&mut self.scene.camera, &mut self.rig, character_pos);
        Ok(())
    }

    fn render(&mut self, engine: &mut Engine) -> Result<()> {
        let extent = engine.swapchain.extent();
        let aspect = extent.width as f32 / extent.height as f32;

        let view_proj = self.scene.camera.view_proj(aspect);
        let globals = FrameGlobals { view_proj };

        let items = self.scene.render_items(&self.meshes);
        engine.draw_frame(globals, &items)?;

        Ok(())
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        // engine Drop already waits idle, so freeing meshes here is fine if order is ok
        // BUT engine owns VkDevice; so destroy meshes inside Engine drop is safer.
        // For now, do it explicitly in main before engine drops, or store mesh destroy in engine.
    }
}
