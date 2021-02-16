use torifune::core::*;
use torifune::graphics::drawable::*;
use torifune::graphics::object::*;
use torifune::numeric;

use crate::core::*;

use super::move_fn;

pub struct EndSceneFlow {
    main_result_vtext: MovableWrap<VerticalText>,
    drwob_essential: DrawableObjectEssential,
}

impl EndSceneFlow {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, t: Clock) -> Self {
        let mut main_result_vtext = Box::new(VerticalText::new(
	    format!("評判　　　{}\n\n総収入　　{}円\n\n接客回数　{}回\n\n貸出回数　{}回\n\n返却回数　{}回\n\n配架冊数　{}冊\n\n誤評価数　{}回",
		    ctx.take_save_data().suzunaan_status.reputation as u32,
		    number_to_jk::number_to_jk(ctx.take_save_data().task_result.total_money as u64),
		    number_to_jk::number_to_jk(ctx.take_save_data().award_data.customer_count as u64),
		    number_to_jk::number_to_jk(ctx.take_save_data().award_data.borrowing_count as u64),
		    number_to_jk::number_to_jk(ctx.take_save_data().award_data.returning_count as u64),
		    number_to_jk::number_to_jk(ctx.take_save_data().award_data.shelving_count as u64),
		    number_to_jk::number_to_jk(ctx.take_save_data().award_data.returning_check_mistake_count as u64),
	    ),
	    numeric::Point2f::new(0.0, 0.0),
	    numeric::Vector2f::new(1.0, 1.0),
	    0.0,
	    0,
	    FontInformation::new(
		ctx.resource.get_font(crate::core::FontID::Cinema),
		numeric::Vector2f::new(28.0, 28.0),
		ggez::graphics::Color::from_rgba_u32(0xff),
	    )));
        let vtext_size = main_result_vtext.get_drawing_size(ctx.context);
        main_result_vtext.set_position(numeric::Point2f::new(-vtext_size.x, 90.0));

        EndSceneFlow {
            main_result_vtext: MovableWrap::new(
                main_result_vtext,
                move_fn::move_constant(numeric::Vector2f::new(0.9, 0.0)),
                t,
            ),
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }

    pub fn update<'a>(&mut self, _ctx: &mut SuzuContext<'a>, t: Clock) {
        self.main_result_vtext.move_with_func(t);
    }
}

impl DrawableComponent for EndSceneFlow {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            if self
                .main_result_vtext
                .get_drawing_area(ctx)
                .overlaps(&numeric::Rect::new(
                    0.0,
                    0.0,
                    WINDOW_SIZE_X as f32,
                    WINDOW_SIZE_Y as f32,
                ))
            {
                self.main_result_vtext.draw(ctx)?;
            }
        }

        Ok(())
    }

    fn hide(&mut self) {
        self.drwob_essential.visible = false;
    }

    fn appear(&mut self) {
        self.drwob_essential.visible = true;
    }

    fn is_visible(&self) -> bool {
        self.drwob_essential.visible
    }

    fn set_drawing_depth(&mut self, depth: i8) {
        self.drwob_essential.drawing_depth = depth;
    }

    fn get_drawing_depth(&self) -> i8 {
        self.drwob_essential.drawing_depth
    }
}
