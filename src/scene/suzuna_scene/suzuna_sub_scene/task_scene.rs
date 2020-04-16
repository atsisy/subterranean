use ginput::mouse::MouseButton;
use torifune::core::*;
use torifune::device as tdev;
use torifune::graphics::drawable::*;
use torifune::graphics::object::Effectable;
use torifune::numeric;

use super::super::*;
use crate::object::Clickable;

use crate::core::{GameData, GensoDate, MouseActionRecord, MouseInformation, TileBatchTextureID, TaskResult};
use crate::object::effect_object;
use crate::object::task_object::*;
use crate::scene::{SceneID, SceneTransition};

use crate::flush_delay_event;
use crate::object::task_object::tt_main_component::*;
use crate::object::task_object::tt_sub_component::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskSceneStatus {
    Init,
    CustomerFree,
    CustomerWait,
    CustomerEvent,
    FinishDay,
}

pub struct TaskScene {
    task_table: TaskTable,
    clock: Clock,
    mouse_info: MouseInformation,
    event_list: DelayEventList<Self>,
    status: TaskSceneStatus,
    task_result: TaskResult,
    customer_request: Option<CustomerRequest>,
    transition_status: SceneTransition,
    transition_scene: SceneID,
    scene_transition_effect: Option<effect_object::ScreenTileEffect>,
}

impl TaskScene {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        today_date: GensoDate,
        customer_request: Option<CustomerRequest>,
        record_book_data: Option<BorrowingRecordBookData>,
    ) -> TaskScene {
        let animation_time = 30;

        let scene_transition_effect = Some(effect_object::ScreenTileEffect::new(
            ctx,
            game_data,
            TileBatchTextureID::Shoji,
            numeric::Rect::new(
                0.0,
                0.0,
                crate::core::WINDOW_SIZE_X as f32,
                crate::core::WINDOW_SIZE_Y as f32,
            ),
            animation_time,
            effect_object::SceneTransitionEffectType::Open,
	    effect_object::TilingEffectType::WholeTile,
            -128,
            0,
        ));

        let mut event_list = DelayEventList::new();
        event_list.add_event(
            Box::new(move |slf: &mut TaskScene, _, _| {
                slf.scene_transition_effect = None;
            }),
            animation_time + 1,
        );

        TaskScene {
            task_table: TaskTable::new(
                ctx,
                game_data,
                numeric::Rect::new(0.0, 0.0, 1366.0, 768.0),
                numeric::Rect::new(0.0, 0.0, 800.0, 300.0),
                numeric::Rect::new(0.0, 310.0, 900.0, 500.0),
                numeric::Rect::new(900.0, 310.0, 500.0, 500.0),
                today_date,
                record_book_data,
                0,
            ),
            clock: 0,
            mouse_info: MouseInformation::new(),
            event_list: event_list,
            status: TaskSceneStatus::Init,
            task_result: TaskResult::new(),
            customer_request: customer_request,
            transition_status: SceneTransition::Keep,
            transition_scene: SceneID::MainDesk,
            scene_transition_effect: scene_transition_effect,
        }
    }

    fn dragging_handler(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
        _offset: numeric::Vector2f,
        _game_data: &GameData,
    ) {
        let last = self.mouse_info.get_last_dragged(MouseButton::Left);
        self.task_table.dragging_handler(point, last);
        self.task_table.hand_over_check(ctx, point);
    }

    fn unselect_dragging_object(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        t: Clock,
    ) {
        self.task_table.unselect_dragging_object(ctx, game_data, t);
    }

    fn insert_customer_event(&mut self, request: CustomerRequest, delay_clock: Clock) {
        self.event_list.add_event(
            Box::new(
                |s: &mut TaskScene, ctx: &mut ggez::Context, game_data: &GameData| {
                    s.task_table.start_customer_event(
                        ctx,
                        game_data,
                        request.clone(),
                        s.get_current_clock(),
                    );
                    s.status = TaskSceneStatus::CustomerEvent;
                    s.check_done_today_work(request);
                },
            ),
            self.get_current_clock() + delay_clock,
        );
    }

    fn check_done_today_work(&mut self, request: CustomerRequest) {
        match request {
            CustomerRequest::Borrowing(request_information) => {
                // 貸出本を記録
                self.task_result.done_works += 1;
                self.task_result.total_money += request_information.calc_fee();
                self.task_result
                    .borrowing_books
                    .extend(request_information.borrowing);
            }
            CustomerRequest::Returning(request_information) => {
                // 貸出本を記録
                self.task_result.done_works += 1;
                self.task_result
                    .not_shelved_books
                    .extend(request_information.returning);
            }
            CustomerRequest::Copying(copying_request_information) => {
                // 写本依頼を記録
                self.task_result.done_works += 1;
                self.task_result
                    .remain_copy_request
                    .push(copying_request_information);
            }
        }
    }

    pub fn get_task_result(&self) -> &TaskResult {
        &self.task_result
    }

    pub fn reset_task_result(&mut self) {
        self.task_result.reset();
    }

    pub fn get_task_status(&self) -> TaskSceneStatus {
        self.status
    }

    pub fn ready_to_finish_scene(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        t: Clock,
    ) {
        self.status = TaskSceneStatus::FinishDay;
        self.event_list.add_event(
            Box::new(move |slf: &mut TaskScene, _, _| {
                slf.transition_scene = SceneID::SuzunaShop;
                slf.transition_status = SceneTransition::PoppingTransition;
            }),
            t + 31,
        );

        self.scene_transition_effect = Some(effect_object::ScreenTileEffect::new(
            ctx,
            game_data,
            TileBatchTextureID::Shoji,
            numeric::Rect::new(
                0.0,
                0.0,
                crate::core::WINDOW_SIZE_X as f32,
                crate::core::WINDOW_SIZE_Y as f32,
            ),
            30,
            effect_object::SceneTransitionEffectType::Close,
	    effect_object::TilingEffectType::WholeTile,
            -128,
            t,
        ));
    }

    pub fn export_borrowing_record_book_data(&self) -> BorrowingRecordBookData {
        self.task_table.export_borrowing_record_book_data()
    }
}

impl SceneManager for TaskScene {
    fn key_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        vkey: tdev::VirtualKey,
    ) {
        self.task_table
            .key_event_handler(ctx, game_data, vkey, self.get_current_clock());
        match vkey {
            tdev::VirtualKey::Action1 => {
                println!("Action1 down!");
                self.event_list.add_event(
                    Box::new(|_, _, _| println!("aaaaaaaaaa")),
                    self.get_current_clock() + 2,
                );
                self.event_list.add_event(
                    Box::new(|_, _, _| println!("bbbbbbbbbb")),
                    self.get_current_clock() + 500,
                );
            }
            _ => (),
        }
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _game_data: &GameData,
        vkey: tdev::VirtualKey,
    ) {
        match vkey {
            tdev::VirtualKey::Action1 => println!("Action1 up!"),
            tdev::VirtualKey::Action2 => {}
            _ => (),
        }
    }

    fn mouse_motion_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        point: numeric::Point2f,
        offset: numeric::Vector2f,
    ) {
        if self.mouse_info.is_dragging(MouseButton::Left) {
            let d = numeric::Vector2f::new(offset.x / 2.0, offset.y / 2.0);
            self.dragging_handler(ctx, point, d, game_data);
            self.mouse_info
                .set_last_dragged(MouseButton::Left, point, self.get_current_clock());
        }

        self.task_table
            .mouse_motion_handler(ctx, game_data, point, offset);

        let mouse_cursor_status = self.task_table.clickable_status(ctx, point);
        ggez::input::mouse::set_cursor_type(ctx, mouse_cursor_status);
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        let info: &MouseActionRecord = &self.mouse_info.last_clicked.get(&button).unwrap();
        if info.point == point {
            if (self.get_current_clock() - info.t) < 30 {
                self.task_table.double_click_handler(
                    ctx,
                    point,
                    game_data,
                    self.get_current_clock(),
                );
            }
        }

        self.mouse_info
            .set_last_clicked(button, point, self.get_current_clock());
        self.mouse_info
            .set_last_down(button, point, self.get_current_clock());
        self.mouse_info
            .set_last_dragged(button, point, self.get_current_clock());
        self.mouse_info.update_dragging(button, true);

        self.task_table
            .button_down(ctx, game_data, self.get_current_clock(), button, point);
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        self.mouse_info.update_dragging(button, false);
        //self.paper.button_up(ctx, button, point);
        self.unselect_dragging_object(ctx, game_data, self.get_current_clock());

        self.task_table
            .button_up(ctx, game_data, self.get_current_clock(), button, point);

        let info: &MouseActionRecord = &self.mouse_info.last_down.get(&button).unwrap();
        if info.point == point {
            self.task_table
                .on_click(ctx, game_data, self.get_current_clock(), button, point);
        }

        self.mouse_info
            .set_last_up(button, point, self.get_current_clock());
    }

    fn pre_process(&mut self, ctx: &mut ggez::Context, game_data: &GameData) {
        let t = self.get_current_clock();
        self.task_table
            .update(ctx, game_data, self.get_current_clock());

        if (self.status == TaskSceneStatus::CustomerEvent || self.status == TaskSceneStatus::Init)
            && self.task_table.get_remaining_customer_object_number() == 0
        {
            debug::debug_screen_push_text(&format!("register delay process!!"));
            self.event_list.add_event(
                Box::new(|scene: &mut TaskScene, _, _| {
                    scene
                        .task_table
                        .finish_customer_event(scene.get_current_clock());
                    debug::debug_screen_push_text(&format!("run delay process!! finish!!"));
                    scene.status = TaskSceneStatus::CustomerFree;
                }),
                self.get_current_clock() + 30,
            );

            self.status = TaskSceneStatus::CustomerWait;

            if self.customer_request.is_none() {
                self.event_list.add_event(
                    Box::new(move |scene: &mut TaskScene, ctx, game_data| {
                        scene.ready_to_finish_scene(ctx, game_data, scene.get_current_clock());
                    }),
                    t + 150,
                );
            }
        }

        if self.status == TaskSceneStatus::CustomerFree {
            if let Some(request) = &self.customer_request {
                let cloned_request = request.clone();
                self.insert_customer_event(cloned_request, 100);
            }
            self.customer_request = None;
            self.status = TaskSceneStatus::CustomerWait;
        }

        if let Some(transition_effect) = self.scene_transition_effect.as_mut() {
            transition_effect.effect(ctx, t);
        }

        flush_delay_event!(
            self,
            self.event_list,
            ctx,
            game_data,
            self.get_current_clock()
        );
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.task_table.draw(ctx).unwrap();

        if let Some(transition_effect) = self.scene_transition_effect.as_mut() {
            transition_effect.draw(ctx).unwrap();
        }
    }

    fn post_process(&mut self, _ctx: &mut ggez::Context, _: &GameData) -> SceneTransition {
        self.update_current_clock();
        self.transition_status
    }

    fn transition(&self) -> SceneID {
        self.transition_scene
    }

    fn get_current_clock(&self) -> Clock {
        self.clock
    }

    fn update_current_clock(&mut self) {
        self.clock += 1;
    }
}
