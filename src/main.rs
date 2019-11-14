use ggez::*;
use std::env;
use std::path;
use subterranean::core::*;

pub fn main() {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("subterranean", "akichi")
        .window_setup(conf::WindowSetup::default().title("subterranean"))
        .add_resource_path(resource_dir)
        .build()
        .unwrap();
    let state = &mut State::new(ctx).unwrap();
    event::run(ctx, event_loop, state).unwrap();
}
