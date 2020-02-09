use torifune::device as tdev;
use torifune::core::Clock;
use torifune::numeric;

use crate::core::GameData;

use crate::object::scenario::*;
use crate::object::simulation_ui as sui;
use torifune::graphics::*;

use super::*;

pub struct ScenarioScene {
    scenario_event: ScenarioEvent,
    simulation_status: sui::SimulationStatus,
    clock: Clock,
}

impl ScenarioScene {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData) -> Self {
        let scenario = ScenarioEvent::new(ctx, numeric::Rect::new(100.0, 520.0, 1200.0, 280.0),
                                          "./resources/scenario_parsing_test.toml",
                                          game_data, 0);
        
        ScenarioScene {
            simulation_status: sui::SimulationStatus::new(ctx, numeric::Rect::new(0.0, 0.0, 1366.0, 180.0), game_data),
            scenario_event: scenario,
            clock: 0,
        }
    }
}

impl SceneManager for ScenarioScene {
    
    fn key_down_event(&mut self,
                      _ctx: &mut ggez::Context,
                      _game_data: &GameData,
                      vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => {
                println!("Action1 down!");
                self.scenario_event.next_page();
            },
            _ => (),
        }
    }
    
    fn key_up_event(&mut self,
                    _ctx: &mut ggez::Context,
                    _game_data: &GameData,
                    vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => println!("Action1 up!"),
            _ => (),
        }
    }

    fn pre_process(&mut self, _ctx: &mut ggez::Context, _: &GameData) {
        self.scenario_event.update_text();
        self.simulation_status.update();
    }
    
    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.scenario_event.draw(ctx).unwrap();
        self.simulation_status.draw(ctx).unwrap();
    }
    
    fn post_process(&mut self, _ctx: &mut ggez::Context, _: &GameData) -> SceneTransition {
        self.update_current_clock();
        SceneTransition::Keep
    }

    fn transition(&self) -> SceneID {
        SceneID::Scenario
    }

    fn get_current_clock(&self) -> Clock {
        self.clock
    }
    
    fn update_current_clock(&mut self) {
        self.clock += 1;
    }
    
}

