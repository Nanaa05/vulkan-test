use crate::input::input_state::InputState;
use crate::scene::scene::Scene;
use crate::utils::config::ControlsConfig;
use glam::Vec3;

pub struct CharacterControllerSystem;

impl CharacterControllerSystem {
    pub fn update(
        scene: &mut Scene,
        motor: &mut CharacterMotor,
        input: &InputState,
        cfg: &ControlsConfig,
        dt: f32,
    ) {
        // --- Tunables (keep here for now; later move to config) ---
        let gravity: f32 = 20.0;
        let charge_rate: f32 = 1.5; // charge per second
        let max_charge: f32 = 1.0;

        let base_jump_v: f32 = 6.0; // minimum jump velocity
        let extra_jump_v: f32 = 7.0; // added when fully charged

        let jump_horiz_speed: f32 = cfg.move_speed; // lock horiz speed to movement speed

        let floor_y: f32 = 0.0;
        let character_half_height: f32 = 0.5; // because your cube is [-0.5..0.5] in model space
        let ground_y = floor_y - character_half_height;

        // --- 1) Charge jump (allowed even in air) ---
        if input.jump_down {
            motor.charge = (motor.charge + charge_rate * dt).min(max_charge);
        }

        // --- 2) Compute camera-relative move dir from WASD (same as your old code) ---
        let yaw = scene.camera.yaw.to_radians();
        let cam_forward = glam::vec3(yaw.cos(), 0.0, yaw.sin()).normalize();
        let cam_right = glam::vec3(-cam_forward.z, 0.0, cam_forward.x);

        let mut dir = Vec3::ZERO;
        if input.forward {
            dir += cam_forward;
        }
        if input.back {
            dir -= cam_forward;
        }
        if input.right {
            dir += cam_right;
        }
        if input.left {
            dir -= cam_right;
        }
        if dir.length_squared() > 0.0 {
            dir = dir.normalize();
        }

        // --- 3) Release-to-jump (only if grounded) ---
        if input.jump_released {
            if motor.grounded {
                let v = base_jump_v + extra_jump_v * motor.charge;

                // horizontal direction at jump time
                let horiz = if dir.length_squared() > 0.0 {
                    dir.normalize()
                } else {
                    Vec3::ZERO
                };

                // split jump force evenly between vertical & horizontal
                // --- weighted jump vector (keeps total jump energy constant) ---
                let vertical_weight: f32 = 0.3;
                let horizontal_weight: f32 = 0.7;

                // normalize weights so total magnitude stays = v
                let norm = (vertical_weight * vertical_weight
                    + horizontal_weight * horizontal_weight)
                    .sqrt();

                // vertical (up is negative Y in your world)
                motor.vel.y = -v * (vertical_weight / norm);

                // horizontal shares remaining impulse
                motor.vel.x = horiz.x * v * (horizontal_weight / norm);
                motor.vel.z = horiz.z * v * (horizontal_weight / norm);

                motor.grounded = false;
            }
            // always reset charge on release
            motor.charge = 0.0;
        }

        // --- 4) Ground movement vs Air movement ---
        let character = scene.character_mut();

        if motor.grounded {
            // On ground: regular WASD movement (no momentum)
            if dir.length_squared() > 0.0 {
                character.transform.position += dir * cfg.move_speed * dt;
            }

            // keep vertical stable
            motor.vel.y = 0.0;
            motor.vel.x = 0.0;
            motor.vel.z = 0.0;
        } else {
            // In air: apply gravity + integrate locked velocity (no steering)
            motor.vel.y += gravity * dt;
            character.transform.position += motor.vel * dt;

            // collision with floor: stop only when we touch floor panel
            if character.transform.position.y >= ground_y {
                character.transform.position.y = ground_y;
                motor.vel = Vec3::ZERO;
                motor.grounded = true;
            }
        }
    }
}

pub struct CharacterMotor {
    vel: Vec3,
    grounded: bool,
    charge: f32, // 0..1
}

impl CharacterMotor {
    pub fn new() -> Self {
        Self {
            vel: Vec3::ZERO,
            grounded: true,
            charge: 0.0,
        }
    }
}
