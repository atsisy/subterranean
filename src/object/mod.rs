pub mod move_fn;
use torifune::core::Clock;
use torifune::core::Updatable;
use torifune::numeric;
use torifune::graphics::object as tobj;
use torifune::graphics::DrawableObject;

pub struct TextureSpeedInfo {
    fall_begin: Clock,
    gravity_acc: f32,
    speed: numeric::Vector2f,
}

impl TextureSpeedInfo {
    pub fn new(gravity_acc: f32, speed: numeric::Vector2f) -> TextureSpeedInfo {
        TextureSpeedInfo {
            fall_begin: 0,
            gravity_acc: gravity_acc,
            speed: speed
        }
    }

    pub fn fall_start(&mut self, t: Clock) {
        self.fall_begin = t;
    }

    fn apply_gravity(&mut self, t: Clock) {
        self.speed.y += (self.gravity_acc * (t - self.fall_begin) as f32);
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
    speed_info: TextureSpeedInfo,
    object: tobj::SimpleObject<'a>,
}

impl<'a> Character<'a> {
    pub fn new(obj: tobj::SimpleObject<'a>, speed_info: TextureSpeedInfo) -> Character<'a> {
        Character {
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
}

impl<'a> Updatable for Character<'a> {
    fn update(&mut self, _ctx: &ggez::Context, t: Clock) -> Result<(), &'static str> {
        self.speed_info.apply_gravity(t);

        let p = self.object.get_position();
        let mut next = p + self.speed_info.get_speed();
        if next.y > 600.0 {
            next.y = 600.0;
        }
        
        self.object.set_position(next);
        Ok(())
    }
}
