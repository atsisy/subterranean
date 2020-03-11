pub mod copy_scene;
pub mod task_result_scene;
pub mod task_scene;

use ggez::input::mouse::MouseButton;
use torifune::core::*;
use torifune::device::VirtualKey;
use torifune::numeric;

use crate::core::GameData;
use crate::scene::*;

use crate::scene::shop_scene::ShopScene;

use copy_scene::*;
use task_result_scene::*;
use task_scene::*;

#[derive(PartialEq, Clone, Copy)]
pub enum SuzunaSceneStatus {
    Shop,
    DeskWork,
    DayResult,
    Copying,
}

///
/// 鈴奈庵シーンのサブシーンをまとめる構造体
///
pub struct SuzunaSubScene {
    pub shop_scene: Option<Box<ShopScene>>,
    pub desk_work_scene: Option<Box<TaskScene>>,
    pub day_result_scene: Option<Box<TaskResultScene>>,
    pub copying_scene: Option<Box<CopyingScene>>,
    scene_status: SuzunaSceneStatus,
}

impl SuzunaSubScene {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData, map_id: u32) -> Self {
        SuzunaSubScene {
            shop_scene: Some(Box::new(ShopScene::new(ctx, game_data, map_id))),
            desk_work_scene: None,
            day_result_scene: None,
            copying_scene: None,
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

    pub fn switch_shop_to_deskwork(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        transition: SceneTransition,
    ) {
        if transition == SceneTransition::StackingTransition {
            let customer_request = self.shop_scene.as_mut().unwrap().pop_customer_request();
            self.scene_status = SuzunaSceneStatus::DeskWork;
            self.desk_work_scene = Some(Box::new(TaskScene::new(ctx, game_data, customer_request)));
        }
    }

    pub fn switch_shop_to_day_result(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        transition: SceneTransition,
    ) {
        if transition == SceneTransition::SwapTransition {
            let task_result = self.shop_scene.as_mut().unwrap().current_task_result();
            self.scene_status = SuzunaSceneStatus::DayResult;
            self.day_result_scene =
                Some(Box::new(TaskResultScene::new(ctx, game_data, task_result)));
        }
    }

    pub fn switch_deskwork_to_shop(&mut self, transition: SceneTransition) {
        if transition == SceneTransition::PoppingTransition {
            self.scene_status = SuzunaSceneStatus::Shop;
            self.desk_work_scene = None;
            self.shop_scene.as_mut().unwrap().switched_and_restart();
        }
    }

    pub fn switch_shop_to_copying(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        transition: SceneTransition,
    ) {
        if transition == SceneTransition::StackingTransition {
            self.scene_status = SuzunaSceneStatus::Copying;
            self.copying_scene = Some(Box::new(CopyingScene::new(ctx, game_data, Vec::new())));
        }
    }
}

impl SceneManager for SuzunaSubScene {
    fn key_down_event(&mut self, ctx: &mut ggez::Context, game_data: &GameData, vkey: VirtualKey) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene
                    .as_mut()
                    .unwrap()
                    .key_down_event(ctx, game_data, vkey);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .key_down_event(ctx, game_data, vkey);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .key_down_event(ctx, game_data, vkey);
            }
            SuzunaSceneStatus::Copying => {
                self.copying_scene
                    .as_mut()
                    .unwrap()
                    .key_down_event(ctx, game_data, vkey);
            }
        }
    }

    fn key_up_event(&mut self, ctx: &mut ggez::Context, game_data: &GameData, vkey: VirtualKey) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene
                    .as_mut()
                    .unwrap()
                    .key_up_event(ctx, game_data, vkey);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .key_up_event(ctx, game_data, vkey);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .key_up_event(ctx, game_data, vkey);
            }
            SuzunaSceneStatus::Copying => {
                self.copying_scene
                    .as_mut()
                    .unwrap()
                    .key_up_event(ctx, game_data, vkey);
            }
        }
    }

    fn mouse_motion_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        point: numeric::Point2f,
        offset: numeric::Vector2f,
    ) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene
                    .as_mut()
                    .unwrap()
                    .mouse_motion_event(ctx, game_data, point, offset);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .mouse_motion_event(ctx, game_data, point, offset);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .mouse_motion_event(ctx, game_data, point, offset);
            }
            SuzunaSceneStatus::Copying => {
                self.copying_scene
                    .as_mut()
                    .unwrap()
                    .mouse_motion_event(ctx, game_data, point, offset);
            }
        }
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_down_event(ctx, game_data, button, point);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_down_event(ctx, game_data, button, point);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_down_event(ctx, game_data, button, point);
            }
            SuzunaSceneStatus::Copying => {
                self.copying_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_down_event(ctx, game_data, button, point);
            }
        }
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_up_event(ctx, game_data, button, point);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_up_event(ctx, game_data, button, point);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_up_event(ctx, game_data, button, point);
            }
            SuzunaSceneStatus::Copying => {
                self.copying_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_up_event(ctx, game_data, button, point);
            }
        }
    }

    fn pre_process(&mut self, ctx: &mut ggez::Context, game_data: &GameData) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene
                    .as_mut()
                    .unwrap()
                    .pre_process(ctx, game_data);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .pre_process(ctx, game_data);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .pre_process(ctx, game_data);
            }
            SuzunaSceneStatus::Copying => {
                self.copying_scene
                    .as_mut()
                    .unwrap()
                    .pre_process(ctx, game_data);
            }
        }
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene.as_mut().unwrap().drawing_process(ctx);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene.as_mut().unwrap().drawing_process(ctx);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene.as_mut().unwrap().drawing_process(ctx);
            }
            SuzunaSceneStatus::Copying => {
                self.copying_scene.as_mut().unwrap().drawing_process(ctx);
            }
        }
    }

    fn post_process(&mut self, ctx: &mut ggez::Context, game_data: &GameData) -> SceneTransition {
        match self.scene_status {
            SuzunaSceneStatus::Shop => self
                .shop_scene
                .as_mut()
                .unwrap()
                .post_process(ctx, game_data),
            SuzunaSceneStatus::DeskWork => self
                .desk_work_scene
                .as_mut()
                .unwrap()
                .post_process(ctx, game_data),
            SuzunaSceneStatus::DayResult => self
                .day_result_scene
                .as_mut()
                .unwrap()
                .post_process(ctx, game_data),
            SuzunaSceneStatus::Copying => self
                .copying_scene
                .as_mut()
                .unwrap()
                .post_process(ctx, game_data),
        }
    }

    fn transition(&self) -> SceneID {
        match self.scene_status {
            SuzunaSceneStatus::Shop => self.shop_scene.as_ref().unwrap().transition(),
            SuzunaSceneStatus::DeskWork => self.desk_work_scene.as_ref().unwrap().transition(),
            SuzunaSceneStatus::DayResult => self.day_result_scene.as_ref().unwrap().transition(),
            SuzunaSceneStatus::Copying => self.copying_scene.as_ref().unwrap().transition(),
        }
    }

    fn get_current_clock(&self) -> Clock {
        match self.scene_status {
            SuzunaSceneStatus::Shop => self.shop_scene.as_ref().unwrap().get_current_clock(),
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene.as_ref().unwrap().get_current_clock()
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene.as_ref().unwrap().get_current_clock()
            }
            SuzunaSceneStatus::Copying => self.copying_scene.as_ref().unwrap().get_current_clock(),
        }
    }

    fn update_current_clock(&mut self) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => self.shop_scene.as_mut().unwrap().update_current_clock(),
            SuzunaSceneStatus::DeskWork => self
                .desk_work_scene
                .as_mut()
                .unwrap()
                .update_current_clock(),
            SuzunaSceneStatus::DayResult => self
                .day_result_scene
                .as_mut()
                .unwrap()
                .update_current_clock(),
            SuzunaSceneStatus::Copying => {
                self.copying_scene.as_mut().unwrap().update_current_clock()
            }
        }
    }
}
