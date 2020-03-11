use ggez::graphics as ggraphics;

use torifune::graphics::object::sub_screen;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::graphics::object::*;
use torifune::graphics::*;
use torifune::numeric;

use crate::core::{GameData, TextureID};

pub struct EffectableHangi {
    hangi_texture: UniTexture,
    canvas: SubScreen,
}

impl EffectableHangi {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData, rect: numeric::Rect) -> Self {
        EffectableHangi {
            hangi_texture: UniTexture::new(
                game_data.ref_texture(TextureID::Wood1),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                1,
            ),
            canvas: SubScreen::new(ctx, rect, 0, ggraphics::Color::from_rgba_u32(0xffffffff)),
        }
    }
}

impl DrawableComponent for EffectableHangi {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.hangi_texture.draw(ctx).unwrap();

            sub_screen::pop_screen(ctx);
            self.canvas.draw(ctx).unwrap();
        }
        Ok(())
    }

    fn hide(&mut self) {
        self.canvas.hide()
    }

    fn appear(&mut self) {
        self.canvas.appear();
    }

    fn is_visible(&self) -> bool {
        self.canvas.is_visible()
    }

    fn set_drawing_depth(&mut self, depth: i8) {
        self.canvas.set_drawing_depth(depth);
    }

    fn get_drawing_depth(&self) -> i8 {
        self.canvas.get_drawing_depth()
    }
}
