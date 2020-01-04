use torifune::device as tdev;
use torifune::core::Clock;
use torifune::graphics as tgraphics;
use tgraphics::object as tobj;
use ggez::graphics as ggraphics;
use ginput::mouse::MouseButton;
use torifune::numeric;

use torifune::graphics::*;
use torifune::graphics::object::TextureObject;
use torifune::graphics::object::MovableObject;

use super::*;

use crate::core::{TextureID, GameData};
use crate::object::task_object::*;
use crate::object::move_fn;

pub struct TaskScene {
    desk_objects: DeskObjects,
    dobj_container: ObjectContainer<Box<dyn DrawableComponent>>,
    clock: Clock,
    mouse_info: MouseInformation,
}

impl TaskScene {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData) -> TaskScene  {
        
        TaskScene {
            desk_objects: DeskObjects::new(ctx, game_data,
                                           ggraphics::Rect::new(150.0, 350.0, 1000.0, 400.0)),
            /*
            paper: BorrowingPaper::new(ctx, ggraphics::Rect::new(10.0, 10.0, 700.0, 700.0), TextureID::Paper1,
                                       &BorrowingInformation::new(vec!["テスト本1".to_string(), "テスト本2".to_string()],
                                                                  "霧雨魔里沙".to_string(), GensoDate::new(128, 12, 8),
                                                                  GensoDate::new(128, 12, 8)), game_data, 0),
            */
            clock: 0,
            dobj_container: ObjectContainer::<Box<dyn DrawableComponent>>::new(),
            mouse_info: MouseInformation::new(),
        }
    }
    
    fn dragging_handler(&mut self,
                        _ctx: &mut ggez::Context,
                        point: numeric::Point2f,
                        _offset: numeric::Vector2f) {
        let last = self.mouse_info.get_last_dragged(MouseButton::Left);
        self.desk_objects.dragging_handler(point, last);
    }

    fn select_dragging_object(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        self.desk_objects.select_dragging_object(ctx, point);
    }

    fn unselect_dragging_object(&mut self) {
        self.desk_objects.unselect_dragging_object();
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
                    _ctx: &mut ggez::Context,
                    _game_data: &GameData,
                    vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => println!("Action1 up!"),
            _ => (),
        }
    }

    fn mouse_motion_event(&mut self,
                          ctx: &mut ggez::Context,
                          _game_data: &GameData,
                          point: numeric::Point2f,
                          offset: numeric::Vector2f) {
        if self.mouse_info.is_dragging(MouseButton::Left) {
            //println!("x: {}, y: {} ::: offset_x: {}, offset_y: {}", point.x, point.y, offset.x, offset.y);
            let d = numeric::Vector2f::new(offset.x / 2.0, offset.y / 2.0);
            self.dragging_handler(ctx, point, d);
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
            self.desk_objects.double_click_handler(ctx, point, game_data);
        }
        
        self.mouse_info.set_last_clicked(button, point, self.get_current_clock());
        self.mouse_info.set_last_dragged(button, point, self.get_current_clock());
        self.mouse_info.update_dragging(button, true);

        self.select_dragging_object(ctx, point);
    }
    
    fn mouse_button_up_event(&mut self,
                             _ctx: &mut ggez::Context,
                             _game_data: &GameData,
                             button: MouseButton,
                             _point: numeric::Point2f) {
        self.mouse_info.update_dragging(button, false);
        //self.paper.button_up(ctx, button, point);
        self.unselect_dragging_object();
    }

    fn pre_process(&mut self, ctx: &mut ggez::Context) {
        self.desk_objects.update(ctx, self.get_current_clock());
    }
    
    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.desk_objects.draw(ctx).unwrap();
        for obj in self.dobj_container.iter_mut() {
            obj.draw(ctx).unwrap();
        }
    }
    
    fn post_process(&mut self, _ctx: &mut ggez::Context) -> SceneTransition {
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
