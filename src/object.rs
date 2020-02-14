pub mod move_fn;
pub mod effect;
pub mod collision;
pub mod character_factory;
pub mod scenario;
pub mod simulation_ui;
pub mod task_object;
pub mod task_result_object;
pub mod map_object;

use std::rc::Rc;
use std::cmp::Ordering;

use ggez::graphics as ggraphics;

use torifune::core::Clock;
use torifune::numeric;
use torifune::graphics::object as tobj;
use torifune::graphics::object::*;
use torifune::graphics::*;
use torifune::impl_texture_object_for_wrapped;
use torifune::impl_drawable_object_for_wrapped;

use crate::core::map_parser as mp;
use crate::core::{TextureID, GameData};

pub trait Clickable : TextureObject {
    fn button_down(&mut self,
                   _ctx: &mut ggez::Context,
		   _: &GameData,
		   _: Clock,
                   _button: ggez::input::mouse::MouseButton,
                   _point: numeric::Point2f) {}
    
    fn button_up(&mut self,
                 _ctx: &mut ggez::Context,
		 _: &GameData,
		 _: Clock,
                 _button: ggez::input::mouse::MouseButton,
                 _point: numeric::Point2f) {}

    fn on_click(&mut self,
                _ctx: &mut ggez::Context,
		_: &GameData,
		_: Clock,
                _button: ggez::input::mouse::MouseButton,
                _point: numeric::Point2f) {}

    fn clickable_status(&mut self,
                _ctx: &mut ggez::Context,
                    _point: numeric::Point2f) -> ggez::input::mouse::MouseCursor {
	ggez::input::mouse::MouseCursor::Default
    }
}

pub struct BlackOutParam {
    pub black_out: Clock,
    pub black_keep: Clock,
    pub black_return: Clock,
}

impl BlackOutParam {
    pub fn new(black_out: Clock, black_keep: Clock, black_return: Clock) -> Self {
	BlackOutParam {
	    black_out: black_out,
	    black_keep: black_keep,
	    black_return: black_return,
	}
    }
}

pub struct BlackOutTexture {
    texture: EffectableWrap<MovableWrap<UniTexture>>,
}

impl BlackOutTexture {
    pub fn new(game_data: &mut GameData,
	       texture_id: TextureID,
	       pos: numeric::Point2f,
	       drawing_depth: i8,
	       now: Clock) -> Self {
	BlackOutTexture {
	    texture:
	    EffectableWrap::new(
		MovableWrap::new(
		    Box::new(
			UniTexture::new(game_data.ref_texture(texture_id),
					pos,
					numeric::Vector2f::new(1.0, 1.0),
					0.0,
					drawing_depth)), None, now),
		vec![]),
	}
    }

    pub fn run_black_out(&mut self, param: BlackOutParam, now: Clock) {
	self.texture.clear_effect();
	self.texture.add_effect(vec![
	    effect::fade_in(param.black_out, now),
	    effect::fade_out(param.black_return, now + param.black_out + param.black_keep)
	]);
    }
}

impl DrawableComponent for BlackOutTexture {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
	self.texture.draw(ctx)
    }

    #[inline(always)]
    fn hide(&mut self) {
	self.texture.hide()
    }

    #[inline(always)]
    fn appear(&mut self) {
	self.texture.appear()
    }

    #[inline(always)]
    fn is_visible(&self) -> bool {
	self.texture.is_visible()
    }

    #[inline(always)]
    fn set_drawing_depth(&mut self, depth: i8) {
	self.texture.set_drawing_depth(depth)
    }

    #[inline(always)]
    fn get_drawing_depth(&self) -> i8 {
	self.texture.get_drawing_depth()
    }
}

impl DrawableObject for BlackOutTexture {
    impl_drawable_object_for_wrapped!{texture}
}

impl TextureObject for BlackOutTexture {
    impl_texture_object_for_wrapped!{texture}
}
