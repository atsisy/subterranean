use std::rc::Rc;

use ggez::graphics as ggraphics;

use torifune::core::Clock;
use torifune::graphics::drawable::*;
use torifune::graphics::object::sub_screen::*;
use torifune::graphics::object::*;
use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;
use torifune::numeric;
use torifune::roundup2f;

use crate::core::{FontID, GensoDate, SuzuContext, SavableData, TileBatchTextureID};
use crate::object::util_object::*;
use crate::object::effect;
use crate::set_table_frame_cell_center;

use number_to_jk::number_to_jk;

pub struct DrawableTaskResult {
    result_frame: TableFrame,
    fixed_text: Vec<EffectableWrap<MovableWrap<VerticalText>>>,
    background: SimpleObject,
    canvas: SubScreen,
}

impl DrawableTaskResult {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        rect_pos: numeric::Rect,
        background: SimpleObject,
	initial_save_data: SavableData,
        date: GensoDate,
        t: Clock,
    ) -> Self {
        let task_result = &ctx.savable_data.task_result;

	let font_info_large = FontInformation::new(
            ctx.resource.get_font(FontID::JpFude1),
            numeric::Vector2f::new(45.0, 45.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );
        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::JpFude1),
            numeric::Vector2f::new(35.0, 35.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );
        let init_crop = numeric::Rect::new(0.0, 0.0, 1.0, 0.0);

	let result_frame = TableFrame::new(
	    ctx.resource,
	    numeric::Point2f::new(250.0, 80.0),
	    TileBatchTextureID::OldStyleFrame,
	    FrameData::new(vec![300.0, 250.0], vec![50.0; 3]),
	    numeric::Vector2f::new(0.5, 0.5),
	    0,
	);

	let mut fixed_text = Vec::new();
        let mut title_text = EffectableWrap::new(
            MovableWrap::new(
                Box::new(VerticalText::new(
                    format!("{}", date.to_string()),
                    numeric::Point2f::new(1100.0, 80.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    font_info_large,
                )),
                None,
                t,
            ),
            vec![effect::appear_bale_down_from_top(50, t)],
        );
        title_text.set_crop(init_crop);
	fixed_text.push(title_text);

	let mut title_desc_text = EffectableWrap::new(
            MovableWrap::new(
                Box::new(VerticalText::new(
                    "御仕事結果".to_string(),
                    numeric::Point2f::new(1040.0, 300.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    font_info_large,
                )),
                None,
                t,
            ),
            vec![effect::appear_bale_down_from_top(50, t + 50)],
        );
	title_desc_text.set_crop(init_crop);
	fixed_text.push(title_desc_text);

        let mut done_work_text = EffectableWrap::new(
            MovableWrap::new(
                Box::new(VerticalText::new(
                    "御客人数".to_string(),
                    numeric::Point2f::new(600.0, 100.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    font_info,
                )),
                None,
                t,
            ),
            vec![effect::appear_bale_down_from_top(100, t + 100)],
        );
        done_work_text.set_crop(init_crop);
	set_table_frame_cell_center!(
            ctx.context,
            result_frame,
            done_work_text,
            numeric::Vector2u::new(2, 0)
        );
	fixed_text.push(done_work_text);

	let mut done_work_num_text = EffectableWrap::new(
            MovableWrap::new(
                Box::new(VerticalText::new(
                    format!("{}人", number_to_jk(task_result.done_works as u64)),
                    numeric::Point2f::new(600.0, 100.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    font_info,
                )),
                None,
                t,
            ),
            vec![effect::appear_bale_down_from_top(100, t + 100)],
        );
        done_work_num_text.set_crop(init_crop);
	set_table_frame_cell_center!(
            ctx.context,
            result_frame,
            done_work_num_text,
            numeric::Vector2u::new(2, 1)
        );
	fixed_text.push(done_work_num_text);
	
        let mut money_desc_text = EffectableWrap::new(
            MovableWrap::new(
                Box::new(VerticalText::new(
                    format!("収入"),
                    numeric::Point2f::new(480.0, 100.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    font_info,
                )),
                None,
                t,
            ),
            vec![effect::appear_bale_down_from_top(100, t + 200)],
        );
        money_desc_text.set_crop(init_crop);
	set_table_frame_cell_center!(
            ctx.context,
            result_frame,
            money_desc_text,
            numeric::Vector2u::new(1, 0)
        );
	fixed_text.push(money_desc_text);

	let mut money_text = EffectableWrap::new(
            MovableWrap::new(
                Box::new(VerticalText::new(
                    format!("{}円", number_to_jk(task_result.total_money as u64)),
                    numeric::Point2f::new(600.0, 100.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    font_info,
                )),
                None,
                t,
            ),
            vec![effect::appear_bale_down_from_top(100, t + 100)],
        );
        money_text.set_crop(init_crop);
	set_table_frame_cell_center!(
            ctx.context,
            result_frame,
            money_text,
            numeric::Vector2u::new(1, 1)
        );
	fixed_text.push(money_text);

	let mut total_money_desc_text = EffectableWrap::new(
            MovableWrap::new(
                Box::new(VerticalText::new(
                    format!("所持金"),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    font_info,
                )),
                None,
                t,
            ),
            vec![effect::appear_bale_down_from_top(100, t + 200)],
        );
        total_money_desc_text.set_crop(init_crop);
	set_table_frame_cell_center!(
            ctx.context,
            result_frame,
            total_money_desc_text,
            numeric::Vector2u::new(0, 0)
        );
	fixed_text.push(total_money_desc_text);
	
	let mut total_money_text = EffectableWrap::new(
            MovableWrap::new(
                Box::new(VerticalText::new(
                    format!("{}円", number_to_jk(ctx.savable_data.task_result.total_money as u64)),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    font_info,
                )),
                None,
                t,
            ),
            vec![effect::appear_bale_down_from_top(100, t + 100)],
        );
        total_money_text.set_crop(init_crop);
	set_table_frame_cell_center!(
            ctx.context,
            result_frame,
            total_money_text,
            numeric::Vector2u::new(0, 1)
        );
	fixed_text.push(total_money_text);
	
        DrawableTaskResult {
	    result_frame: result_frame,
	    fixed_text: fixed_text,
            background: background,
            canvas: SubScreen::new(ctx.context, rect_pos, 0, ggraphics::Color::from_rgba_u32(0)),
        }
    }
}

impl DrawableComponent for DrawableTaskResult {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.background.draw(ctx)?;

	    self.result_frame.draw(ctx)?;

	    for vtext in self.fixed_text.iter_mut() {
		vtext.draw(ctx)?;
	    }

            sub_screen::pop_screen(ctx);
            self.canvas.draw(ctx).unwrap();
        }
        Ok(())
    }

    fn hide(&mut self) {
        self.canvas.hide()
    }

    fn appear(&mut self) {
        self.canvas.appear()
    }

    fn is_visible(&self) -> bool {
        self.canvas.is_visible()
    }

    fn set_drawing_depth(&mut self, depth: i8) {
        self.canvas.set_drawing_depth(depth)
    }

    fn get_drawing_depth(&self) -> i8 {
        self.canvas.get_drawing_depth()
    }
}

impl DrawableObject for DrawableTaskResult {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for DrawableTaskResult {
    impl_texture_object_for_wrapped! {canvas}
}

impl Effectable for DrawableTaskResult {
    fn effect(&mut self, ctx: &mut ggez::Context, t: Clock) {
	for vtext in self.fixed_text.iter_mut() {
	    vtext.effect(ctx, t);
	}
    }
}