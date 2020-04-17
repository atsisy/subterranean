use torifune::core::Clock;
use torifune::device as tdev;
use torifune::numeric;

use crate::core::SuzuContext;

use crate::object::scenario::*;
use crate::object::simulation_ui as sui;
use torifune::graphics::drawable::*;

use super::*;

pub struct ScenarioScene {
    scenario_event: ScenarioEvent,
    simulation_status: sui::SimulationStatus,
    scene_transition: SceneID,
    clock: Clock,
}

impl ScenarioScene {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>) -> Self {
        let file_path = ctx
            .resource
            .get_day_scenario_path(&ctx.savable_data.date)
            .expect("BUG");

        let scenario = ScenarioEvent::new(
            ctx,
            numeric::Rect::new(0.0, 180.0, 1366.0, 600.0),
            &file_path,
            0,
        );

        ScenarioScene {
            simulation_status: sui::SimulationStatus::new(
                ctx,
                numeric::Rect::new(0.0, 0.0, 1366.0, 180.0),
            ),
            scenario_event: scenario,
            scene_transition: SceneID::Scenario,
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
                    ctx.context,
                    ctx.resource,
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

    fn pre_process<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        self.scenario_event.update_text(ctx);
        self.simulation_status.update();
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.scenario_event.draw(ctx).unwrap();

        self.simulation_status.draw(ctx).unwrap();
    }

    fn post_process<'a>(&mut self, _ctx: &mut SuzuContext<'a>) -> SceneTransition {
        self.update_current_clock();

        // SceneTransition状態になっている
        if self.scenario_event.get_status() == ScenarioEventStatus::SceneTransition {
            // 遷移先のSceneIDを取り出し、遷移先として登録する
            if let Some(scene_id) = self.scenario_event.get_scene_transition() {
                self.scene_transition = scene_id;
                return SceneTransition::SwapTransition;
            }
        }

        SceneTransition::Keep
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
