mod app;
mod core;
mod platform;
mod renderer;
mod resources;
mod utils;
use utils::config::load_config;
mod assets;
mod engine;
mod gfx;

fn main() -> anyhow::Result<()> {
    utils::logger::init();

    let cfg = load_config("assets/config.toml")?;
    let mut app = app::App::new(cfg)?;
    app.run()?;

    Ok(())
}
