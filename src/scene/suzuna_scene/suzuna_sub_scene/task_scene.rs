use ginput::mouse::MouseButton;
use torifune::core::*;
use torifune::device as tdev;
use torifune::graphics::drawable::*;
use torifune::graphics::object::Effectable;
use torifune::numeric;

use super::super::*;
use crate::object::{Clickable, scenario::ScenarioEvent};

use crate::core::{MouseActionRecord, MouseInformation, ReputationEvent, TileBatchTextureID};
use crate::object::effect_object;
use crate::object::task_object::*;
use crate::object::util_object::*;
use crate::scene::{SceneID, SceneTransition};

use crate::perf_measure;

use crate::flush_delay_event;
use crate::object::task_object::tt_main_component::*;
use crate::{flush_delay_event_and_redraw_check, object::task_object::tt_sub_component::*};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskSceneStatus {
    CustomerFree,
    CustomerWait,
    CustomerEvent,
    FinishDay,
}

pub struct TaskScene {
    task_table: TaskTable,
    clock: Clock,
    pause_screen_set: PauseScreenSet,
    mouse_info: MouseInformation,
    event_list: DelayEventList<Self>,
    status: TaskSceneStatus,
    customer_request: Option<CustomerRequest>,
    transition_status: SceneTransition,
    transition_scene: SceneID,
    scene_transition_effect: Option<effect_object::ScreenTileEffect>,
    tutorial_context: TutorialContext,
    scenario_event: Option<ScenarioEvent>,
}

impl TaskScene {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        customer_request: Option<CustomerRequest>,
        record_book_data: BorrowingRecordBookData,
	tutorial_context: &TutorialContext,
    ) -> TaskScene {
        let animation_time = 30;

        let scene_transition_effect = Some(effect_object::ScreenTileEffect::new(
            ctx,
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

        if let Some(customer_request) = customer_request.as_ref() {
            match customer_request {
                CustomerRequest::Borrowing(_) => {
		    ctx.take_save_data_mut().award_data.borrowing_count += 1;
		    if !tutorial_context.borrowing_request {
			event_list.add_event(
			    Box::new(move |slf: &mut TaskScene, ctx, t| {
				slf.set_fixed_text_into_scenario_box(ctx, "/scenario/tutorial/task/b1.toml", t);
			    }),
			    31
			);
		    }
		},
                CustomerRequest::Returning(_) => {
		    ctx.take_save_data_mut().award_data.returning_count += 1;
		    if !tutorial_context.returning_request {
			event_list.add_event(
			    Box::new(move |slf: &mut TaskScene, ctx, t| {
				slf.set_fixed_text_into_scenario_box(ctx, "/scenario/tutorial/task/r1.toml", t);
			    }),
			    31
			);
		    }
		}
            }
        }

        TaskScene {
            task_table: TaskTable::new(
                ctx,
                numeric::Rect::new(0.0, 0.0, 1366.0, 768.0),
                numeric::Rect::new(0.0, 0.0, 300.0, 768.0),
                numeric::Rect::new(300.0, 0.0, 1066.0, 300.0),
                numeric::Rect::new(300.0, 300.0, 766.0, 468.0),
                numeric::Rect::new(1066.0, 300.0, 300.0, 468.0),
                record_book_data,
                customer_request.clone(),
                0,
            ),
            clock: 0,
            pause_screen_set: PauseScreenSet::new(ctx, 0, 0),
            mouse_info: MouseInformation::new(),
            event_list: event_list,
            status: TaskSceneStatus::CustomerFree,
            customer_request: customer_request,
            transition_status: SceneTransition::Keep,
            transition_scene: SceneID::MainDesk,
            scene_transition_effect: scene_transition_effect,
	    tutorial_context: tutorial_context.clone(),
	    scenario_event: None,
        }
    }

    fn dragging_handler<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        _offset: numeric::Vector2f,
    ) {
        let last = self.mouse_info.get_last_dragged(MouseButton::Left);
        self.task_table.dragging_handler(ctx, point, last);
        self.task_table.hand_over_check(ctx.context, point);
    }

    fn unselect_dragging_object<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        self.task_table.unselect_dragging_object(ctx, t);
    }

    fn insert_customer_event(&mut self, request: CustomerRequest, delay_clock: Clock) {
        self.event_list.add_event(
            Box::new(|s: &mut TaskScene, ctx, _| {
                s.task_table
                    .start_customer_event(ctx, request.clone(), s.get_current_clock());
                s.status = TaskSceneStatus::CustomerEvent;
                s.check_done_today_work(ctx, request);
            }),
            self.get_current_clock() + delay_clock,
        );
    }

    fn check_done_today_work<'a>(&mut self, ctx: &mut SuzuContext<'a>, request: CustomerRequest) {
        let task_result = &mut ctx.take_save_data_mut().task_result;
        match request {
            CustomerRequest::Borrowing(request_information) => {
                // 貸出本を記録
                task_result.done_works += 1;
                task_result.total_money += request_information.calc_fee();
                task_result
                    .borrowing_books
                    .extend(request_information.borrowing);
            }
            CustomerRequest::Returning(request_information) => {
                // 貸出本を記録
                task_result.done_works += 1;
                task_result
                    .not_shelved_books
                    .extend(request_information.returning);
            }
        }
    }

    pub fn get_task_status(&self) -> TaskSceneStatus {
        self.status
    }

    pub fn ready_to_finish_scene<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        self.status = TaskSceneStatus::FinishDay;
        self.event_list.add_event(
            Box::new(move |slf: &mut TaskScene, _, _| {
                slf.transition_scene = SceneID::SuzunaShop;
                slf.transition_status = SceneTransition::PoppingTransition;
            }),
            t + 31,
        );

        self.scene_transition_close_effect(ctx, t);

        ctx.take_save_data_mut()
            .suzunaan_status
            .eval_reputation(ReputationEvent::DoneDeskTask);
    }

    pub fn export_borrowing_record_book_data(&self) -> BorrowingRecordBookData {
        self.task_table.export_borrowing_record_book_data()
    }

    pub fn get_tutorial_context(&self) -> &TutorialContext {
	&self.tutorial_context
    }

    fn scene_transition_close_effect<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        self.scene_transition_effect = Some(effect_object::ScreenTileEffect::new(
            ctx,
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

    fn transition_to_title_scene<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        self.event_list.add_event(
            Box::new(|slf: &mut Self, _, _| {
                slf.transition_status = SceneTransition::SwapTransition;
                slf.transition_scene = SceneID::Title;
            }),
            t + 60,
        );
        self.scene_transition_close_effect(ctx, t);
    }

    fn exit_pause_screen(&mut self, t: Clock) {
        self.pause_screen_set.exit_pause(t);
    }

    fn enter_pause_screen<'a>(&mut self, t: Clock) {
        self.pause_screen_set.enter_pause(t);
    }

    fn now_paused(&self) -> bool {
        self.pause_screen_set.is_paused_now()
    }

    pub fn get_elapsed_clock(&self) -> Clock {
        self.get_current_clock()
    }

    fn pause_screen_click_handler<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        t: Clock,
    ) {
        if !self.pause_screen_set.is_paused_now() {
            return;
        }

        if let Some(pause_result) = self.pause_screen_set.mouse_click_handler(ctx, point, t) {
            match pause_result {
                PauseResult::GoToTitle => self.transition_to_title_scene(ctx, t),
                PauseResult::ReleasePause => self.exit_pause_screen(t),
            }
        }
    }

    fn non_paused_mouse_button_up_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: MouseButton,
        point: numeric::Point2f,
	t: Clock,
    ) {
        //self.paper.button_up(ctx, button, point);
        self.unselect_dragging_object(ctx, self.get_current_clock());

        self.task_table
            .button_up(ctx, self.get_current_clock(), button, point);

        let info: &MouseActionRecord = &self.mouse_info.last_down.get(&button).unwrap();
        if info.point == point {
            self.task_table
                .on_click(ctx, self.get_current_clock(), button, point);
        }

        self.mouse_info
            .set_last_up(button, point, self.get_current_clock());

	if let Some(scenario_event) = self.scenario_event.as_mut() {
	    if scenario_event.contains_scenario_text_box(point) {
                scenario_event
		    .key_down_action1(ctx, Some(point), t);
	    }
	}
    }

    fn non_paused_mouse_button_down_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        self.mouse_info
            .set_last_clicked(button, point, self.get_current_clock());
        self.mouse_info
            .set_last_down(button, point, self.get_current_clock());
        self.mouse_info
            .set_last_dragged(button, point, self.get_current_clock());

        self.task_table
            .button_down(ctx, self.get_current_clock(), button, point);
    }

    pub fn get_target_page_book_condition_eval_report(&self) -> Option<BookConditionEvalReport> {
        if let Some(report) = self.task_table.get_target_page_book_condition_eval_report() {
            Some(report.clone())
        } else {
            None
        }
    }

    fn after_task_done_process<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
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
                Box::new(move |scene: &mut TaskScene, ctx, _| {
                    scene.ready_to_finish_scene(ctx, scene.get_current_clock());
                }),
                t + 150,
            );
        }

        ctx.process_utility.redraw();
    }

    fn scenario_event_handler<'a>(&mut self, _ctx: &mut SuzuContext<'a>, _t: Clock) {
	if let Some(scenario_event) = self.scenario_event.as_ref() {
	    if let Some(opecode) = scenario_event.get_scenario_waiting_opecode() {
		match opecode {
		    "TutorialFinishBorrowing" => {
			self.scenario_event = None;
			self.tutorial_context.borrowing_request = true;
		    },
		    "TutorialFinishReturning" => {
			self.scenario_event = None;
			self.tutorial_context.returning_request = true;
		    },
		    _ => (),
		}
	    }
	}
    }

    fn set_fixed_text_into_scenario_box<'a>(&mut self, ctx: &mut SuzuContext<'a>, path: &str, t: Clock) {
	let scenario_box = ScenarioEvent::new(ctx, numeric::Rect::new(0.0, 0.0, 1366.0, 748.0), path, t);
        self.scenario_event = Some(scenario_box);
    }
}

impl SceneManager for TaskScene {
    fn key_down_event<'a>(&mut self, ctx: &mut SuzuContext<'a>, vkey: tdev::VirtualKey) {
        if self.now_paused() {
            match vkey {
                tdev::VirtualKey::Action4 => {
                    let t = self.get_current_clock();
                    self.exit_pause_screen(t);
                }
                _ => (),
            }
        } else {
            match vkey {
                tdev::VirtualKey::Action4 => {
                    let t = self.get_current_clock();
                    self.enter_pause_screen(t);
                }
                _ => (),
            }
            self.task_table
                .key_event_handler(ctx, vkey, self.get_current_clock());
        }
    }

    fn mouse_motion_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        offset: numeric::Vector2f,
    ) {
        let t = self.get_current_clock();

        if self.now_paused() {
            if self.pause_screen_set.is_paused_now() {
                if self.mouse_info.is_dragging(MouseButton::Left) {
                    self.pause_screen_set
                        .dragging_handler(ctx, MouseButton::Left, point, t);
                } else {
                    self.pause_screen_set.mouse_motion_handler(ctx, point);
                }
            }
        } else {
            if self.mouse_info.is_dragging(MouseButton::Left) {
                let d = numeric::Vector2f::new(offset.x / 2.0, offset.y / 2.0);
                self.dragging_handler(ctx, point, d);
                self.mouse_info.set_last_dragged(
                    MouseButton::Left,
                    point,
                    self.get_current_clock(),
                );
            }

            self.task_table.mouse_motion_handler(ctx, point, offset);

            let mouse_cursor_status = self.task_table.clickable_status(ctx.context, point);
            ggez::input::mouse::set_cursor_type(ctx.context, mouse_cursor_status);
        }
    }

    fn mouse_wheel_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        x: f32,
        y: f32,
    ) {
        println!("wheel");
        self.task_table.mouse_wheel_event(ctx, point, x, y);
    }

    fn mouse_button_down_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        self.mouse_info.update_dragging(button, true);

        if self.now_paused() {
            let t = self.get_current_clock();
            if self.pause_screen_set.is_paused_now() {
                self.pause_screen_set
                    .mouse_button_down(ctx, button, point, t);
            }
        } else {
            self.non_paused_mouse_button_down_event(ctx, button, point);
        }
    }

    fn mouse_button_up_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
	let t = self.get_current_clock();
        self.mouse_info.update_dragging(button, false);

        if self.now_paused() {
            match button {
                MouseButton::Left => {
                    self.pause_screen_click_handler(ctx, point, t);
                }
                _ => (),
            }
        } else {
            self.non_paused_mouse_button_up_event(ctx, button, point, t);
        }
    }

    fn pre_process<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        //println!("{}", perf_measure!({
        let t = self.get_current_clock();

        if !self.now_paused() {
	    if let Some(scenario_event) = self.scenario_event.as_mut() {
		scenario_event.update_text(ctx, None);
	    }
	    self.scenario_event_handler(ctx, t);
	    
            self.task_table.update(ctx, self.get_current_clock());

            if self.status == TaskSceneStatus::CustomerEvent && self.task_table.task_is_done() {
                self.after_task_done_process(ctx, t);
            }

            if self.status == TaskSceneStatus::CustomerFree {
                if let Some(request) = &self.customer_request {
                    let cloned_request = request.clone();
                    self.insert_customer_event(cloned_request, 100);
                }
                self.customer_request = None;
                self.status = TaskSceneStatus::CustomerWait;

                ctx.process_utility.redraw();
            }
        }

        self.pause_screen_set.effect(ctx, t);

        if let Some(transition_effect) = self.scene_transition_effect.as_mut() {
            transition_effect.effect(ctx.context, t);
            ctx.process_utility.redraw();
        }

        flush_delay_event_and_redraw_check!(
            self,
            self.event_list,
            ctx,
            self.get_current_clock(),
            { () }
        );
        //}));
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        //println!("{}", perf_measure!(
        {
            self.task_table.draw(ctx).unwrap();
            self.pause_screen_set.draw(ctx).unwrap();

	    if let Some(scenario_box) = self.scenario_event.as_mut() {
		scenario_box.draw(ctx).unwrap();
            }

            if let Some(transition_effect) = self.scene_transition_effect.as_mut() {
                transition_effect.draw(ctx).unwrap();
            }
        } //));
    }

    fn post_process<'a>(&mut self, _ctx: &mut SuzuContext<'a>) -> SceneTransition {
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
