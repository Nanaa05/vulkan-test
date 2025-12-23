use crate::input::keybind::KeyBind;
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub window: WindowConfig,
    pub renderer: RendererConfig,
    pub camera: CameraConfig,
    pub controls: ControlsConfig,
    pub game: GameConfig,
    pub graphics: GraphicsConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RendererConfig {
    pub frames_in_flight: usize,
}

pub fn load_config(path: &str) -> Result<Config> {
    let text = std::fs::read_to_string(path)?;
    let cfg = toml::from_str(&text)?;
    Ok(cfg)
}

#[derive(Debug, Deserialize, Clone)]
pub struct CameraConfig {
    pub fov_deg: f32,
    pub near: f32,
    pub far: f32,
    pub orbit_radius: f32,
    pub orbit_speed_deg: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ControlsConfig {
    pub move_speed: f32,

    pub forward: KeyBind,
    pub back: KeyBind,
    pub left: KeyBind,
    pub right: KeyBind,
    pub jump: KeyBind,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GameConfig {
    pub arena_size: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GraphicsConfig {
    pub clear_color: [f32; 4],
}
