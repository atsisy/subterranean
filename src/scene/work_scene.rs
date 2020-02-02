pub mod work_scene_sub;

use ggez::input::mouse::MouseButton;

use torifune::core::*;
use torifune::numeric;
use torifune::device::VirtualKey;

use crate::core::{MouseInformation, MouseActionRecord, GameData};
use super::SceneEventList;
use crate::scene::*;

use work_scene_sub::TaskScene;

pub struct WorkScene {
    clock: Clock,
    mouse_info: MouseInformation,
    event_list: SceneEventList<Self>,
    task_scene: TaskScene,
}

impl WorkScene {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData) -> Self {

        WorkScene {
            clock: 0,
            mouse_info: MouseInformation::new(),
	    event_list: SceneEventList::new(),
	    task_scene: TaskScene::new(ctx, game_data),
        }
    }
}


impl SceneManager for WorkScene {

    fn key_down_event(&mut self,
                      ctx: &mut ggez::Context,
                      game_data: &GameData,
                      vkey: VirtualKey) {
	self.task_scene.key_down_event(ctx, game_data, vkey);
    }

    fn key_up_event(&mut self,
                    ctx: &mut ggez::Context,
                    game_data: &GameData,
                    vkey: VirtualKey) {
	self.task_scene.key_up_event(ctx, game_data, vkey);
    }

    fn mouse_motion_event(&mut self,
                          ctx: &mut ggez::Context,
                          game_data: &GameData,
                          point: numeric::Point2f,
                          offset: numeric::Vector2f) {
	self.task_scene.mouse_motion_event(ctx, game_data, point, offset);
    }

    fn mouse_button_down_event(&mut self,
                               ctx: &mut ggez::Context,
                               game_data: &GameData,
                               button: MouseButton,
                               point: numeric::Point2f) {
	self.task_scene.mouse_button_down_event(ctx, game_data, button, point);
    }

    fn mouse_button_up_event(&mut self,
                             ctx: &mut ggez::Context,
                             game_data: &GameData,
                             button: MouseButton,
                             point: numeric::Point2f) {
	self.task_scene.mouse_button_up_event(ctx, game_data, button, point);
    }

    fn pre_process(&mut self,
                   ctx: &mut ggez::Context,
                   game_data: &GameData) {
	self.task_scene.pre_process(ctx, game_data);
    }
    
    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
	self.task_scene.drawing_process(ctx);
    }
    
    fn post_process(&mut self, ctx: &mut ggez::Context, game_data: &GameData) -> SceneTransition {
	self.task_scene.post_process(ctx, game_data);
        SceneTransition::Keep
    }
    
    fn transition(&self) -> SceneID {
        SceneID::MainDesk
    }

    fn get_current_clock(&self) -> Clock {
        self.clock
    }

    fn update_current_clock(&mut self) {
        self.clock += 1;
    }

}
