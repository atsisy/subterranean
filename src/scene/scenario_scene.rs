use ggez::graphics as ggraphics;
use ggez::input::mouse::MouseButton;

use torifune::core::Clock;
use torifune::device as tdev;
use torifune::numeric;
use torifune::graphics::object::Effectable;

use crate::core::{SuzuContext, TileBatchTextureID};

use crate::object::scenario::*;
use crate::object::simulation_ui::*;
use torifune::graphics::drawable::*;
use crate::object::util_object::*;
use crate::object::scenario_object::*;
use crate::object::effect_object;
use crate::flush_delay_event;
use crate::flush_delay_event_and_redraw_check;
use effect_object::{SceneTransitionEffectType, TilingEffectType};
use crate::object::DarkEffectPanel;

use super::*;

#[derive(Clone)]
pub enum ScenarioSelect {
    DayBegin = 0,
    DayEnd,
    OpeningEpisode,
}

pub struct ScenarioScene {
    scenario_event: ScenarioEvent,
    dark_effect_panel: DarkEffectPanel,
    pause_screen_set: Option<PauseScreenSet>,
    graph_sample: GraphDrawer,
    event_list: DelayEventList<Self>,
    status_screen: SuzunaStatusScreen,
    scene_transition_type: SceneTransition,
    scene_transition_effect: Option<effect_object::ScreenTileEffect>,
    scene_transition: SceneID,
    clock: Clock,
}

impl ScenarioScene {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, scenario_select: ScenarioSelect) -> Self {
        let file_path = match scenario_select {
            ScenarioSelect::DayBegin => ctx
                .resource
                .get_day_scenario_path(&ctx.savable_data.date)
                .expect("BUG"),
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
	    vec![numeric::Vector2f::new(0.0, 0.0), numeric::Vector2f::new(10.0, 10.0), numeric::Vector2f::new(20.0, 20.0),
		 numeric::Vector2f::new(50.0, 50.0)],
	    6.0,
	    ggraphics::Color::from_rgba_u32(0x00ff00ff),
	    2.0,
	    ggraphics::Color::from_rgba_u32(0xff),
	    0
	);

        ScenarioScene {
            scenario_event: scenario,
	    pause_screen_set: None,
	    dark_effect_panel: DarkEffectPanel::new(
                ctx.context,
                numeric::Rect::new(0.0, 0.0, 1366.0, 768.0),
                0,
            ),
	    scene_transition_effect: None,
	    event_list: DelayEventList::new(),
	    graph_sample: graph_drawer,
            scene_transition: SceneID::Scenario,
	    status_screen: SuzunaStatusScreen::new(ctx, numeric::Rect::new(616.0, 25.0, 700.0, 400.0), 0),
            scene_transition_type: SceneTransition::Keep,
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
	    t + 60
	);
	self.scene_transition_close_effect(ctx, t);
    }
    
    fn exit_pause_screen(&mut self, t: Clock) {
	self.dark_effect_panel
            .new_effect(8, t, 220, 0);
	self.pause_screen_set = None;
    }
    
    fn enter_pause_screen<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
	self.dark_effect_panel
            .new_effect(8, t, 0, 220);
	self.pause_screen_set = Some(PauseScreenSet::new(ctx, 0));
    }

    fn now_paused(&self) -> bool {
	self.pause_screen_set.is_some()
    }

    fn pause_screen_click_handler<'a>(&mut self, ctx: &mut SuzuContext<'a>, point: numeric::Point2f, t: Clock) {
	let pause_screen_set = match self.pause_screen_set.as_ref() {
	    Some(it) => it,
	    _ => return,
	};
	
	if let Some(pause_result) = pause_screen_set.mouse_click_handler(ctx, point) {
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
                self.scenario_event.key_down_action1(
                    ctx,
                    self.get_current_clock(),
                );
            }
	    tdev::VirtualKey::Action4 => {
		let t = self.get_current_clock();
		self.enter_pause_screen(ctx, t);
	    }
            tdev::VirtualKey::Right => {
                self.scenario_event.key_down_right(ctx);
            }
            tdev::VirtualKey::Left => {
                self.scenario_event.key_down_left(ctx);
            }
            _ => (),
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
		},
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
	if self.now_paused() {
	    if let Some(pause_screen_set) = self.pause_screen_set.as_mut() {
		pause_screen_set.mouse_motion_handler(ctx, point);
	    }
	}
    }
    
    fn scene_popping_return_handler<'a>(&mut self, _: &mut SuzuContext<'a>) {
        println!("recover!!!!");
        self.scene_transition = SceneID::Scenario;
        self.scene_transition_type = SceneTransition::Keep;
        self.scenario_event
            .scenario_control_mut()
            .turn_back_scenario_offset(1);
    }

    fn mouse_button_up_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
	if self.now_paused() {
	    match button {
		MouseButton::Left => {
		    let t = self.get_current_clock();
		    self.pause_screen_click_handler(ctx, point, t);
		}
		_ => (),
            }
	    
	    return;
	}
    }
    
    fn pre_process<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
	let t = self.get_current_clock();

	flush_delay_event_and_redraw_check!(self, self.event_list, ctx, t);

	if self.now_paused() {   
	} else {
	    // 再描画要求はupdate_textメソッドの中で行われている
            self.scenario_event.update_text(ctx);
	}
	
	self.dark_effect_panel.run_effect(ctx, t);

	if let Some(transition_effect) = self.scene_transition_effect.as_mut() {
            transition_effect.effect(ctx.context, t);
	    ctx.process_utility.redraw();
        }
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.scenario_event.draw(ctx).unwrap();
        //self.scenario_menu.draw(ctx).unwrap();
	//self.graph_sample.draw(ctx).unwrap();
	self.status_screen.draw(ctx).unwrap();
	self.dark_effect_panel.draw(ctx).unwrap();

	if let Some(pause_screen_set) = self.pause_screen_set.as_mut() {
	    pause_screen_set.draw(ctx).unwrap();
	}

	if let Some(transition_effect) = self.scene_transition_effect.as_mut() {
            transition_effect.draw(ctx).unwrap();
        }
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
