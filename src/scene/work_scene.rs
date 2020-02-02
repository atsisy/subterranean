pub mod work_scene_sub;

use ggez::input::mouse::MouseButton;

use torifune::core::*;
use torifune::numeric;
use torifune::device::VirtualKey;

use crate::core::{MouseInformation, MouseActionRecord, GameData};
use super::SceneEventList;
use crate::scene::*;

use work_scene_sub::TaskScene;

#[derive(PartialEq, Clone)]
pub struct TaskResult {
    done_works: u32,
    total_money: i32,
}

impl TaskResult {
    pub fn new() -> Self {
	TaskResult {
	    done_works: 0,
	    total_money: 0,
	}
    }

    pub fn add_done_works(&mut self, works: u32) -> &mut Self {
	self.done_works += works;
	self
    }

    pub fn add_money(&mut self, money: i32) -> &mut Self {
	self.total_money += money;
	self
    }

    pub fn add_result(&mut self, task_result: &TaskResult) -> &mut Self {
	self.done_works += task_result.done_works;
	self.total_money += task_result.total_money;
	self
    }

    pub fn get_done_works(&self) -> u32 {
	self.done_works
    }

    pub fn get_total_money(&self) -> i32 {
	self.total_money
    }

    pub fn reset(&mut self) -> &mut Self {
	self.done_works = 0;
	self.total_money = 0;
	
	self
    }
}

pub struct WorkScene {
    clock: Clock,
    mouse_info: MouseInformation,
    event_list: SceneEventList<Self>,
    task_scene: TaskScene,
    task_result: TaskResult,
}

impl WorkScene {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData) -> Self {

        WorkScene {
            clock: 0,
            mouse_info: MouseInformation::new(),
	    event_list: SceneEventList::new(),
	    task_scene: TaskScene::new(ctx, game_data),
	    task_result: TaskResult::new(),
        }
    }

    fn flush_task_result_data(&mut self) {
	self.task_result.add_result(self.task_scene.get_task_result());
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
	if self.task_scene.get_task_result().get_done_works() > 5 {
	    self.flush_task_result_data();
	}
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
