use ggez::input::mouse::MouseButton;

use torifune::core::*;
use torifune::numeric;
use torifune::device::VirtualKey;
use torifune::debug;

use crate::core::GameData;
use crate::scene::*;

use crate::scene::work_scene::WorkScene;
use crate::scene::work_scene::work_scene_sub::TaskScene;
use crate::scene::work_scene::work_scene_sub::TaskResultScene;
use crate::scene::shop_scene::ShopScene;

use crate::scene::work_scene::TaskResult;

#[derive(PartialEq, Clone, Copy)]
pub enum SuzunaSceneStatus {
    Shop,
    DeskWork,
    DayResult,
}

pub struct SuzunaSubScene {
    shop_scene: Option<Box<ShopScene>>,
    desk_work_scene: Option<Box<TaskScene>>,
    day_result_scene: Option<Box<TaskResultScene>>,
    scene_status: SuzunaSceneStatus,
}

impl SuzunaSubScene {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData, map_id: u32) -> Self {
	SuzunaSubScene {
	    shop_scene: Some(Box::new(ShopScene::new(ctx, game_data, map_id))),
	    desk_work_scene: None,
	    day_result_scene: None,
	    scene_status: SuzunaSceneStatus::Shop,
	}
    }

    pub fn get_shop_scene_mut(&mut self) -> Option<&mut Box<ShopScene>> {
	self.shop_scene.as_mut()
    }

    pub fn get_deskwork_scene_mut(&mut self) -> Option<&mut Box<TaskScene>> {
	self.desk_work_scene.as_mut()
    }

    pub fn get_dayresult_scene_mut(&mut self) -> Option<&mut Box<TaskResultScene>> {
	self.day_result_scene.as_mut()
    }
    
    pub fn get_scene_status(&self) -> SuzunaSceneStatus {
	self.scene_status
    }

    fn switch_shop_to_deskwork(&mut self, ctx: &mut ggez::Context, game_data: &GameData, transition: SceneTransition) {
	if transition == SceneTransition::StackingTransition {
	    self.scene_status = SuzunaSceneStatus::DeskWork;
	    self.desk_work_scene = Some(Box::new(TaskScene::new(ctx, game_data)));
	}
    }

    fn switch_deskwork_to_shop(&mut self, transition: SceneTransition) {
	if transition == SceneTransition::PoppingTransition {
	    self.scene_status = SuzunaSceneStatus::Shop;
	    self.desk_work_scene = None;
	    self.shop_scene.as_mut().unwrap().switched_and_restart();
	}
    }
}

impl SceneManager for SuzunaSubScene {
    fn key_down_event(&mut self,
                      ctx: &mut ggez::Context,
                      game_data: &GameData,
                      vkey: VirtualKey) {
	match self.scene_status {
	    SuzunaSceneStatus::Shop => {
		self.shop_scene.as_mut().unwrap().key_down_event(ctx, game_data, vkey);
	    },
	    SuzunaSceneStatus::DeskWork => {
		self.desk_work_scene.as_mut().unwrap().key_down_event(ctx, game_data, vkey);
	    },
	    SuzunaSceneStatus::DayResult => {
		self.day_result_scene.as_mut().unwrap().key_down_event(ctx, game_data, vkey);
	    }
	}
    }

    fn key_up_event(&mut self,
                    ctx: &mut ggez::Context,
                    game_data: &GameData,
                    vkey: VirtualKey) {
	match self.scene_status {
	    SuzunaSceneStatus::Shop => {
		self.shop_scene.as_mut().unwrap().key_up_event(ctx, game_data, vkey);
	    },
	    SuzunaSceneStatus::DeskWork => {
		self.desk_work_scene.as_mut().unwrap().key_up_event(ctx, game_data, vkey);
	    },
	    SuzunaSceneStatus::DayResult => {
		self.day_result_scene.as_mut().unwrap().key_up_event(ctx, game_data, vkey);
	    }
	}
    }

    fn mouse_motion_event(&mut self,
                          ctx: &mut ggez::Context,
                          game_data: &GameData,
                          point: numeric::Point2f,
                          offset: numeric::Vector2f) {
	match self.scene_status {
	    SuzunaSceneStatus::Shop => {
		self.shop_scene.as_mut().unwrap().mouse_motion_event(ctx, game_data, point, offset);
	    },
	    SuzunaSceneStatus::DeskWork => {
		self.desk_work_scene.as_mut().unwrap().mouse_motion_event(ctx, game_data, point, offset);
	    },
	    SuzunaSceneStatus::DayResult => {
		self.day_result_scene.as_mut().unwrap().mouse_motion_event(ctx, game_data, point, offset);
	    }
	}
    }

    fn mouse_button_down_event(&mut self,
                               ctx: &mut ggez::Context,
                               game_data: &GameData,
                               button: MouseButton,
                               point: numeric::Point2f) {
	match self.scene_status {
	    SuzunaSceneStatus::Shop => {
		self.shop_scene.as_mut().unwrap().mouse_button_down_event(ctx, game_data, button, point);
	    },
	    SuzunaSceneStatus::DeskWork => {
		self.desk_work_scene.as_mut().unwrap().mouse_button_down_event(ctx, game_data, button, point);
	    },
	    SuzunaSceneStatus::DayResult => {
		self.day_result_scene.as_mut().unwrap().mouse_button_down_event(ctx, game_data, button, point);
	    }
	}
    }

    fn mouse_button_up_event(&mut self,
                             ctx: &mut ggez::Context,
                             game_data: &GameData,
                             button: MouseButton,
                             point: numeric::Point2f) {
	match self.scene_status {
	    SuzunaSceneStatus::Shop => {
		self.shop_scene.as_mut().unwrap().mouse_button_up_event(ctx, game_data, button, point);
	    },
	    SuzunaSceneStatus::DeskWork => {
		self.desk_work_scene.as_mut().unwrap().mouse_button_up_event(ctx, game_data, button, point);
	    },
	    SuzunaSceneStatus::DayResult => {
		self.day_result_scene.as_mut().unwrap().mouse_button_up_event(ctx, game_data, button, point);
	    }
	}
    }

    fn pre_process(&mut self,
                   ctx: &mut ggez::Context,
                   game_data: &GameData) {	
	match self.scene_status {
	    SuzunaSceneStatus::Shop => {
		self.shop_scene.as_mut().unwrap().pre_process(ctx, game_data);
	    },
	    SuzunaSceneStatus::DeskWork => {
		self.desk_work_scene.as_mut().unwrap().pre_process(ctx, game_data);
	    },
	    SuzunaSceneStatus::DayResult => {
		self.day_result_scene.as_mut().unwrap().pre_process(ctx, game_data);
	    }
	}
    }
    
    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
	match self.scene_status {
	    SuzunaSceneStatus::Shop => {
		self.shop_scene.as_mut().unwrap().drawing_process(ctx);
	    },
	    SuzunaSceneStatus::DeskWork => {
		self.desk_work_scene.as_mut().unwrap().drawing_process(ctx);
	    },
	    SuzunaSceneStatus::DayResult => {
		self.day_result_scene.as_mut().unwrap().drawing_process(ctx);
	    }
	}
    }
    
    fn post_process(&mut self, ctx: &mut ggez::Context, game_data: &GameData) -> SceneTransition {
	match self.scene_status {
	    SuzunaSceneStatus::Shop => {
		self.shop_scene.as_mut().unwrap().post_process(ctx, game_data)
	    },
	    SuzunaSceneStatus::DeskWork => {
		self.desk_work_scene.as_mut().unwrap().post_process(ctx, game_data)
	    },
	    SuzunaSceneStatus::DayResult => {
		self.day_result_scene.as_mut().unwrap().post_process(ctx, game_data)
	    }
	}
    }
    
    fn transition(&self) -> SceneID {
	match self.scene_status {
	    SuzunaSceneStatus::Shop => {
		self.shop_scene.as_ref().unwrap().transition()
	    },
	    SuzunaSceneStatus::DeskWork => {
		self.desk_work_scene.as_ref().unwrap().transition()
	    },
	    SuzunaSceneStatus::DayResult => {
		self.day_result_scene.as_ref().unwrap().transition()
	    }
	}
    }

    fn get_current_clock(&self) -> Clock {
	match self.scene_status {
	    SuzunaSceneStatus::Shop => {
		self.shop_scene.as_ref().unwrap().get_current_clock()
	    },
	    SuzunaSceneStatus::DeskWork => {
		self.desk_work_scene.as_ref().unwrap().get_current_clock()
	    },
	    SuzunaSceneStatus::DayResult => {
		self.day_result_scene.as_ref().unwrap().get_current_clock()
	    }
	}
    }

    fn update_current_clock(&mut self) {
	match self.scene_status {
	    SuzunaSceneStatus::Shop => {
		self.shop_scene.as_mut().unwrap().update_current_clock()
	    },
	    SuzunaSceneStatus::DeskWork => {
		self.desk_work_scene.as_mut().unwrap().update_current_clock()
	    },
	    SuzunaSceneStatus::DayResult => {
		self.day_result_scene.as_mut().unwrap().update_current_clock()
	    }
	}
    }
}

pub struct SuzunaScene {
    clock: Clock,
    sub_scene: SuzunaSubScene,
    task_result: TaskResult,
}

impl SuzunaScene {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData, suzuna_map_id: u32) -> Self {
	SuzunaScene {
	    clock: 0,
	    sub_scene: SuzunaSubScene::new(ctx, game_data, suzuna_map_id),
	    task_result: TaskResult::new(),
	}
    }
}

impl SceneManager for SuzunaScene {

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
	let transition_status = self.sub_scene.post_process(ctx, game_data);

	match self.sub_scene.get_scene_status() {
	    SuzunaSceneStatus::Shop => {
		if transition_status == SceneTransition::StackingTransition {
		    if self.sub_scene.get_shop_scene_mut().unwrap().transition() == SceneID::MainDesk {
			debug::debug_screen_push_text("switch shop -> work");
			self.sub_scene.switch_shop_to_deskwork(ctx, game_data, transition_status);
		    }
		}
	    },
	    SuzunaSceneStatus::DeskWork => {
		if transition_status == SceneTransition::PoppingTransition {
		    if self.sub_scene.get_deskwork_scene_mut().unwrap().transition() == SceneID::Dream {
			self.task_result.add_result(self.sub_scene.desk_work_scene.as_ref().unwrap().get_task_result());
			self.sub_scene.switch_deskwork_to_shop(transition_status);
			debug::debug_screen_push_text(&format!("{:?}", self.task_result));
		    }
		}
	    },
	    SuzunaSceneStatus::DayResult => {
		debug::debug_screen_push_text("Implement Result!!!!!!!!!!!!!");
	    }
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
