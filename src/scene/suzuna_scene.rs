pub mod suzuna_sub_scene;

use ggez::input::mouse::MouseButton;

use torifune::core::*;
use torifune::debug;
use torifune::device::VirtualKey;
use torifune::numeric;

use crate::core::{BookInformation, GameData};
use crate::scene::*;

use crate::object::task_object::tt_sub_component::CopyingRequestInformation;

use suzuna_sub_scene::*;

#[derive(Clone, Debug)]
pub struct TaskResult {
    pub done_works: u32,                                     // 総仕事数
    pub not_shelved_books: Vec<BookInformation>,             // 返却済, 未配架
    pub borrowing_books: Vec<BookInformation>,               // 貸出中
    pub remain_copy_request: Vec<CopyingRequestInformation>, // 写本待
    pub total_money: i32,                                    // 稼いだ金額
}

impl TaskResult {
    pub fn new() -> Self {
        TaskResult {
            done_works: 0,
            not_shelved_books: Vec::new(),
            total_money: 0,
            borrowing_books: Vec::new(),
            remain_copy_request: Vec::new(),
        }
    }

    pub fn add_result(&mut self, task_result: &TaskResult) -> &mut Self {
        self.done_works = task_result.done_works;
        self.not_shelved_books
            .extend(task_result.not_shelved_books.clone());
        self.borrowing_books
            .extend(task_result.borrowing_books.clone());
        self.remain_copy_request
            .extend(task_result.remain_copy_request.clone());
        self.total_money = task_result.total_money;

        self
    }

    pub fn reset(&mut self) -> &mut Self {
        self.done_works = 0;
        self.not_shelved_books.clear();
        self.borrowing_books.clear();
        self.remain_copy_request.clear();
        self.total_money = 0;

        self
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
                        self.sub_scene.switch_deskwork_to_shop(transition_status);
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
                debug::debug_screen_push_text("Implement Result!!!!!!!!!!!!!");
            }
            SuzunaSceneStatus::Copying => {}
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
