pub mod map_parser;

use ggez::*;
use ggez::graphics as ggraphics;

use torifune::core::Clock;
use torifune::device as tdev;
use torifune::numeric;
use tdev::ProgramableKey;
use ggez::input as ginput;
use ggez::input::keyboard::*;
use std::rc::Rc;
use crate::scene;
use std::str::FromStr;

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

#[derive(Debug, Clone, Copy)]
pub enum TextureID {
    Ghost1 = 0,
    LotusPink,
    LotusBlue,
    LotusYellow,
}

#[derive(Debug, Clone, Copy)]
pub enum FontID {
    DEFAULT = 0,
}

impl FromStr for TextureID {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "Ghost1" => Ok(Self::Ghost1),
            "LotusPink" => Ok(Self::LotusPink),
            "LotusBlue" => Ok(Self::LotusBlue),
            "LotusYellow" => Ok(Self::LotusYellow),
            _ => Err(())
        }
    }
}


#[derive(Debug, Deserialize)]
pub struct RawConfigFile {
    texture_paths: Vec<String>,
    font_paths: Vec<String>,
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
    textures: Vec<Rc<ggraphics::Image>>,
    fonts: Vec<ggraphics::Font>,
}

impl GameData {
    pub fn new(ctx: &mut ggez::Context, file_path: String) -> GameData {
        let src_file = RawConfigFile::new(file_path);

        let mut data = GameData {
            textures: vec![],
            fonts: vec![],
        };
        
        for texture_path in &src_file.texture_paths {
            print!("Loading texture {}...", texture_path);
            data.textures.push(Rc::new(ggraphics::Image::new(ctx, texture_path).unwrap()));
            println!(" done!");
        }

        for font_path in &src_file.font_paths {
            print!("Loading font {}...", font_path);
            data.fonts.push(ggraphics::Font::new(ctx, font_path).unwrap());
            println!(" done!");
        }

        data
    }

    pub fn ref_texture(&self, id: TextureID) -> Rc<ggraphics::Image> {
        match self.textures.get(id as usize) {
            Some(texture) => texture.clone(),
            None => panic!("Unknown Texture ID: {}", id as i32),
        }
    }

    pub fn get_font(&self, id: FontID) -> ggraphics::Font {
        match self.fonts.get(id as usize) {
            Some(font) => *font,
            None => panic!("Unknown Font ID: {}", id as i32),
        }
    }
}

struct SceneController<'a> {
    current_scene: Box<dyn scene::SceneManager + 'a>,
    key_map: tdev::ProgramableGenericKey,
}

impl<'a> SceneController<'a> {

    pub fn new(ctx: &mut ggez::Context, game_data: &'a GameData) -> SceneController<'a> {
        SceneController {
            current_scene: Box::new(scene::task_scene::TaskScene::new(ctx, game_data)),
            key_map: tdev::ProgramableGenericKey::new()
        }
    }
    
    fn switch_scene(&mut self,
                    ctx: &mut ggez::Context,
                    game_data: &'a GameData,
                    next_scene_id: scene::SceneID) {
        if next_scene_id == scene::SceneID::MainDesk {
            self.current_scene = Box::new(scene::task_scene::TaskScene::new(ctx, game_data));
        } else if next_scene_id == scene::SceneID::Dream {
            self.current_scene = Box::new(scene::dream_scene::DreamScene::new(ctx, game_data));
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

    fn key_down_event(&mut self,
                      ctx: &mut Context,
                      keycode: KeyCode,
                      _keymods: KeyMods,
                      _repeat: bool) {
        self.current_scene.key_down_event(ctx, self.key_map.real_to_virtual(keycode));
    }
    
    fn key_up_event(&mut self,
                    ctx: &mut Context,
                    keycode: KeyCode,
                    _keymods: KeyMods,){
        self.current_scene.key_up_event(ctx, self.key_map.real_to_virtual(keycode));
    }

    fn mouse_motion_event(&mut self,
                          ctx: &mut ggez::Context,
                          point: numeric::Point2f,
                          offset: numeric::Vector2f){
        self.current_scene.mouse_motion_event(ctx, point, offset);
    }

    fn mouse_button_down_event(&mut self,
                               ctx: &mut ggez::Context,
                               button: ginput::mouse::MouseButton,
                               point: numeric::Point2f){
        self.current_scene.mouse_button_down_event(ctx, button, point);
    }
    
    fn mouse_button_up_event(&mut self,
                             ctx: &mut ggez::Context,
                             button: ginput::mouse::MouseButton,
                             point: numeric::Point2f){
        self.current_scene.mouse_button_up_event(ctx, button, point);
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

    fn key_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        keycode: KeyCode,
        keymods: KeyMods,
        repeat: bool) {
        self.scene_controller.key_down_event(ctx, keycode, keymods, repeat);
    }

    fn key_up_event(
        &mut self,
        ctx: &mut ggez::Context,
        keycode: KeyCode,
        keymods: KeyMods) {
        self.scene_controller.key_up_event(ctx, keycode, keymods);
    }

    fn mouse_motion_event(
        &mut self,
        ctx: &mut Context,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32) {
        self.scene_controller.mouse_motion_event(
            ctx,
            numeric::Point2f::new(x, y),
            numeric::Vector2f::new(dx, dy));
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        button: ginput::mouse::MouseButton,
        x: f32,
        y: f32) {
        self.scene_controller.mouse_button_down_event(ctx, button, numeric::Point2f::new(x, y));
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut Context,
        button: ginput::mouse::MouseButton,
        x: f32,
        y: f32) {
        self.scene_controller.mouse_button_up_event(ctx, button, numeric::Point2f::new(x, y));
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
