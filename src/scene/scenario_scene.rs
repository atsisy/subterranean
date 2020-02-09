use ggez::graphics as ggraphics;

use torifune::device as tdev;
use torifune::core::Clock;
use torifune::numeric;
use torifune::graphics::object::*;

use crate::core::{GameData, FontID};

use crate::object::scenario::*;
use crate::object::simulation_ui as sui;
use crate::object::scenario::ChoiceBox;
use torifune::graphics::*;

use super::*;

pub struct ScenarioScene {
    scenario_event: ScenarioEvent,
    choice_box: Option<ChoiceBox>,
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
	    choice_box: None,
            scenario_event: scenario,
            clock: 0,
        }
    }
}

impl SceneManager for ScenarioScene {
    
    fn key_down_event(&mut self,
                      ctx: &mut ggez::Context,
                      game_data: &GameData,
                      vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => {
                println!("Action1 down!");
		if let Some(choice) = &mut self.choice_box {
		    self.scenario_event.make_scenario_event();
		}
		self.choice_box = None;
                self.scenario_event.next_page();
            },
	    tdev::VirtualKey::Right => {
		if let Some(choice) = &mut self.choice_box {
		    choice.move_right();
		    
		    self.scenario_event.set_fixed_text(choice.get_selecting_str(),
						       FontInformation::new(game_data.get_font(FontID::JpFude1),
									    numeric::Vector2f::new(32.0, 32.0),
									    ggraphics::Color::from_rgba_u32(0x000000ff)));
		}
	    },
	    tdev::VirtualKey::Left => {
		if let Some(choice) = &mut self.choice_box {
		    choice.move_left();
		    
		    self.scenario_event.set_fixed_text(choice.get_selecting_str(),
						       FontInformation::new(game_data.get_font(FontID::JpFude1),
									    numeric::Vector2f::new(32.0, 32.0),
									    ggraphics::Color::from_rgba_u32(0x000000ff)));
		}
	    },
	    tdev::VirtualKey::Action2 => {
		println!("choice box is appearing");
		self.choice_box = Some(ChoiceBox::new(
		    ctx, numeric::Rect::new(110.0, 600.0, 1200.0, 150.0),
		    game_data, vec!["選択肢1".to_string(), "選択肢2".to_string(), "選択肢3".to_string()]));
		self.scenario_event.set_fixed_text(self.choice_box.as_ref().unwrap().get_selecting_str(),
						   FontInformation::new(game_data.get_font(FontID::JpFude1),
									numeric::Vector2f::new(32.0, 32.0),
									ggraphics::Color::from_rgba_u32(0x000000ff)));
	    }
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

	if let Some(choice_box) = &mut self.choice_box {
	    choice_box.draw(ctx).unwrap();
	}
	
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

