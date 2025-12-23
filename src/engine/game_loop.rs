use anyhow::Result;

use crate::engine::engine::Engine;
use crate::input::input_state::InputState;

pub trait GameLoop {
    fn update(&mut self, engine: &mut Engine, input: &InputState, dt: f32) -> Result<()>;
    fn render(&mut self, engine: &mut Engine) -> Result<()>;
}
