use ggez::graphics as ggraphics;
use ggez::input::mouse::MouseButton;

use torifune::core::Clock;
use torifune::device as tdev;
use torifune::graphics::object::Effectable;
use torifune::numeric;

use crate::core::{GameMode, GeneralScenarioID, MouseInformation, SuzuContext, TileBatchTextureID};

use crate::add_delay_event;
use crate::core::game_system::*;
use crate::flush_delay_event;
use crate::flush_delay_event_and_redraw_check;
use crate::object::effect_object;
use crate::object::scenario::*;
use crate::object::scenario_object::*;
use crate::object::util_object::*;
use crate::perf_measure;
use effect_object::{SceneTransitionEffectType, TilingEffectType};
use torifune::graphics::drawable::*;

use super::*;

#[derive(Clone)]
pub enum ScenarioSelect {
    DayBegin = 0,
    DayEnd,
    OpeningEpisode,
}

pub struct ScenarioContext {
    pub schedule_redefine: bool,
    pub scenario_is_finish_and_wait: bool,
    pub wait_opecode_running: bool,
    pub schedule_define_done: bool,
    pub builtin_command_inexec: bool,
}

pub struct ScenarioScene {
    mouse_info: MouseInformation,
    scenario_event: ScenarioEvent,
    pause_screen_set: PauseScreenSet,
    graph_sample: GraphDrawer,
    event_list: DelayEventList<Self>,
    status_screen: SuzunaStatusScreen,
    scene_transition_type: SceneTransition,
    scene_transition_effect: Option<effect_object::ScreenTileEffect>,
    scene_transition: SceneID,
    scenario_ctx: ScenarioContext,
    clock: Clock,
}

impl ScenarioScene {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, scenario_select: ScenarioSelect) -> Self {
        let file_path = match scenario_select {
            ScenarioSelect::DayBegin => {
		match ctx.take_save_data().game_mode {
		    GameMode::Story => 		ctx
			.resource
			.get_day_scenario_path(&ctx.take_save_data().date)
			.expect("BUG"),
		    GameMode::TimeAttack(_) => {
			"/scenario/time_attack_default.toml".to_string()
		    }
		}
	    },
            ScenarioSelect::OpeningEpisode => panic!(""),
            _ => panic!(""),
        };

        let scenario = ScenarioEvent::new(
            ctx,
            numeric::Rect::new(0.0, 0.0, 1366.0, 768.0),
            &file_path,
            0,
        );

        let graph_drawer = GraphDrawer::new(
            ctx,
            numeric::Rect::new(300.0, 100.0, 700.0, 600.0),
            numeric::Rect::new(20.0, 20.0, 660.0, 560.0),
            vec![
                numeric::Vector2f::new(0.0, 0.0),
                numeric::Vector2f::new(10.0, 10.0),
                numeric::Vector2f::new(20.0, 20.0),
                numeric::Vector2f::new(50.0, 50.0),
            ],
            6.0,
            ggraphics::Color::from_rgba_u32(0x00ff00ff),
            2.0,
            ggraphics::Color::from_rgba_u32(0xff),
            0,
        );

        let scenario_ctx = ScenarioContext {
            schedule_redefine: !ctx.holding_week_schedule_is_available(),
            scenario_is_finish_and_wait: false,
            wait_opecode_running: false,
            schedule_define_done: false,
            builtin_command_inexec: false,
        };

        let mut status_screen = SuzunaStatusScreen::new(
            ctx,
            &scenario_ctx,
            numeric::Rect::new(30.0, 25.0, 700.0, 470.0),
            0,
        );
        status_screen.hide();

        ScenarioScene {
            mouse_info: MouseInformation::new(),
            scenario_event: scenario,
            pause_screen_set: PauseScreenSet::new(ctx, 0, 0),
            scene_transition_effect: None,
            event_list: DelayEventList::new(),
            graph_sample: graph_drawer,
            scene_transition: SceneID::Scenario,
            status_screen: status_screen,
            scene_transition_type: SceneTransition::Keep,
            scenario_ctx: scenario_ctx,
            clock: 0,
        }
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
            60,
            SceneTransitionEffectType::Close,
            TilingEffectType::WholeTile,
            -128,
            t,
        ));
    }

    fn transition_to_title_scene<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        self.event_list.add_event(
            Box::new(|slf: &mut Self, _, _| {
                slf.scene_transition_type = SceneTransition::SwapTransition;
                slf.scene_transition = SceneID::Title;
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

    fn non_paused_key_down_event(&mut self, ctx: &mut SuzuContext, vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => {
                println!("Action1 down!");
                self.scenario_event
                    .key_down_action1(ctx, self.get_current_clock());
            }
            tdev::VirtualKey::Action4 => {
                let t = self.get_current_clock();
                self.enter_pause_screen(t);
            }
            _ => (),
        }
    }

    fn schedule_check<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        if !self.scenario_ctx.wait_opecode_running {
            if let Some(opecode) = self.scenario_event.get_scenario_waiting_opecode() {
                self.scenario_ctx.wait_opecode_running = true;
                match opecode {
                    "ScheduleCheck" => {
                        self.status_screen.show_schedule_page();
                        self.scenario_event.set_fixed_text_to_scenario_box(
                            ctx,
                            if self.scenario_ctx.schedule_redefine {
                                "新しく計画を建てるわヨ"
                            } else {
                                "計画通り働くわよ"
                            },
                        );
                    }
                    "ShowSchedule" => {
                        self.status_screen.show_schedule_page();
                        self.scenario_event.release_scenario_waiting();
                        self.scenario_ctx.wait_opecode_running = false;
                    }
                    "ShowAd" => {
                        add_delay_event!(
                            self.event_list,
                            |slf, _, _| {
                                slf.status_screen.show_ad_page();
                            },
                            self.get_current_clock() + 30
                        );
                        self.scenario_event.release_scenario_waiting();
                        self.scenario_ctx.wait_opecode_running = false;
                    }
                    "ShowMain" => {
                        add_delay_event!(
                            self.event_list,
                            |slf, ctx, _| {
                                slf.status_screen.show_main_page(ctx);
                            },
                            self.get_current_clock() + 30
                        );
                        self.scenario_event.release_scenario_waiting();
                        self.scenario_ctx.wait_opecode_running = false;
                    }
                    "ShowAdAgency" => {
                        add_delay_event!(
                            self.event_list,
                            |slf, _, _| {
                                slf.status_screen.show_ad_agency_page();
                            },
                            self.get_current_clock() + 30
                        );
                        self.scenario_event.release_scenario_waiting();
                        self.scenario_ctx.wait_opecode_running = false;
                    }
                    "NextDay" => {
                        self.status_screen.show_main_page(ctx);
                        self.status_screen.change_suzunaan_reputation(ctx, -2.0);
                        ctx.take_save_data_mut().suzunaan_status.reputation -= 2.0;

                        add_delay_event!(
                            self.event_list,
                            |slf, ctx, t| {
                                slf.scene_transition_close_effect(ctx, t);
                            },
                            self.get_current_clock() + 80
                        );

                        add_delay_event!(
                            self.event_list,
                            |slf, ctx, _| {
                                slf.scene_transition = SceneID::Scenario;
                                slf.scene_transition_type = SceneTransition::SwapTransition;
                                ctx.go_next_day();
                            },
                            self.get_current_clock() + 120
                        );
                    }
                    "GoToEndRoll" => {
                        add_delay_event!(
                            self.event_list,
                            |slf, ctx, t| {
                                slf.scene_transition_close_effect(ctx, t);
                            },
                            self.get_current_clock() + 80
                        );

                        add_delay_event!(
                            self.event_list,
                            |slf, _, _| {
                                slf.scene_transition = SceneID::End;
                                slf.scene_transition_type = SceneTransition::SwapTransition;
                                //ctx.go_next_day();
                            },
                            self.get_current_clock() + 1
                        );
                    }
                    "ShowStatusScreen" => {
                        self.status_screen.appear();
                        self.scenario_event.release_scenario_waiting();
                        self.scenario_ctx.wait_opecode_running = false;
                    }
                    _ => (),
                }
            }
        }

        if self.scenario_ctx.schedule_redefine
            && ctx.holding_week_schedule_is_available()
            && !self.scenario_ctx.schedule_define_done
        {
            self.scenario_event.release_scenario_waiting();
            self.status_screen.update_main_page_todays_sched_text(ctx);
            self.scenario_ctx.schedule_define_done = true;
            self.scenario_ctx.wait_opecode_running = false;
        }
    }

    fn start_shop_work_schedule<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        self.scenario_ctx.builtin_command_inexec = true;
        self.status_screen.show_main_page(ctx);
        self.status_screen.change_kosuzu_hp(ctx, -20.0);
        ctx.take_save_data_mut().suzunaan_status.kosuzu_hp -= 20.0;

        add_delay_event!(
            self.event_list,
            |slf, ctx, _| {
                let reputation_diff = ctx.current_total_ad_reputation_gain();
                slf.status_screen
                    .change_suzunaan_reputation(ctx, reputation_diff as f32);
                ctx.take_save_data_mut().suzunaan_status.reputation += reputation_diff as f32;
            },
            self.get_current_clock() + 100
        );

        let money_diff = ctx.current_total_ad_agency_money_gain() - ctx.current_total_ad_cost();
        self.status_screen
            .change_main_page_money(ctx, money_diff, self.get_current_clock());
        ctx.take_save_data_mut().task_result.total_money += money_diff;

        add_delay_event!(
            self.event_list,
            |slf, ctx, _| {
                slf.scene_transition = SceneID::SuzunaShop;
                slf.scene_transition_type = SceneTransition::SwapTransition;
                ctx.take_save_data_mut().award_data.shop_work_count += 1;
            },
            self.get_current_clock() + 300
        );
        add_delay_event!(
            self.event_list,
            |slf, _, _| {
                slf.scenario_ctx.builtin_command_inexec = false;
            },
            self.get_current_clock() + 301
        );
        add_delay_event!(
            self.event_list,
            |slf, ctx, t| {
                slf.scene_transition_close_effect(ctx, t);
            },
            self.get_current_clock() + 250
        );
    }

    fn start_going_out_schedule<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        self.status_screen.show_main_page(ctx);
        let money_diff =
            ctx.current_total_ad_agency_money_gain() - ctx.current_total_ad_cost() - 400;

        if ctx.take_save_data().task_result.total_money + money_diff < 0 {
            let path = ctx
                .resource
                .get_general_scenario_path(&GeneralScenarioID::NoEnoughMoney)
                .unwrap();
            let t = self.get_current_clock();
            self.scenario_event
                .replace_scenario(ctx, &mut self.scenario_ctx, &path, t);
            return;
        }

        self.status_screen
            .change_main_page_money(ctx, money_diff, self.get_current_clock());
        ctx.take_save_data_mut().task_result.add_total_money(money_diff);

        self.scenario_ctx.builtin_command_inexec = true;
        self.status_screen.show_main_page(ctx);
        self.status_screen.change_kosuzu_hp(ctx, 40.0);
        ctx.take_save_data_mut().suzunaan_status.kosuzu_hp += 40.0;
        if ctx.take_save_data().suzunaan_status.kosuzu_hp > 100.0 {
            ctx.take_save_data_mut().suzunaan_status.kosuzu_hp = 100.0;
        }

        add_delay_event!(
            self.event_list,
            |slf, ctx, _| {
                let reputation_diff = ctx.current_total_ad_reputation_gain() as f32;
                slf.status_screen
                    .change_suzunaan_reputation(ctx, reputation_diff);
                ctx.take_save_data_mut().suzunaan_status.reputation += reputation_diff;
            },
            self.get_current_clock() + 100
        );

        add_delay_event!(
            self.event_list,
            |slf, ctx, _| {
                slf.scene_transition = SceneID::Scenario;
                slf.scene_transition_type = SceneTransition::SwapTransition;
                ctx.take_save_data_mut().award_data.going_out_count += 1;
                ctx.go_next_day();
            },
            self.get_current_clock() + 200
        );
        add_delay_event!(
            self.event_list,
            |slf, _, _| {
                slf.scenario_ctx.builtin_command_inexec = false;
            },
            self.get_current_clock() + 201
        );
        add_delay_event!(
            self.event_list,
            |slf, ctx, t| {
                slf.scene_transition_close_effect(ctx, t);
            },
            self.get_current_clock() + 150
        );
    }

    fn start_taking_rest_schedule<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        self.scenario_ctx.builtin_command_inexec = true;
        self.status_screen.show_main_page(ctx);
        self.status_screen.change_kosuzu_hp(ctx, 20.0);
        ctx.take_save_data_mut().suzunaan_status.kosuzu_hp += 20.0;
        if ctx.take_save_data().suzunaan_status.kosuzu_hp > 100.0 {
            ctx.take_save_data_mut().suzunaan_status.kosuzu_hp = 100.0;
        }

        let money_diff = ctx.current_total_ad_agency_money_gain() - ctx.current_total_ad_cost();
        self.status_screen
            .change_main_page_money(ctx, money_diff, self.get_current_clock());
        ctx.take_save_data_mut().task_result.total_money += money_diff;

        add_delay_event!(
            self.event_list,
            |slf, ctx, _| {
                let reputation_diff = ctx.current_total_ad_reputation_gain() as f32 - 2.0;
                slf.status_screen
                    .change_suzunaan_reputation(ctx, reputation_diff);
                ctx.take_save_data_mut().suzunaan_status.reputation += reputation_diff;
            },
            self.get_current_clock() + 100
        );

        add_delay_event!(
            self.event_list,
            |slf, ctx, _| {
                slf.scene_transition = SceneID::Scenario;
                slf.scene_transition_type = SceneTransition::SwapTransition;
                ctx.take_save_data_mut().award_data.taking_rest_count += 1;
                ctx.go_next_day();
            },
            self.get_current_clock() + 300
        );
        add_delay_event!(
            self.event_list,
            |slf, _, _| {
                slf.scenario_ctx.builtin_command_inexec = false;
            },
            self.get_current_clock() + 301
        );
        add_delay_event!(
            self.event_list,
            |slf, ctx, t| {
                slf.scene_transition_close_effect(ctx, t);
            },
            self.get_current_clock() + 250
        );
    }

    pub fn start_schedule<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        match ctx.take_save_data().get_todays_schedule().unwrap() {
            DayWorkType::ShopWork => {
                self.start_shop_work_schedule(ctx);
            }
            DayWorkType::TakingRest => {
                self.start_taking_rest_schedule(ctx);
            }
            DayWorkType::GoingOut(_) => {
                self.start_going_out_schedule(ctx);
            }
        }
    }
}

impl SceneManager for ScenarioScene {
    fn key_down_event<'a>(&mut self, ctx: &mut SuzuContext, vkey: tdev::VirtualKey) {
        if self.now_paused() {
            match vkey {
                tdev::VirtualKey::Action4 => {
                    let t = self.get_current_clock();
                    self.exit_pause_screen(t);
                }
                _ => (),
            }
        } else {
            self.non_paused_key_down_event(ctx, vkey);
        }
    }

    fn mouse_motion_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        _offset: numeric::Vector2f,
    ) {
        let t = self.get_current_clock();

        if self.now_paused() {
            if self.pause_screen_set.is_paused_now() {
                if self.mouse_info.is_dragging(ggez::event::MouseButton::Left) {
                    self.pause_screen_set.dragging_handler(
                        ctx,
                        ggez::event::MouseButton::Left,
                        point,
                        t,
                    );
                } else {
                    self.pause_screen_set.mouse_motion_handler(ctx, point);
                }
            }
        } else {
            self.scenario_event.mouse_motion_handler(ctx, point);
        }
    }

    fn scene_popping_return_handler<'a>(&mut self, _: &mut SuzuContext<'a>) {
        self.scene_transition = SceneID::Scenario;
        self.scene_transition_type = SceneTransition::Keep;
        self.scenario_event
            .scenario_control_mut()
            .turn_back_scenario_offset(1);
    }

    fn mouse_button_down_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: ginput::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        let t = self.get_current_clock();

        self.mouse_info.set_last_clicked(button, point, t);
        self.mouse_info.set_last_down(button, point, t);
        self.mouse_info.set_last_dragged(button, point, t);
        self.mouse_info.update_dragging(button, true);

        if self.now_paused() {
            match button {
                MouseButton::Left => {
                    let t = self.get_current_clock();
                    if self.pause_screen_set.is_paused_now() {
                        self.pause_screen_set
                            .mouse_button_down(ctx, button, point, t);
                    }
                }
                _ => (),
            }
        } else {
            self.status_screen.mouse_down_handler(ctx, point, button);
        }
    }

    fn mouse_button_up_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        self.mouse_info.update_dragging(button, false);

        if self.now_paused() {
            match button {
                MouseButton::Left => {
                    let t = self.get_current_clock();
                    self.pause_screen_click_handler(ctx, point, t);
                }
                _ => (),
            }
        } else {
            self.status_screen.click_handler(ctx, point, button);

            match button {
                MouseButton::Left => {
                    let _t = self.get_current_clock();

                    if self.scenario_event.contains_scenario_text_box(point) {
                        self.scenario_event
                            .key_down_action1(ctx, self.get_current_clock());
                    }
                }
                _ => (),
            }
        }
    }

    fn pre_process<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        let t = self.get_current_clock();

        flush_delay_event_and_redraw_check!(self, self.event_list, ctx, t, {});

        if self.now_paused() {
        } else {
            // 再描画要求はupdate_textメソッドの中で行われている
            self.scenario_event.update_text(ctx, &mut self.scenario_ctx);

            if self.scenario_event.get_status() == ScenarioEventStatus::StartSchedule
                && !self.scenario_ctx.builtin_command_inexec
            {
                self.start_schedule(ctx);
            }

            self.status_screen.update(ctx, t);

            self.schedule_check(ctx);
        }

        self.pause_screen_set.effect(ctx, t);

        if let Some(transition_effect) = self.scene_transition_effect.as_mut() {
            transition_effect.effect(ctx.context, t);
            ctx.process_utility.redraw();
        }
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        //println!("{}", perf_measure!({
        self.scenario_event.draw(ctx).unwrap();
        //self.scenario_menu.draw(ctx).unwrap();
        //self.graph_sample.draw(ctx).unwrap();
        self.status_screen.draw(ctx).unwrap();

        self.pause_screen_set.draw(ctx).unwrap();

        if let Some(transition_effect) = self.scene_transition_effect.as_mut() {
            transition_effect.draw(ctx).unwrap();
        }

        //println!("status -> {}", if self.scenario_ctx.scenario_is_finish_and_wait { "finish" } else { "not finish" });
        //}));
    }

    fn post_process<'a>(&mut self, _ctx: &mut SuzuContext<'a>) -> SceneTransition {
        self.update_current_clock();

        // SceneTransition状態になっている
        if self.scenario_event.get_status() == ScenarioEventStatus::SceneTransition {
            // 遷移先のSceneIDを取り出し、遷移先として登録する
            if let Some(scene_id) = self.scenario_event.get_scene_transition() {
                self.scene_transition = scene_id;
            }

            if let Some(scene_transition) = self.scenario_event.get_scene_transition_type() {
                self.scene_transition_type = scene_transition;
            }
        }

        self.scene_transition_type
    }

    fn transition(&self) -> SceneID {
        self.scene_transition
    }

    fn get_current_clock(&self) -> Clock {
        self.clock
    }

    fn update_current_clock(&mut self) {
        self.clock += 1;
    }
}
