use ggez::graphics as ggraphics;

use torifune::core::Clock;
use torifune::device as tdev;
use torifune::numeric;

use crate::core::SuzuContext;

use crate::object::scenario::*;
use crate::object::simulation_ui::*;
use torifune::graphics::drawable::*;
use crate::object::util_object::*;

use super::*;

#[derive(Clone)]
pub enum ScenarioSelect {
    DayBegin = 0,
    DayEnd,
    OpeningEpisode,
}

pub struct ScenarioScene {
    scenario_event: ScenarioEvent,
    scenario_menu: ScenarioMenu,
    graph_sample: GraphDrawer,
    scene_transition_type: SceneTransition,
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
            numeric::Rect::new(300.0, 0.0, 1066.0, 768.0),
            &file_path,
            0,
        );

	let graph_drawer = GraphDrawer::new(
	    ctx,
	    numeric::Rect::new(300.0, 100.0, 700.0, 600.0),
	    numeric::Rect::new(10.0, 10.0, 680.0, 580.0),
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
            scenario_menu: ScenarioMenu::new(ctx, numeric::Vector2f::new(300.0, 768.0)),
	    graph_sample: graph_drawer,
            scene_transition: SceneID::Scenario,
            scene_transition_type: SceneTransition::Keep,
            clock: 0,
        }
    }
}

impl SceneManager for ScenarioScene {
    fn key_down_event<'a>(&mut self, ctx: &mut SuzuContext, vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => {
                println!("Action1 down!");
                self.scenario_event.key_down_action1(
                    ctx,
                    self.get_current_clock(),
                );
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

    fn scene_popping_return_handler<'a>(&mut self, _: &mut SuzuContext<'a>) {
        println!("recover!!!!");
        self.scene_transition = SceneID::Scenario;
        self.scene_transition_type = SceneTransition::Keep;
        self.scenario_event
            .scenario_control_mut()
            .turn_back_scenario_offset(1);
    }

    fn pre_process<'a>(&mut self, ctx: &mut SuzuContext<'a>) -> DrawRequest {
        self.scenario_event.update_text(ctx);

	DrawRequest::Draw
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.scenario_event.draw(ctx).unwrap();
        self.scenario_menu.draw(ctx).unwrap();
	self.graph_sample.draw(ctx).unwrap();
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
