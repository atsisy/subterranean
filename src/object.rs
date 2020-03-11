pub mod character_factory;
pub mod collision;
pub mod copy_scene_object;
pub mod effect;
pub mod map_object;
pub mod move_fn;
pub mod scenario;
pub mod shop_object;
pub mod simulation_ui;
pub mod task_object;
pub mod task_result_object;

use std::rc::Rc;

use ggez::graphics as ggraphics;

use torifune::core::Clock;
use torifune::graphics::object as tobj;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::graphics::object::*;
use torifune::graphics::*;
use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;
use torifune::numeric;

use crate::core::map_parser as mp;
use crate::core::{GameData, TextureID};

pub trait Clickable: TextureObject {
    fn button_down(
        &mut self,
        _ctx: &mut ggez::Context,
        _: &GameData,
        _: Clock,
        _button: ggez::input::mouse::MouseButton,
        _point: numeric::Point2f,
    ) {
    }

    fn button_up(
        &mut self,
        _ctx: &mut ggez::Context,
        _: &GameData,
        _: Clock,
        _button: ggez::input::mouse::MouseButton,
        _point: numeric::Point2f,
    ) {
    }

    fn on_click(
        &mut self,
        _ctx: &mut ggez::Context,
        _: &GameData,
        _: Clock,
        _button: ggez::input::mouse::MouseButton,
        _point: numeric::Point2f,
    ) {
    }

    fn clickable_status(
        &mut self,
        _ctx: &mut ggez::Context,
        _point: numeric::Point2f,
    ) -> ggez::input::mouse::MouseCursor {
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
    pub fn new(
        game_data: &mut GameData,
        texture_id: TextureID,
        pos: numeric::Point2f,
        drawing_depth: i8,
        now: Clock,
    ) -> Self {
        BlackOutTexture {
            texture: EffectableWrap::new(
                MovableWrap::new(
                    Box::new(UniTexture::new(
                        game_data.ref_texture(texture_id),
                        pos,
                        numeric::Vector2f::new(1.0, 1.0),
                        0.0,
                        drawing_depth,
                    )),
                    None,
                    now,
                ),
                vec![],
            ),
        }
    }

    pub fn run_black_out(&mut self, param: BlackOutParam, now: Clock) {
        self.texture.clear_effect();
        self.texture.add_effect(vec![
            effect::fade_in(param.black_out, now),
            effect::fade_out(param.black_return, now + param.black_out + param.black_keep),
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
    impl_drawable_object_for_wrapped! {texture}
}

impl TextureObject for BlackOutTexture {
    impl_texture_object_for_wrapped! {texture}
}

pub struct DarkEffectPanel {
    canvas: EffectableWrap<MovableWrap<SubScreen>>,
}

impl DarkEffectPanel {
    pub fn new(ctx: &mut ggez::Context, rect: numeric::Rect, now: Clock) -> Self {
        DarkEffectPanel {
            canvas: EffectableWrap::new(
                MovableWrap::new(
                    Box::new(SubScreen::new(
                        ctx,
                        rect,
                        0,
                        ggraphics::Color::from_rgba_u32(0),
                    )),
                    None,
                    now,
                ),
                vec![],
            ),
        }
    }

    pub fn new_effect(
        &mut self,
        required_time: Clock,
        now: Clock,
        init_dark_alpha: u8,
        fin_dark_alpha: u8,
    ) {
        self.canvas.add_effect(vec![effect::alpha_effect(
            required_time,
            now,
            init_dark_alpha,
            fin_dark_alpha,
        )]);
    }

    pub fn run_effect(&mut self, ctx: &mut ggez::Context, t: Clock) {
        self.canvas.effect(ctx, t);
    }
}

impl DrawableComponent for DarkEffectPanel {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        sub_screen::stack_screen(ctx, self.canvas.ref_wrapped_object().ref_wrapped_object());
        sub_screen::pop_screen(ctx);
        self.canvas.draw(ctx)
    }

    #[inline(always)]
    fn hide(&mut self) {
        self.canvas.hide()
    }

    #[inline(always)]
    fn appear(&mut self) {
        self.canvas.appear()
    }

    #[inline(always)]
    fn is_visible(&self) -> bool {
        self.canvas.is_visible()
    }

    #[inline(always)]
    fn set_drawing_depth(&mut self, depth: i8) {
        self.canvas.set_drawing_depth(depth)
    }

    #[inline(always)]
    fn get_drawing_depth(&self) -> i8 {
        self.canvas.get_drawing_depth()
    }
}
