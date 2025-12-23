use crate::engine::game_loop::GameLoop;
use crate::engine::time::Time;
use crate::gfx::context::VkContext;
use crate::gfx::swapchain::SwapchainManager;
use crate::input::input_state::InputState;
use crate::platform::window_glfw::GlfwWindow;
use crate::renderer::error::RenderError;
use crate::renderer::renderer::Renderer;
use crate::utils::config::Config;
use anyhow::Result;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub struct Engine {
    pub window: GlfwWindow,
    pub config: Config,

    pub context: VkContext,
    pub swapchain: SwapchainManager,
    pub renderer: Renderer,

    pub input: InputState,
    pub time: Time,
}

impl Engine {
    pub fn new(cfg: Config) -> Result<Self> {
        let window = GlfwWindow::new(cfg.window.width, cfg.window.height, &cfg.window.title)?;

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

        Ok(Self {
            window,
            config: cfg,
            context,
            swapchain,
            renderer,
            input: InputState::default(),
            time: Time::new(),
        })
    }

    pub fn run<G: GameLoop>(&mut self, game: &mut G) -> Result<()> {
        while !self.window.should_close() {
            self.window.poll_events();

            let dt = self.time.tick();

            self.input.update(&self.window, &self.config.controls);

            let input = self.input; // COPY
            game.update(self, &input, dt)?;

            if self.window.is_minimized() {
                std::thread::sleep(std::time::Duration::from_millis(16));
                continue;
            }

            // swapchain resize handling stays in engine
            if self.window.take_resized() {
                let (w, h) = self.window.framebuffer_size();
                if self.swapchain.recreate(&self.context, w, h)? {
                    self.renderer
                        .rebuild_for_swapchain(&self.context, &self.swapchain.swapchain)?;
                }
            }

            // game drives what to render
            match game.render(self) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }
        }

        unsafe {
            self.context.device.device.device_wait_idle()?;
        }
        Ok(())
    }

    pub fn draw_frame(
        &mut self,
        globals: crate::renderer::render_types::FrameGlobals,
        items: &[crate::renderer::render_types::RenderItem],
    ) -> Result<()> {
        match self.renderer.draw_frame(
            &self.context.device,
            &self.context.surface,
            &self.swapchain.swapchain,
            globals,
            items,
        ) {
            Ok(()) => Ok(()),
            Err(RenderError::SwapchainOutOfDate) => {
                let (w, h) = self.window.framebuffer_size();
                if self.swapchain.recreate(&self.context, w, h)? {
                    self.renderer
                        .rebuild_for_swapchain(&self.context, &self.swapchain.swapchain)?;
                }
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe {
            self.context.device.device.device_wait_idle().ok();
        }
        self.renderer.destroy(&self.context.device.device);
        self.swapchain.destroy(&mut self.context.device.device);
    }
}
