pub mod move_fn;
use torifune::core::Clock;
use torifune::core::Updatable;
use torifune::numeric;
use torifune::graphics::object as tobj;
use torifune::graphics::object::TextureObject;
use torifune::graphics::DrawableObject;

use crate::core::map_parser::CollisionInformation;

pub struct TextureSpeedInfo {
    fall_begin: Clock,
    border: numeric::Rect,
    gravity_acc: f32,
    speed: numeric::Vector2f,
}

impl TextureSpeedInfo {
    pub fn new(gravity_acc: f32, speed: numeric::Vector2f, border: numeric::Rect)
               -> TextureSpeedInfo {
        TextureSpeedInfo {
            fall_begin: 0,
            border: border,
            gravity_acc: gravity_acc,
            speed: speed
        }
    }

    pub fn fall_start(&mut self, t: Clock) {
        self.fall_begin = t;
    }

    fn apply_gravity(&mut self, t: Clock) {
        self.speed.y += self.gravity_acc * (t - self.fall_begin) as f32;
    }

    pub fn add_speed(&mut self, speed: numeric::Vector2f) {
        self.speed += speed;
    }

    pub fn set_speed(&mut self, speed: numeric::Vector2f) {
        self.speed = speed;
    }

    fn get_speed(&self) -> numeric::Vector2f {
        self.speed
    }

    fn set_gravity(&mut self, g: f32) {
        self.gravity_acc = g;
    }
}

pub struct Character<'a> {
    last_position: numeric::Point2f,
    speed_info: TextureSpeedInfo,
    object: tobj::SimpleObject<'a>,
}

impl<'a> Character<'a> {
    pub fn new(obj: tobj::SimpleObject<'a>, speed_info: TextureSpeedInfo) -> Character<'a> {
        Character {
            last_position: obj.get_position(),
            speed_info: speed_info,
            object: obj
        }
    }

    pub fn speed_info(&self) -> &TextureSpeedInfo {
        &self.speed_info
    }

    pub fn speed_info_mut(&mut self) -> &mut TextureSpeedInfo {
        &mut self.speed_info
    }

    pub fn obj(&self) -> &tobj::SimpleObject<'a> {
        &self.object
    }
    
    pub fn obj_mut(&mut self) -> &mut tobj::SimpleObject<'a> {
        &mut self.object
    }

    pub fn get_last_position(&self) -> numeric::Point2f {
        self.last_position
    }

    pub fn undo_move(&mut self) {
        self.object.set_position(self.get_last_position());
    }

    pub fn fix_collision(&mut self, ctx: &mut ggez::Context, info: &CollisionInformation, t: Clock) {
        self.speed_info.fall_start(t);
        
        let p = self.object.get_position();
        let v = info.boundly.unwrap();

        let area = self.object.get_drawing_size(ctx);

        let next = numeric::Point2f::new(p.x, v.y - area.y);
        
        self.object.set_position(next);
    }
}

impl<'a> Updatable for Character<'a> {
    fn update(&mut self, _ctx: &ggez::Context, t: Clock) {
        self.speed_info.apply_gravity(t);

        let p = self.object.get_position();
        
        let mut next = p + self.speed_info.get_speed();
        if next.y > self.speed_info().border.y + self.speed_info().border.h {
            next.y = 600.0;
        }

        self.last_position = p;
        self.object.set_position(next);
    }
}
