pub mod move_fn;
pub mod collision;

use torifune::core::Clock;
use torifune::core::Updatable;
use torifune::numeric;
use torifune::graphics::object as tobj;
use torifune::graphics::object::TextureObject;
use torifune::graphics::DrawableObject;

use crate::object::collision::*;

pub struct SpeedBorder {
    pub positive_x: f32,
    pub negative_x: f32,
    pub positive_y: f32,
    pub negative_y: f32,
}

impl SpeedBorder {
    pub fn round_speed_x(&self, speed: f32) -> f32 {
        if speed > self.positive_x {
            self.positive_x
        } else if speed < self.negative_x {
            self.negative_x
        } else {
            speed
        }
    }

    pub fn round_speed_y(&self, speed: f32) -> f32 {
        if speed > self.positive_y {
            self.positive_y
        } else if speed < self.negative_y {
            self.negative_y
        } else {
            speed
        }
    }
}

pub struct TextureSpeedInfo {
    fall_begin: Clock,
    gravity_acc: f32,
    horizon_resistance: f32,
    speed: numeric::Vector2f,
    speed_border: SpeedBorder,
}

impl TextureSpeedInfo {
    pub fn new(gravity_acc: f32, horizon_res: f32, speed: numeric::Vector2f, border: SpeedBorder)
               -> TextureSpeedInfo {
        TextureSpeedInfo {
            fall_begin: 0,
            gravity_acc: gravity_acc,
            horizon_resistance: horizon_res,
            speed: speed,
            speed_border: border,
        }
    }

    pub fn fall_start(&mut self, t: Clock) {
        self.fall_begin = t;
    }

    pub fn apply_resistance(&mut self, t: Clock) {
        self.set_speed_y(self.speed.y + (self.gravity_acc * (t - self.fall_begin) as f32));
        
        if self.speed.x.abs() <= self.horizon_resistance {
            self.set_speed_x(0.0);
        }else {
            self.set_speed_x(self.speed.x + if self.speed.x > 0.0 { -self.horizon_resistance } else { self.horizon_resistance });
        }
    }

    pub fn add_speed(&mut self, speed: numeric::Vector2f) {
        self.speed += speed;
    }

    pub fn set_speed(&mut self, speed: numeric::Vector2f) {
        self.speed = speed;
    }

    pub fn set_speed_x(&mut self, speed: f32) {
        self.speed.x = self.speed_border.round_speed_x(speed);
    }

    pub fn set_speed_y(&mut self, speed: f32) {
        self.speed.y = self.speed_border.round_speed_y(speed);
    }

    pub fn get_speed(&self) -> numeric::Vector2f {
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
    map_position: numeric::Point2f,
    last_map_position: numeric::Point2f,
}

impl<'a> Character<'a> {
    pub fn new(ctx: &mut ggez::Context, obj: tobj::SimpleObject<'a>, speed_info: TextureSpeedInfo) -> Character<'a> {
        Character {
            last_position: obj.get_position(),
            map_position: obj.get_position(),
            last_map_position: obj.get_position(),
            speed_info: speed_info,
            object: obj,
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

    fn get_last_move_distance(&self) -> numeric::Vector2f {
        let current = self.object.get_position();
        numeric::Vector2f::new(
            current.x - self.last_position.x,
            current.y - self.last_position.y,
        )
    }

    pub fn get_last_map_move_distance(&self) -> numeric::Vector2f {
        numeric::Vector2f::new(
            self.map_position.x - self.last_map_position.x,
            self.map_position.y - self.last_map_position.y,
        )
    }

    pub fn get_map_position(&self) -> numeric::Point2f {
        self.map_position
    }

    fn fix_collision_above(&mut self,
                            ctx: &mut ggez::Context,
                            info: &CollisionInformation,
                           t: Clock) -> f32 {
        self.speed_info.fall_start(t);
        self.speed_info.set_speed_y(1.0);
        (info.tile_position.unwrap().y + 17.0) - info.player_position.unwrap().y
    }

    fn fix_collision_bottom(&mut self,
                            ctx: &mut ggez::Context,
                            info: &CollisionInformation,
                            t: Clock) -> f32 {
        self.speed_info.fall_start(t);
        self.speed_info.set_speed_x(0.0);

        let area = self.object.get_drawing_size(ctx);
        info.tile_position.unwrap().y - (info.player_position.unwrap().y + area.y)
    }

    fn fix_collision_right(&mut self,
                            ctx: &mut ggez::Context,
                            info: &CollisionInformation,
                           t: Clock) -> f32 {
        let area = self.object.get_drawing_size(ctx);
        info.tile_position.unwrap().x - (info.player_position.unwrap().x + area.x)
    }

    fn fix_collision_left(&mut self,
                           ctx: &mut ggez::Context,
                           info: &CollisionInformation,
                          t: Clock) -> f32 {
        self.speed_info.set_speed_x(0.0);
        let area = self.object.get_drawing_size(ctx);
        info.tile_position.unwrap().x + 17.0 - info.player_position.unwrap().x
        
    }
    
    pub fn fix_collision_vertical(&mut self, ctx: &mut ggez::Context,
                         info: &CollisionInformation,
                                  t: Clock) -> f32 {
        self.speed_info.set_speed_x(0.0);
        if info.center_diff.unwrap().y < 0.0 {
            return self.fix_collision_bottom(ctx, &info, t);
        } else if info.center_diff.unwrap().y > 0.0 {
            return self.fix_collision_above(ctx, &info, t);
        }

        0.0
    }

    pub fn fix_collision_horizon(&mut self, ctx: &mut ggez::Context,
                                 info: &CollisionInformation,
                                 t: Clock)  -> f32 {
        let right = info.player_position.unwrap().x + self.object.get_drawing_area(ctx).w;
        if right > info.tile_position.unwrap().x && right < info.tile_position.unwrap().x + info.tile_position.unwrap().w {
            return self.fix_collision_right(ctx, &info, t);
        } else {
            return self.fix_collision_left(ctx, &info, t);
        }
    }
    
    pub fn apply_resistance(&mut self, t: Clock) {
        self.speed_info.apply_resistance(t);
    }

    pub fn move_right(&mut self) {
        self.speed_info.set_speed_x(6.0);
    }

    pub fn move_left(&mut self) {
        self.speed_info.set_speed_x(-6.0);
    }

    pub fn move_y(&mut self) {

        let current_y = self.object.get_position().y;
        let mut next = current_y + self.speed_info.get_speed().y;
        
        if next > 600.0 {
            next = 600.0;
        }

        let diff = next - current_y;

        self.last_position.y = current_y;
        self.object.move_diff(numeric::Vector2f::new(0.0, diff));

        self.last_map_position = self.map_position;
        self.map_position.y += diff;
    }

    pub fn move_x(&mut self) {

        let current_x = self.object.get_position().x;
        let mut next = current_x + self.speed_info.get_speed().x;

        let diff = next - current_x;

        self.last_position.x = current_x;
        self.object.move_diff(numeric::Vector2f::new(diff, 0.0));

        self.last_map_position = self.map_position;
        self.map_position.x += diff;
    }
}
