use ggez::graphics as ggraphics;

use torifune::core::Clock;
use torifune::graphics::drawable::*;
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::graphics::object::tile_batch::*;
use torifune::graphics::object::*;
use torifune::numeric;

use crate::core::*;

pub enum SceneTransitionEffectType {
    Open,
    Close,
}

pub enum TilingEffectType {
    OneTile,
    WholeTile,
}

pub struct ScreenTileEffect {
    tile_batch: TileBatch,
    effect_start: Clock,
    animation_rate: f32,
    st_effect_type: SceneTransitionEffectType,
    canvas: SubScreen,
    tiling_effect_type: TilingEffectType,
}

impl ScreenTileEffect {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        tile_batch_texture_id: TileBatchTextureID,
        rect: numeric::Rect,
        animation_time: Clock,
        st_effect_type: SceneTransitionEffectType,
	tiling_effect_type: TilingEffectType,
        depth: i8,
        t: Clock,
    ) -> Self {
        let tile_batch = game_data.ref_tile_batch(tile_batch_texture_id);
        let size = tile_batch.get_tile_size();

        ScreenTileEffect {
            tile_batch: tile_batch,
            animation_rate: animation_time as f32 / (rect.w + rect.h + size.x as f32),
            canvas: SubScreen::new(ctx, rect, depth, ggraphics::Color::from_rgba_u32(0)),
            st_effect_type: st_effect_type,
	    tiling_effect_type: tiling_effect_type,
            effect_start: t,
        }
    }

    pub fn update_batch(&mut self, t: Clock) {
        self.tile_batch.clear_batch();

        let elapsed = (t - self.effect_start) as f32;
        let size = self.tile_batch.get_tile_size();

        for x in (0..crate::core::WINDOW_SIZE_X).step_by(size.x as usize) {
            for y in (0..crate::core::WINDOW_SIZE_Y).step_by(size.y as usize) {
                let alpha = match self.st_effect_type {
                    SceneTransitionEffectType::Close => {
                        elapsed / ((size.x as i16 + x + y) as f32 * self.animation_rate)
                    }
                    SceneTransitionEffectType::Open => {
                        1.0 - (elapsed / ((size.x as i16 + x + y) as f32 * self.animation_rate))
                    }
                };

		let tile_pos = match self.tiling_effect_type {
		    TilingEffectType::OneTile => numeric::Vector2u::new(0, 0),
		    TilingEffectType::WholeTile => numeric::Vector2u::new(
			(x / size.x as i16) as u32, (y / size.y as i16) as u32
		    ),
		};
		
                self.tile_batch.add_batch_tile_position(
		    tile_pos,
                    numeric::Point2f::new(x as f32, y as f32),
                    numeric::Vector2f::new(1.0, 1.0),
                    ggraphics::Color::new(1.0, 1.0, 1.0, alpha),
                );
            }
        }
    }
}

impl DrawableComponent for ScreenTileEffect {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.tile_batch.draw(ctx)?;

            sub_screen::pop_screen(ctx);
            self.canvas.draw(ctx).unwrap();
        }

        Ok(())
    }

    fn hide(&mut self) {
        self.canvas.hide();
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

impl Effectable for ScreenTileEffect {
    fn effect(&mut self, _: &mut ggez::Context, t: Clock) {
        self.update_batch(t);
    }
}
