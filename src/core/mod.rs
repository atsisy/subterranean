use ggez::*;
use ggez::graphics as ggraphics;

use torifune::core::Clock;
use crate::scene;

use std::fs;
use std::io::{BufReader, Read};

use serde::Deserialize;

fn read_whole_file(path: String) -> Result<String, String> {
    let mut file_content = String::new();

    let mut fr = fs::File::open(path)
        .map(|f| BufReader::new(f))
        .map_err(|e| e.to_string())?;

    fr.read_to_string(&mut file_content)
        .map_err(|e| e.to_string())?;

    Ok(file_content)
}

#[derive(Debug, Deserialize)]
pub struct RawConfigFile {
    texture_paths: Vec<String>,
}

impl RawConfigFile {
    pub fn new(file_path: String) -> RawConfigFile {
        let s = match read_whole_file(file_path) {
            Ok(s) => s,
            Err(e) => panic!("Failed to read file: {}", e),
        };
        
        let raw_data: Result<RawConfigFile, toml::de::Error> = toml::from_str(&s);
        match raw_data {
            Ok(p) => p,
            Err(e) => panic!("Failed to parse toml: {}", e),
        }
    }
}

pub struct GameData {
    textures: Vec<ggraphics::Image>,
}

impl GameData {
    pub fn new(ctx: &mut ggez::Context, file_path: String) -> GameData {
        let src_file = RawConfigFile::new(file_path);

        let mut data = GameData {
            textures: vec![]
        };
        
        for texture_path in &src_file.texture_paths {
            data.textures.push(ggraphics::Image::new(ctx, texture_path).unwrap());
        }

        data
    }

    pub fn ref_texture(&self, index: usize) -> &ggraphics::Image {
        match self.textures.get(index) {
            Some(texture) => &texture,
            None => panic!("Unknown Texture ID: {}", index),
        }
    }
}

struct SceneController<'a> {
    current_scene: Box<dyn scene::SceneManager + 'a>,
}

impl<'a> SceneController<'a> {

    pub fn new(ctx: &mut ggez::Context, game_data: &'a GameData) -> SceneController<'a> {
        SceneController {
            current_scene: Box::new(scene::task_scene::TaskScene::new(ctx, game_data))
        }
    }
    
    fn switch_scene(&mut self,
                    ctx: &mut ggez::Context,
                    game_data: &'a GameData,
                    next_scene_id: scene::SceneID) {
        if next_scene_id == scene::SceneID::MainDesk {
            self.current_scene = Box::new(scene::task_scene::TaskScene::new(ctx, game_data));
        } else if next_scene_id == scene::SceneID::Null {
            self.current_scene = Box::new(scene::NullScene::new());
        }
    }

    fn run_pre_process(&mut self, ctx: &mut ggez::Context) {
        self.current_scene.pre_process(ctx);
    }

    fn run_drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.current_scene.drawing_process(ctx);
    }

    fn run_post_process(&mut self, ctx: &mut ggez::Context, game_data: &'a GameData) {
        match self.current_scene.post_process(ctx) {
            scene::SceneTransition::Keep => (),
            _ => self.switch_scene(ctx, game_data, self.current_scene.transition()),
        }
    }
}

pub struct State<'data> {
    clock: Clock,
    fps: f64,
    scene_controller: SceneController<'data>,
    game_data: &'data GameData,
}

impl<'data> ggez::event::EventHandler for State<'data> {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {

        self.scene_controller.run_pre_process(ctx);
        
        self.clock += 1;
        if (self.clock % 100) == 0 {
            self.fps = timer::fps(ctx);
        }

        Ok(())
    }
    
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 0.0].into());

        self.scene_controller.run_drawing_process(ctx);
        
        graphics::present(ctx)?;

        self.scene_controller.run_post_process(ctx, self.game_data);
        
        Ok(())
    }
}

impl<'data> State<'data> {
    pub fn new(ctx: &mut Context, game_data: &'data GameData) -> GameResult<State<'data>> {
        let s = State {
            clock: 0,
            fps: 0.0,
            game_data: game_data,
            scene_controller: SceneController::new(ctx, game_data)
        };
        
        Ok(s)
    }

}
