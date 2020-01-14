use torifune::device as tdev;
use torifune::core::Clock;
use ggez::graphics as ggraphics;
use ginput::mouse::MouseButton;
use torifune::numeric;

use torifune::graphics::*;

use super::*;

use crate::object::task_object;

use crate::core::GameData;
use crate::object::task_object::*;
use crate::object::simulation_ui as sui;

pub struct TaskScene {
    task_table: TaskTable,
    simulation_status: sui::SimulationStatus,
    clock: Clock,
    mouse_info: MouseInformation,
}

impl TaskScene {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData) -> TaskScene  {

        TaskScene {
            task_table: TaskTable::new(ctx, game_data,
                                       ggraphics::Rect::new(10.0, 230.0, 1300.0, 500.0),
                                       ggraphics::Rect::new(10.0, 0.0, 520.0, 500.0),
                                       ggraphics::Rect::new(540.0, 0.0, 900.0, 500.0)),
	    simulation_status: sui::SimulationStatus::new(ctx, numeric::Rect::new(0.0, 0.0, 1366.0, 180.0), game_data),
            clock: 0,
            mouse_info: MouseInformation::new(),
        }
    }

    fn dragging_handler(&mut self,
                        ctx: &mut ggez::Context,
                        point: numeric::Point2f,
                        _offset: numeric::Vector2f,
                        _game_data: &GameData) {
        let last = self.mouse_info.get_last_dragged(MouseButton::Left);
        self.task_table.dragging_handler(point, last);
        self.task_table.hand_over_check(ctx, point);
    }

    fn select_dragging_object(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        self.task_table.select_dragging_object(ctx, point);
    }

    fn unselect_dragging_object(&mut self, ctx: &mut ggez::Context, t: Clock) {
        self.task_table.unselect_dragging_object(ctx, t);
    }
}

impl SceneManager for TaskScene {

    fn key_down_event(&mut self,
                      _ctx: &mut ggez::Context,
                      _game_data: &GameData,
                      vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => {
                println!("Action1 down!");
            },
            _ => (),
        }
    }

    fn key_up_event(&mut self,
                    ctx: &mut ggez::Context,
                    game_data: &GameData,
                    vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => println!("Action1 up!"),
            tdev::VirtualKey::Action2 => {
                self.task_table.start_customer_event(
                    ctx, game_data,
                    task_object::BorrowingInformation::new_random(
                        game_data,
                        task_object::GensoDate::new(128, 12, 20),
                        task_object::GensoDate::new(128, 12, 20)
                    ), self.get_current_clock());
            },
            _ => (),
        }
    }

    fn mouse_motion_event(&mut self,
                          ctx: &mut ggez::Context,
                          game_data: &GameData,
                          point: numeric::Point2f,
                          offset: numeric::Vector2f) {
        if self.mouse_info.is_dragging(MouseButton::Left) {
            let d = numeric::Vector2f::new(offset.x / 2.0, offset.y / 2.0);
            self.dragging_handler(ctx, point, d, game_data);
            self.mouse_info.set_last_dragged(MouseButton::Left, point, self.get_current_clock());
        }
    }

    fn mouse_button_down_event(&mut self,
                               ctx: &mut ggez::Context,
                               game_data: &GameData,
                               button: MouseButton,
                               point: numeric::Point2f) {
        let info: &MouseActionRecord = &self.mouse_info.last_clicked.get(&button).unwrap();
        if info.point == point && (self.get_current_clock() - info.t) < 20 {
            self.task_table.double_click_handler(ctx, point, game_data);
        }

        self.mouse_info.set_last_clicked(button, point, self.get_current_clock());
        self.mouse_info.set_last_dragged(button, point, self.get_current_clock());
        self.mouse_info.update_dragging(button, true);

        self.select_dragging_object(ctx, point);
    }

    fn mouse_button_up_event(&mut self,
                             ctx: &mut ggez::Context,
                             _game_data: &GameData,
                             button: MouseButton,
                             _point: numeric::Point2f) {
        self.mouse_info.update_dragging(button, false);
        //self.paper.button_up(ctx, button, point);
        self.unselect_dragging_object(ctx, self.get_current_clock());
    }

    fn pre_process(&mut self,
                   ctx: &mut ggez::Context,
                   game_data: &GameData) {
        self.task_table.update(ctx, game_data, self.get_current_clock());
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.task_table.draw(ctx).unwrap();
	self.simulation_status.draw(ctx).unwrap();
    }

    fn post_process(&mut self, _ctx: &mut ggez::Context, _: &GameData) -> SceneTransition {
        self.update_current_clock();
        SceneTransition::Keep
    }

    fn transition(&self) -> SceneID {
        SceneID::MainDesk
    }

    fn get_current_clock(&self) -> Clock {
        self.clock
    }

    fn update_current_clock(&mut self) {
        self.clock += 1;
    }

}
