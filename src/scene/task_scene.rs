use torifune::device as tdev;
use torifune::core::Clock;
use torifune::graphics as tgraphics;
use torifune::graphics::object::*;
use tgraphics::object as tobj;
use ggez::input as ginput;
use ggez::graphics as ggraphics;
use torifune::numeric;
use crate::core::GameData;
use super::*;

pub struct TaskScene<'a> {
    desk_objects: Vec<tobj::SimpleObject<'a>>,
    clock: Clock,
}

impl<'a> TaskScene<'a> {
    pub fn new(_ctx: &mut ggez::Context, game_data: &'a GameData) -> TaskScene<'a>  {
        
        let obj = tobj::SimpleObject::new(
            tobj::MovableUniTexture::new(
                game_data.ref_texture(0),
                numeric::Point2f { x: 0.0, y: 0.0 },
                numeric::Vector2f { x: 1.0, y: 1.0 },
                0.0, 0,  Box::new(move |p: & dyn tobj::MovableObject, t: Clock| {
                    torifune::numeric::Point2f{x: p.get_position().x + 1.0, y: p.get_position().y}
                }),
                0), vec![]);
        
        TaskScene {
            desk_objects: vec![obj],
            clock: 0
        }
    }
}

impl<'a> SceneManager for TaskScene<'a> {
    
    fn key_down_event(&mut self, ctx: &mut ggez::Context, vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => println!("Action1 down!"),
            _ => (),
        }
    }
    
    fn key_up_event(&mut self,
                    _ctx: &mut ggez::Context,
                    vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => println!("Action1 up!"),
            _ => (),
        }
    }

    fn mouse_motion_event(&mut self,
                          _ctx: &mut ggez::Context,
                          point: numeric::Point2f,
                          _offset: numeric::Vector2f) {
        println!("x: {}, y: {}", point.x, point.y);
    }

    fn mouse_button_down_event(&mut self,
                               ctx: &mut ggez::Context,
                               button: ginput::mouse::MouseButton,
                               point: numeric::Point2f) {
        
    }
    
    fn mouse_button_up_event(&mut self,
                             ctx: &mut ggez::Context,
                             button: ginput::mouse::MouseButton,
                             point: numeric::Point2f) {
        
    }

    fn pre_process(&mut self, ctx: &mut ggez::Context) {
        for p in &mut self.desk_objects {
            p.move_with_func(self.clock);
        }
    }
    
    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        for p in &self.desk_objects {
            p.draw(ctx).unwrap();
        }
    }
    
    fn post_process(&mut self, ctx: &mut ggez::Context) -> SceneTransition {
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
