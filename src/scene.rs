pub mod task_scene;
pub mod dream_scene;
pub mod scenario_scene;

use torifune::device as tdev;
use torifune::core::Clock;
use ggez::input as ginput;
use torifune::numeric;
use torifune::graphics as tgraphics;
use tgraphics::object as tobj;
use tgraphics::{DrawableComponent, DrawableObject};

use crate::core::GameData;

#[derive(Debug, Eq, PartialEq)]
pub enum SceneTransition {
    Keep,
    Reset,
    Transition,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SceneID {
    Null,
    MainDesk,
    Scenario,
    Dream,
}

pub trait SceneManager {
    
    fn key_down_event(&mut self,
                      _ctx: &mut ggez::Context,
                      _game_data: &GameData,
                      _vkey: tdev::VirtualKey) {
    }
     
    fn key_up_event(&mut self,
                    _ctx: &mut ggez::Context,
                    _game_data: &GameData,
                    _vkey: tdev::VirtualKey){
    }

    fn mouse_motion_event(&mut self,
                          _ctx: &mut ggez::Context,
                          _game_data: &GameData,
                          _point: numeric::Point2f,
                          _offset: numeric::Vector2f){
    }

    fn mouse_button_down_event(&mut self,
                               _ctx: &mut ggez::Context,
                               _game_data: &GameData,
                               _button: ginput::mouse::MouseButton,
                               _point: numeric::Point2f){
    }
    
    fn mouse_button_up_event(&mut self,
                             _ctx: &mut ggez::Context,
                             _game_data: &GameData,
                             _button: ginput::mouse::MouseButton,
                             _point: numeric::Point2f){
    }

    fn pre_process(&mut self, ctx: &mut ggez::Context, game_data: &GameData);
    
    fn drawing_process(&mut self, ctx: &mut ggez::Context);
    fn post_process(&mut self, ctx: &mut ggez::Context, game_data: &GameData) -> SceneTransition;
    fn transition(&self) -> SceneID;
    

    fn get_current_clock(&self) -> Clock;
    
    fn update_current_clock(&mut self);
}

pub struct NullScene {
}

impl NullScene {

    pub fn new() -> Self {
        NullScene {}
    }
}

impl SceneManager for NullScene {

    fn pre_process(&mut self, _ctx: &mut ggez::Context, _: &GameData) {
        
    }
    
    fn drawing_process(&mut self, _ctx: &mut ggez::Context) {
        
    }
    fn post_process(&mut self, _ctx: &mut ggez::Context, _: &GameData) -> SceneTransition {
        SceneTransition::Keep
    }

    fn transition(&self) -> SceneID {
        SceneID::Null
    }

    fn get_current_clock(&self) -> Clock {
        0
    }
    
    fn update_current_clock(&mut self) {
        
    }
}
