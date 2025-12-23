use anyhow::Result;
use glfw::{Action, ClientApiHint, GlfwReceiver, WindowEvent, WindowHint, WindowMode};

pub struct GlfwWindow {
    pub glfw: glfw::Glfw,
    pub window: glfw::PWindow,
    pub events: GlfwReceiver<(f64, WindowEvent)>,
    resized: bool,
}

impl GlfwWindow {
    pub fn new(width: u32, height: u32, title: &str) -> Result<Self> {
        let mut glfw = glfw::init(glfw::fail_on_errors)?;

        // Vulkan wants NO OpenGL context.
        glfw.window_hint(WindowHint::ClientApi(ClientApiHint::NoApi));
        glfw.window_hint(WindowHint::Resizable(true));

        let (mut window, events) = glfw
            .create_window(width, height, title, WindowMode::Windowed)
            .ok_or_else(|| anyhow::anyhow!("Failed to create GLFW window"))?;

        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);

        Ok(Self {
            glfw,
            window,
            events,
            resized: false,
        })
    }

    pub fn poll_events(&mut self) {
        self.glfw.poll_events();
        for (_, e) in glfw::flush_messages(&self.events) {
            match e {
                WindowEvent::FramebufferSize(_, _) => self.resized = true,
                WindowEvent::Key(glfw::Key::Escape, _, Action::Press, _) => {
                    self.window.set_should_close(true)
                }
                _ => {}
            }
        }
    }

    pub fn should_close(&self) -> bool {
        self.window.should_close()
    }

    pub fn take_resized(&mut self) -> bool {
        let r = self.resized;
        self.resized = false;
        r
    }

    pub fn framebuffer_size(&self) -> (u32, u32) {
        let (w, h) = self.window.get_framebuffer_size();
        (w as u32, h as u32)
    }
    pub fn is_minimized(&self) -> bool {
        let (w, h) = self.window.get_framebuffer_size();
        w == 0 || h == 0
    }

    pub fn key_down(&self, key: glfw::Key) -> bool {
        matches!(self.window.get_key(key), Action::Press | Action::Repeat)
    }
}
