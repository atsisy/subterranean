pub mod work_scene_sub;

use std::any::Any;

use ggez::input::mouse::MouseButton;

use torifune::core::*;
use torifune::numeric;
use torifune::device::VirtualKey;

use crate::core::{MouseInformation, GameData};
use super::SceneEventList;
use crate::scene::*;

use work_scene_sub::{TaskScene, TaskResultScene};
use work_scene_sub::TaskSceneStatus;

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

#[derive(PartialEq, Clone, Copy)]
pub enum WorkSceneStatus {
    InTask,
    Result,
}

pub struct WorkSubScene {
    task_scene: Option<Box<TaskScene>>,
    task_result_scene: Option<Box<TaskResultScene>>,
    scene_status: WorkSceneStatus,
}

impl WorkSubScene {
    pub fn new(scene_box: Box<dyn Any>, scene_status: WorkSceneStatus) -> Self {
	let mut sub_scene = WorkSubScene {
	    task_scene: None,
	    task_result_scene: None,
	    scene_status: scene_status,
	};

	sub_scene.switch_scene(scene_box, scene_status);

	sub_scene
    }

    pub fn get_task_scene_mut(&mut self) -> Option<&mut Box<TaskScene>> {
	self.task_scene.as_mut()
    }

    pub fn get_result_scene_mut(&mut self) -> Option<&mut Box<TaskResultScene>> {
	self.task_result_scene.as_mut()
    }

    pub fn switch_scene(&mut self, scene_box: Box<dyn Any>, scene_status: WorkSceneStatus) {
	match scene_status {
	    WorkSceneStatus::InTask => {
		if let Ok(task_scene) = scene_box.downcast::<TaskScene>() {
		    self.task_scene = Some(task_scene);
		}
	    },
	    WorkSceneStatus::Result => {
		if let Ok(result_scene) = scene_box.downcast::<TaskResultScene>() {
		    self.task_result_scene = Some(result_scene);
		}
	    }
	}

	self.scene_status = scene_status;
    }

    pub fn get_status(&self) -> WorkSceneStatus {
	self.scene_status
    }

}

impl SceneManager for WorkSubScene {
    fn key_down_event(&mut self,
                      ctx: &mut ggez::Context,
                      game_data: &GameData,
                      vkey: VirtualKey) {
	match self.scene_status {
	    WorkSceneStatus::InTask => {
		self.task_scene.as_mut().unwrap().key_down_event(ctx, game_data, vkey);
	    },
	    WorkSceneStatus::Result => {
		self.task_result_scene.as_mut().unwrap().key_down_event(ctx, game_data, vkey);
	    }
	}
    }

    fn key_up_event(&mut self,
                    ctx: &mut ggez::Context,
                    game_data: &GameData,
                    vkey: VirtualKey) {
	match self.scene_status {
	    WorkSceneStatus::InTask => {
		self.task_scene.as_mut().unwrap().key_up_event(ctx, game_data, vkey);
	    },
	    WorkSceneStatus::Result => {
		self.task_result_scene.as_mut().unwrap().key_up_event(ctx, game_data, vkey);
	    }
	}
    }

    fn mouse_motion_event(&mut self,
                          ctx: &mut ggez::Context,
                          game_data: &GameData,
                          point: numeric::Point2f,
                          offset: numeric::Vector2f) {
	match self.scene_status {
	    WorkSceneStatus::InTask => {
		self.task_scene.as_mut().unwrap().mouse_motion_event(ctx, game_data, point, offset);
	    },
	    WorkSceneStatus::Result => {
		self.task_result_scene.as_mut().unwrap().mouse_motion_event(ctx, game_data, point, offset);
	    }
	}
    }

    fn mouse_button_down_event(&mut self,
                               ctx: &mut ggez::Context,
                               game_data: &GameData,
                               button: MouseButton,
                               point: numeric::Point2f) {
	match self.scene_status {
	    WorkSceneStatus::InTask => {
		self.task_scene.as_mut().unwrap().mouse_button_down_event(ctx, game_data, button, point);
	    },
	    WorkSceneStatus::Result => {
		self.task_result_scene.as_mut().unwrap().mouse_button_down_event(ctx, game_data, button, point);
	    }
	}
    }

    fn mouse_button_up_event(&mut self,
                             ctx: &mut ggez::Context,
                             game_data: &GameData,
                             button: MouseButton,
                             point: numeric::Point2f) {
	match self.scene_status {
	    WorkSceneStatus::InTask => {
		self.task_scene.as_mut().unwrap().mouse_button_up_event(ctx, game_data, button, point);
	    },
	    WorkSceneStatus::Result => {
		self.task_result_scene.as_mut().unwrap().mouse_button_up_event(ctx, game_data, button, point);
	    }
	}
    }

    fn pre_process(&mut self,
                   ctx: &mut ggez::Context,
                   game_data: &GameData) {	
	match self.scene_status {
	    WorkSceneStatus::InTask => {
		self.task_scene.as_mut().unwrap().pre_process(ctx, game_data);
	    },
	    WorkSceneStatus::Result => {
		self.task_result_scene.as_mut().unwrap().pre_process(ctx, game_data);
	    }
	}
    }
    
    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
	match self.scene_status {
	    WorkSceneStatus::InTask => {
		self.task_scene.as_mut().unwrap().drawing_process(ctx);
	    },
	    WorkSceneStatus::Result => {
		self.task_result_scene.as_mut().unwrap().drawing_process(ctx);
	    }
	}
    }
    
    fn post_process(&mut self, ctx: &mut ggez::Context, game_data: &GameData) -> SceneTransition {
	match self.scene_status {
	    WorkSceneStatus::InTask => {
		self.task_scene.as_mut().unwrap().post_process(ctx, game_data)
	    },
	    WorkSceneStatus::Result => {
		self.task_result_scene.as_mut().unwrap().post_process(ctx, game_data)
	    }
	}
    }
    
    fn transition(&self) -> SceneID {
	match self.scene_status {
	    WorkSceneStatus::InTask => {
		self.task_scene.as_ref().unwrap().transition()
	    },
	    WorkSceneStatus::Result => {
		self.task_result_scene.as_ref().unwrap().transition()
	    }
	}
    }

    fn get_current_clock(&self) -> Clock {
	match self.scene_status {
	    WorkSceneStatus::InTask => {
		self.task_scene.as_ref().unwrap().get_current_clock()
	    },
	    WorkSceneStatus::Result => {
		self.task_result_scene.as_ref().unwrap().get_current_clock()
	    }
	}
    }

    fn update_current_clock(&mut self) {
	match self.scene_status {
	    WorkSceneStatus::InTask => {
		self.task_scene.as_mut().unwrap().update_current_clock()
	    },
	    WorkSceneStatus::Result => {
		self.task_result_scene.as_mut().unwrap().update_current_clock()
	    }
	}
    }

}

pub struct WorkScene {
    clock: Clock,
    mouse_info: MouseInformation,
    event_list: SceneEventList<Self>,
    sub_scene: WorkSubScene,
    task_result: TaskResult,
}

impl WorkScene {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData) -> Self {

        WorkScene {
            clock: 0,
            mouse_info: MouseInformation::new(),
	    event_list: SceneEventList::new(),
	    sub_scene: WorkSubScene::new(Box::new(TaskScene::new(ctx, game_data)), WorkSceneStatus::InTask),
	    task_result: TaskResult::new(),
        }
    }

    fn flush_task_result_data(&mut self) {
	if let Some(task_scene) = self.sub_scene.get_task_scene_mut() {
	    self.task_result.add_result(task_scene.get_task_result());
	}
    }

    fn switch_task_scene_to_result_scene(&mut self, ctx: &mut ggez::Context, game_data: &GameData) {
	self.sub_scene.switch_scene(
	    Box::new(TaskResultScene::new(ctx, game_data, self.task_result.clone())),
	    WorkSceneStatus::Result);
    }
}


impl SceneManager for WorkScene {

    fn key_down_event(&mut self,
                      ctx: &mut ggez::Context,
                      game_data: &GameData,
                      vkey: VirtualKey) {
	self.sub_scene.key_down_event(ctx, game_data, vkey);
    }

    fn key_up_event(&mut self,
                    ctx: &mut ggez::Context,
                    game_data: &GameData,
                    vkey: VirtualKey) {
	self.sub_scene.key_up_event(ctx, game_data, vkey);
    }

    fn mouse_motion_event(&mut self,
                          ctx: &mut ggez::Context,
                          game_data: &GameData,
                          point: numeric::Point2f,
                          offset: numeric::Vector2f) {
	self.sub_scene.mouse_motion_event(ctx, game_data, point, offset);
    }

    fn mouse_button_down_event(&mut self,
                               ctx: &mut ggez::Context,
                               game_data: &GameData,
                               button: MouseButton,
                               point: numeric::Point2f) {
	self.sub_scene.mouse_button_down_event(ctx, game_data, button, point);
    }

    fn mouse_button_up_event(&mut self,
                             ctx: &mut ggez::Context,
                             game_data: &GameData,
                             button: MouseButton,
                             point: numeric::Point2f) {
	self.sub_scene.mouse_button_up_event(ctx, game_data, button, point);
    }

    fn pre_process(&mut self,
                   ctx: &mut ggez::Context,
                   game_data: &GameData) {
	self.sub_scene.pre_process(ctx, game_data);
    }
    
    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
	self.sub_scene.drawing_process(ctx);
    }
    
    fn post_process(&mut self, ctx: &mut ggez::Context, game_data: &GameData) -> SceneTransition {
	self.sub_scene.post_process(ctx, game_data);

	match self.sub_scene.get_status() {
	    WorkSceneStatus::InTask => {
		if self.sub_scene.get_task_scene_mut().unwrap().get_task_status() == TaskSceneStatus::FinishDay {
		    self.flush_task_result_data();
		    self.switch_task_scene_to_result_scene(ctx, game_data);
		}
	    },
	    WorkSceneStatus::Result => {
	    },
	}

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
