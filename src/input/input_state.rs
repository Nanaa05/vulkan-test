use crate::platform::window_glfw::GlfwWindow;
use crate::utils::config::ControlsConfig;
use glam::Vec3;

#[derive(Debug, Default, Clone, Copy)]
pub struct InputState {
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,

    pub jump_down: bool,
    pub jump_pressed: bool,
    pub jump_released: bool,
}

impl InputState {
    pub fn update(&mut self, window: &GlfwWindow, cfg: &ControlsConfig) {
        // WASD
        self.forward = window.key_down(cfg.forward.to_glfw());
        self.back = window.key_down(cfg.back.to_glfw());
        self.left = window.key_down(cfg.left.to_glfw());
        self.right = window.key_down(cfg.right.to_glfw());

        // Jump edges
        let new_down = window.key_down(cfg.jump.to_glfw());
        self.jump_pressed = new_down && !self.jump_down;
        self.jump_released = !new_down && self.jump_down;
        self.jump_down = new_down;
    }

    /// World-space move direction on XZ plane (Y = 0)
    pub fn move_dir(&self) -> Vec3 {
        let mut d = Vec3::ZERO;

        // Common convention: forward = -Z
        if self.forward {
            d.z -= 1.0;
        }
        if self.back {
            d.z += 1.0;
        }
        if self.left {
            d.x -= 1.0;
        }
        if self.right {
            d.x += 1.0;
        }

        if d.length_squared() > 0.0 {
            d = d.normalize();
        }
        d
    }
}
