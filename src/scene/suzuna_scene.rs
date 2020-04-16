pub mod suzuna_sub_scene;

use ggez::input::mouse::MouseButton;

use torifune::core::*;
use torifune::debug;
use torifune::device::VirtualKey;
use torifune::numeric;

use crate::core::{GameData, GensoDate, TaskResult, GameStatus};
use crate::scene::*;

use crate::object::task_object::tt_sub_component::CopyingRequestInformation;

use suzuna_sub_scene::*;

pub struct SuzunaScene {
    clock: Clock,
    sub_scene: SuzunaSubScene,
    task_result: TaskResult,
}

impl SuzunaScene {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData, suzuna_map_id: u32, game_status: GameStatus) -> Self {
        SuzunaScene {
            clock: 0,
            sub_scene: SuzunaSubScene::new(ctx, game_data, suzuna_map_id, game_status.clone()),
            task_result: game_status.task_result,
        }
    }

    fn transition_shop_scene_to_others(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        transition_status: SceneTransition,
    ) {
        if transition_status == SceneTransition::StackingTransition {
            if self.sub_scene.get_shop_scene_mut().unwrap().transition() == SceneID::MainDesk {
                debug::debug_screen_push_text("switch shop -> work");
                self.sub_scene
                    .switch_shop_to_deskwork(ctx, game_data, transition_status);
            }
        }

        if self.sub_scene.get_shop_scene_mut().unwrap().transition() == SceneID::DayResult {
            debug::debug_screen_push_text("switch shop -> result");
            self.sub_scene
                .switch_shop_to_day_result(ctx, game_data, transition_status);
        }

        if self.sub_scene.get_shop_scene_mut().unwrap().transition() == SceneID::Copying {
            debug::debug_screen_push_text("switch shop -> copying");
            self.sub_scene
                .switch_shop_to_copying(ctx, game_data, transition_status);
        }
    }

    pub fn get_task_result(&self) -> TaskResult {
	self.task_result.clone()
    }
}

impl SceneManager for SuzunaScene {
    fn key_down_event(&mut self, ctx: &mut ggez::Context, game_data: &GameData, vkey: VirtualKey) {
        self.sub_scene.key_down_event(ctx, game_data, vkey);
    }

    fn key_up_event(&mut self, ctx: &mut ggez::Context, game_data: &GameData, vkey: VirtualKey) {
        self.sub_scene.key_up_event(ctx, game_data, vkey);
    }

    fn mouse_motion_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        point: numeric::Point2f,
        offset: numeric::Vector2f,
    ) {
        self.sub_scene
            .mouse_motion_event(ctx, game_data, point, offset);
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        self.sub_scene
            .mouse_button_down_event(ctx, game_data, button, point);
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        self.sub_scene
            .mouse_button_up_event(ctx, game_data, button, point);
    }

    fn mouse_wheel_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        point: numeric::Point2f,
        x: f32,
        y: f32,
    ) {
        self.sub_scene
            .mouse_wheel_event(ctx, game_data, point, x, y);
    }

    fn pre_process(&mut self, ctx: &mut ggez::Context, game_data: &GameData) {
        self.sub_scene.pre_process(ctx, game_data);
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.sub_scene.drawing_process(ctx);
    }

    fn post_process(&mut self, ctx: &mut ggez::Context, game_data: &GameData) -> SceneTransition {
        let transition_status = self.sub_scene.post_process(ctx, game_data);

        match self.sub_scene.get_scene_status() {
            SuzunaSceneStatus::Shop => {
                self.transition_shop_scene_to_others(ctx, game_data, transition_status);
            }
            SuzunaSceneStatus::DeskWork => {
                if transition_status == SceneTransition::PoppingTransition {
                    if self
                        .sub_scene
                        .get_deskwork_scene_mut()
                        .unwrap()
                        .transition()
                        == SceneID::SuzunaShop
                    {
                        self.task_result.add_result(
                            self.sub_scene
                                .desk_work_scene
                                .as_ref()
                                .unwrap()
                                .get_task_result(),
                        );
                        self.sub_scene
                            .switch_deskwork_to_shop(ctx, game_data, transition_status);
                        self.sub_scene
                            .shop_scene
                            .as_mut()
                            .unwrap()
                            .update_task_result(ctx, game_data, &self.task_result);
                        debug::debug_screen_push_text(&format!("{:?}", self.task_result));
                    }
                }
            }
            SuzunaSceneStatus::DayResult => {
                return transition_status;
            }
            SuzunaSceneStatus::Copying => {}
        }

        SceneTransition::Keep
    }

    fn transition(&self) -> SceneID {
        self.sub_scene.transition()
    }

    fn get_current_clock(&self) -> Clock {
        self.clock
    }

    fn update_current_clock(&mut self) {
        self.clock += 1;
    }
}
