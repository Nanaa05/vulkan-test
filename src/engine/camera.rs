use glam::{Mat4, Vec3};

#[derive(Debug, Clone)]
pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub pos: Vec3,
    pub target: Vec3,
    pub fov_deg: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn view(&self) -> Mat4 {
        Mat4::look_at_rh(self.pos, self.target, Vec3::Y)
    }

    pub fn proj(&self, aspect: f32) -> Mat4 {
        let mut p = Mat4::perspective_rh(self.fov_deg.to_radians(), aspect, self.near, self.far);
        p.y_axis.y *= -1.0; // Vulkan clip correction
        p
    }

    pub fn mvp(&self, aspect: f32, model: Mat4) -> Mat4 {
        self.proj(aspect) * self.view() * model
    }

    pub fn view_proj(&self, aspect: f32) -> glam::Mat4 {
        let view = glam::Mat4::look_at_rh(self.pos, self.target, glam::Vec3::Y);
        let proj =
            glam::Mat4::perspective_rh_gl(self.fov_deg.to_radians(), aspect, self.near, self.far);
        proj * view
    }
}
