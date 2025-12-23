use crate::assets::mesh;
use crate::engine::camera::Camera;
use crate::gfx::context::VkContext;
use crate::gfx::swapchain::SwapchainManager;
use crate::platform::window_glfw::GlfwWindow;
use crate::renderer::error::RenderError;
use crate::renderer::mesh::Mesh;
use crate::renderer::renderer::Renderer;
use crate::utils::config::Config;
use anyhow::Result;
use glfw::Key;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::time::Instant;

pub struct App {
    window: GlfwWindow,
    config: Config,
    camera: Camera,

    context: VkContext, // 👈 NEW
    swapchain: SwapchainManager,
    renderer: Renderer,
    cube_mesh: Mesh,
}

impl App {
    pub fn new(cfg: Config) -> Result<Self> {
        let window = GlfwWindow::new(cfg.window.width, cfg.window.height, &cfg.window.title)?;

        // Raw handles for ash-window
        let display_handle = window
            .window
            .display_handle()
            .map_err(|e| anyhow::anyhow!("display_handle error: {:?}", e))?
            .as_raw();

        let window_handle = window
            .window
            .window_handle()
            .map_err(|e| anyhow::anyhow!("window_handle error: {:?}", e))?
            .as_raw();

        let context = VkContext::new(display_handle, window_handle)?;

        let (fb_w, fb_h) = window.framebuffer_size();

        let swapchain = SwapchainManager::new(&context, &cfg, fb_w, fb_h)?;

        let renderer = Renderer::new(
            &context.device,
            &swapchain.swapchain,
            cfg.renderer.frames_in_flight,
        )?;

        let camera = Camera {
            yaw: -90.0,
            pitch: 0.0,
            pos: glam::vec3(0.0, 0.0, cfg.camera.orbit_radius),
            target: glam::vec3(0.0, 0.0, 0.0),
            fov_deg: cfg.camera.fov_deg,
            near: cfg.camera.near,
            far: cfg.camera.far,
        };

        let cube_mesh = mesh::cube();
        let gpu_cube = renderer.upload_mesh(&context.device, &cube_mesh)?;
        // renderer.record_commands(&context.device, &swapchain.swapchain, &gpu_cube)?;

        Ok(Self {
            window,
            camera,
            config: cfg,
            context,
            swapchain,
            renderer,
            cube_mesh: gpu_cube,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut last = Instant::now();

        while !self.window.should_close() {
            self.window.poll_events();

            // --- Δt ---
            let now = Instant::now();
            let dt = (now - last).as_secs_f32();
            last = now;

            // --- keyboard → rotation ---
            let speed_deg = self.config.camera.orbit_speed_deg;

            if self.window.key_down(Key::H) {
                self.camera.yaw -= speed_deg * dt;
            }
            if self.window.key_down(Key::L) {
                self.camera.yaw += speed_deg * dt;
            }
            if self.window.key_down(Key::J) {
                self.camera.pitch += speed_deg * dt;
            }
            if self.window.key_down(Key::K) {
                self.camera.pitch -= speed_deg * dt;
            }

            // prevent flipping (degrees)
            self.camera.pitch = self.camera.pitch.clamp(-89.0, 89.0);

            // ✅ convert yaw/pitch → direction
            let yaw = self.camera.yaw.to_radians();
            let pitch = self.camera.pitch.to_radians();

            let front = glam::vec3(
                yaw.cos() * pitch.cos(),
                pitch.sin(),
                yaw.sin() * pitch.cos(),
            )
            .normalize();

            // FPS-style camera: look forward from position
            let radius = self.config.camera.orbit_radius;

            // cube stays at origin
            self.camera.target = glam::vec3(0.0, 0.0, 0.0);

            // orbit camera around cube
            self.camera.pos = self.camera.target - front * radius;

            if self.window.is_minimized() {
                // Window minimized → don't render, don't recreate swapchain
                // Let the OS breathe
                std::thread::sleep(std::time::Duration::from_millis(16));
                continue;
            }

            if self.window.take_resized() {
                let (w, h) = self.window.framebuffer_size();

                if self.swapchain.recreate(&self.context, w, h)? {
                    self.renderer
                        .rebuild_for_swapchain(&self.context, &self.swapchain.swapchain)?;

                    // self.renderer.record_commands(
                    //     &self.context.device,
                    //     &self.swapchain.swapchain,
                    //     &self.cube_mesh,
                    // )?;
                }
            }

            // // --- render ---
            let extent = self.swapchain.extent();
            let aspect = extent.width as f32 / extent.height as f32;

            let view_proj = self.camera.view_proj(aspect);

            // upload MVP to GPU

            match self.renderer.draw_frame(
                &self.context.device,
                &self.context.surface,
                &self.swapchain.swapchain,
                &self.cube_mesh,
                view_proj,
            ) {
                Ok(()) => {}
                Err(RenderError::SwapchainOutOfDate) => {
                    let (w, h) = self.window.framebuffer_size();
                    if self.swapchain.recreate(&self.context, w, h)? {
                        self.renderer
                            .rebuild_for_swapchain(&self.context, &self.swapchain.swapchain)?;
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }

        unsafe {
            self.context.device.device.device_wait_idle()?;
        }

        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            self.context.device.device.device_wait_idle().ok();
        }

        self.cube_mesh.destroy(&self.context.device.device);

        self.renderer.destroy(&self.context.device.device);
        self.swapchain.destroy(&mut self.context.device.device);
        // VkContext drops AFTER App, in correct order
    }
}
