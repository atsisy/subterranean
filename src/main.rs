extern crate serde;
extern crate toml;

use ggez::*;
use std::env;
use std::path;

use ggez::conf::{FullscreenType, WindowMode};
use suzu::core::*;

pub fn main() {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("subterranean", "akichi")
        .window_setup(
            conf::WindowSetup::default()
                .title("suzu")
                .samples(ggez::conf::NumSamples::One),
        )
        .add_resource_path(resource_dir)
        .window_mode(WindowMode {
            width: 1366.0,
            height: 768.0,
            maximized: false,
            fullscreen_type: FullscreenType::Windowed,
            borderless: false,
            min_width: 0.0,
            max_width: 0.0,
            min_height: 0.0,
            max_height: 0.0,
            resizable: false,
        })
        .build()
        .unwrap();

    let game_data: GameData = GameData::new(ctx, "game_data.toml".to_owned());

    {
        let state = &mut State::new(ctx, &game_data).unwrap();
        event::run(ctx, event_loop, state).unwrap();
    }
}
