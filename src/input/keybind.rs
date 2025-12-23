use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum KeyBind {
    W,
    A,
    S,
    D,
    Up,
    Down,
    Left,
    Right,
    Space,
    LeftShift,
}

impl KeyBind {
    pub fn to_glfw(self) -> glfw::Key {
        match self {
            KeyBind::W => glfw::Key::W,
            KeyBind::A => glfw::Key::A,
            KeyBind::S => glfw::Key::S,
            KeyBind::D => glfw::Key::D,
            KeyBind::Up => glfw::Key::Up,
            KeyBind::Down => glfw::Key::Down,
            KeyBind::Left => glfw::Key::Left,
            KeyBind::Right => glfw::Key::Right,
            KeyBind::Space => glfw::Key::Space,
            KeyBind::LeftShift => glfw::Key::LeftShift,
        }
    }
}
