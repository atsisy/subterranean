pub mod dream_scene;
pub mod work_scene;
pub mod scenario_scene;

use torifune::device as tdev;
use torifune::core::Clock;
use ggez::input as ginput;
use torifune::numeric;
use torifune::graphics as tgraphics;
use tgraphics::{DrawableComponent, DrawableObject};

use crate::core::GameData;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
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


///
/// # 遅延イベントを起こすための情報を保持する
///
/// ## run_time
/// 処理が走る時間
///
/// ## func
/// run_time時に実行される処理
///
pub struct SceneEvent<T> {
    run_time: Clock,
    func: Box<dyn FnOnce(&mut T, &mut ggez::Context, &GameData) -> ()>,
}

impl<T> SceneEvent<T> {
    pub fn new(f: Box<dyn FnOnce(&mut T, &mut ggez::Context, &GameData) -> ()>, t: Clock) -> Self {
	SceneEvent::<T> {
	    run_time: t,
	    func: f,
	}
    }
}

///
/// # 遅延イベントを保持しておく構造体
///
/// ## list
/// 遅延イベントのリスト, run_timeでソートされている
///
struct SceneEventList<T> {
    list: Vec<SceneEvent<T>>,
}

impl<T> SceneEventList<T> {
    pub fn new() -> Self {
	SceneEventList::<T> {
	    list: Vec::new(),
	}
    }

    pub fn add_event(&mut self, f: Box<dyn FnOnce(&mut T, &mut ggez::Context, &GameData) -> ()>,
		     t: Clock) -> &mut Self {
	self.add(SceneEvent::new(f, t))
    }

    pub fn add(&mut self, event: SceneEvent<T>) -> &mut Self {
	self.list.push(event);
	self.list.sort_by(|o1, o2| { o2.run_time.cmp(&o1.run_time) });
	self
    }

    pub fn move_top(&mut self) -> Option<SceneEvent<T>> {
	if self.list.len() > 0 {
	    self.list.pop()
	} else {
	    None
	}
    }

    pub fn len(&self) -> usize {
	self.list.len()
    }
}
