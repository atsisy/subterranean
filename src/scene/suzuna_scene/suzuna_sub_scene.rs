pub mod task_result_scene;
pub mod task_scene;

use ggez::input::mouse::MouseButton;
use torifune::core::*;
use torifune::device::VirtualKey;
use torifune::numeric;

use crate::core::book_management::*;
use crate::core::*;
use crate::scene::*;

use crate::scene::shop_scene::ShopScene;

use task_result_scene::*;
use task_scene::*;

use crate::object::task_object::tt_main_component::*;

#[derive(PartialEq, Clone, Copy)]
pub enum SuzunaSceneStatus {
    Shop,
    DeskWork,
    DayResult,
}

#[derive(Clone)]
pub struct TaskTutorialContext {
    borrowing_request: bool,
    returning_request: bool,
}

impl TaskTutorialContext {
    pub fn new() -> Self {
	TaskTutorialContext {
	    borrowing_request: false,
	    returning_request: false,
	}
    }

    pub fn new_done() -> Self {
	TaskTutorialContext {
	    borrowing_request: true,
	    returning_request: true,
	}
    }

    pub fn all_done(&self) -> bool {
	self.borrowing_request && self.returning_request
    }
}

///
/// 鈴奈庵シーンのサブシーンをまとめる構造体
///
pub struct SuzunaSubScene {
    pub shop_scene: Option<Box<ShopScene>>,
    pub desk_work_scene: Option<Box<TaskScene>>,
    pub day_result_scene: Option<Box<TaskResultScene>>,
    scene_status: SuzunaSceneStatus,
    new_book_schedule: NewBookSchedule,
    tutorial_context: TaskTutorialContext,
    date: GensoDate,
}

impl SuzunaSubScene {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, map_id: u32) -> Self {
        let date = ctx.take_save_data().date.clone();

        let new_book_schedule =
            NewBookSchedule::from_toml(ctx, "/other_config/new_book_schedule.toml");

        let todays_new_books = if let Some(sched) = new_book_schedule.get_schedule_at(&date) {
	    sched.clone()
	} else {
	    new_book_schedule.get_schedule_at(&GensoDate::new(112, 7, 23)).unwrap().clone()
	};

	let task_tutorial = if ctx.take_save_data().date.first_day() && ctx.take_save_data().game_mode.is_story_mode() {
	    TaskTutorialContext::new()
	} else {
	    TaskTutorialContext::new_done()
	};

        SuzunaSubScene {
            shop_scene: Some(Box::new(ShopScene::new(
                ctx,
                map_id,
                todays_new_books.get_new_books(),
		task_tutorial.clone()
            ))),
            desk_work_scene: None,
            day_result_scene: None,
            scene_status: SuzunaSceneStatus::Shop,
            new_book_schedule: new_book_schedule,
            date: date,
	    tutorial_context: task_tutorial,
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

    pub fn switch_shop_to_deskwork<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        transition: SceneTransition,
    ) {
        if transition == SceneTransition::StackingTransition {
            if let Some(shop_scene) = self.shop_scene.as_mut() {
                // CustomerRequestを構築する上で必須な要素を取得
                let customer_request_hint = shop_scene.pop_customer_request();
                if customer_request_hint.is_none() {
                    return ();
                }

                // 今回のTaskSceneで扱われるCustomerRequestを構築
                let customer_request = match customer_request_hint.as_ref().unwrap() {
                    CustomerRequest::Borrowing(raw_info) => {
                        let borrowing_info = ctx
                            .take_save_data_mut()
                            .suzuna_book_pool
                            .generate_borrowing_request(
                                &raw_info.borrower,
                                raw_info.borrow_date,
                                raw_info.rental_limit.clone(),
                            );

                        CustomerRequest::Borrowing(borrowing_info)
                    }
                    CustomerRequest::Returning(_) => {
                        let request = ctx
                            .take_save_data()
                            .record_book_data
                            .pick_returning_request_up()
                            .unwrap();
                        println!("returning count: {}", request.returning.len());
                        CustomerRequest::Returning(request)
                    }
                };

                let record_book_data = ctx.take_save_data().record_book_data.clone();

                self.scene_status = SuzunaSceneStatus::DeskWork;
                self.desk_work_scene = Some(Box::new(TaskScene::new(
                    ctx,
                    Some(customer_request),
                    record_book_data,
		    &self.tutorial_context,
                )));
            }
        }
    }

    pub fn switch_shop_to_day_result<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        transition: SceneTransition,
    ) {
        if transition == SceneTransition::SwapTransition {
            if let Some(shop_scene) = self.shop_scene.as_ref() {
                let init_data = shop_scene.clone_begning_save_data();
                let result_report = shop_scene.clone_result_report();

                self.scene_status = SuzunaSceneStatus::DayResult;
                self.day_result_scene = Some(Box::new(TaskResultScene::new(
                    ctx,
                    init_data,
                    result_report,
                    self.date.clone(),
                )));
            }
        }
    }

    pub fn switch_deskwork_to_shop<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        transition: SceneTransition,
    ) {
        if transition == SceneTransition::PoppingTransition {
            println!("switch!!!!!!!!!, deskwork -> shop");
            ctx.take_save_data_mut().record_book_data = self
                .desk_work_scene
                .as_ref()
                .unwrap()
                .export_borrowing_record_book_data();
	    self.tutorial_context = self.desk_work_scene.as_ref().unwrap().get_tutorial_context().clone();
            self.scene_status = SuzunaSceneStatus::Shop;
            self.shop_scene.as_mut().unwrap().switched_and_restart(
                ctx,
                self.desk_work_scene.as_ref().unwrap().get_elapsed_clock(),
                self.desk_work_scene
                    .as_ref()
                    .unwrap()
                    .get_target_page_book_condition_eval_report(),
		self.tutorial_context.clone(),
            );
            self.desk_work_scene = None;
        }
    }
}

impl SceneManager for SuzunaSubScene {
    fn key_down_event<'a>(&mut self, ctx: &mut SuzuContext<'a>, vkey: VirtualKey) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene.as_mut().unwrap().key_down_event(ctx, vkey);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .key_down_event(ctx, vkey);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .key_down_event(ctx, vkey);
            }
        }
    }

    fn key_up_event<'a>(&mut self, ctx: &mut SuzuContext<'a>, vkey: VirtualKey) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene.as_mut().unwrap().key_up_event(ctx, vkey);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .key_up_event(ctx, vkey);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .key_up_event(ctx, vkey);
            }
        }
    }

    fn mouse_motion_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        offset: numeric::Vector2f,
    ) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene
                    .as_mut()
                    .unwrap()
                    .mouse_motion_event(ctx, point, offset);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .mouse_motion_event(ctx, point, offset);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .mouse_motion_event(ctx, point, offset);
            }
        }
    }

    fn mouse_button_down_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_down_event(ctx, button, point);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_down_event(ctx, button, point);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_down_event(ctx, button, point);
            }
        }
    }

    fn mouse_button_up_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_up_event(ctx, button, point);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_up_event(ctx, button, point);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_up_event(ctx, button, point);
            }
        }
    }

    fn mouse_wheel_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        x: f32,
        y: f32,
    ) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene
                    .as_mut()
                    .unwrap()
                    .mouse_wheel_event(ctx, point, x, y);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .mouse_wheel_event(ctx, point, x, y);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .mouse_wheel_event(ctx, point, x, y);
            }
        }
    }

    fn pre_process<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene.as_mut().unwrap().pre_process(ctx);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene.as_mut().unwrap().pre_process(ctx);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene.as_mut().unwrap().pre_process(ctx);
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
        }
    }

    fn post_process<'a>(&mut self, ctx: &mut SuzuContext<'a>) -> SceneTransition {
        match self.scene_status {
            SuzunaSceneStatus::Shop => self.shop_scene.as_mut().unwrap().post_process(ctx),
            SuzunaSceneStatus::DeskWork => self.desk_work_scene.as_mut().unwrap().post_process(ctx),
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene.as_mut().unwrap().post_process(ctx)
            }
        }
    }

    fn transition(&self) -> SceneID {
        match self.scene_status {
            SuzunaSceneStatus::Shop => self.shop_scene.as_ref().unwrap().transition(),
            SuzunaSceneStatus::DeskWork => self.desk_work_scene.as_ref().unwrap().transition(),
            SuzunaSceneStatus::DayResult => self.day_result_scene.as_ref().unwrap().transition(),
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
        }
    }

    fn unfocus_event<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => self.shop_scene.as_mut().unwrap().unfocus_event(ctx),
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene.as_mut().unwrap().unfocus_event(ctx)
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene.as_mut().unwrap().unfocus_event(ctx)
            }
        }
    }
}
