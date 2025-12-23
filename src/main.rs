mod core;
mod platform;
mod renderer;
mod resources;
mod utils;

mod assets;
mod engine;
mod game;
mod gfx;
mod input;
mod scene;

use utils::config::load_config;

fn main() -> anyhow::Result<()> {
    utils::logger::init();

    let cfg = load_config("assets/config.toml")?;

    let mut engine = engine::engine::Engine::new(cfg)?;
    let mut game = game::game::Game::new(&mut engine)?;

    engine.run(&mut game)?;

    // cleanup meshes before engine drops (since it owns VkDevice)
    game.meshes.destroy_all(&engine.context.device.device);

    Ok(())
}
