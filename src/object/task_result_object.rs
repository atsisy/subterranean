use ggez::graphics as ggraphics;

use torifune::core::Clock;
use torifune::graphics::drawable::*;
use torifune::graphics::object::sub_screen::*;
use torifune::graphics::object::*;
use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;
use torifune::numeric;
use torifune::roundup2f;

use crate::core::{FontID, GensoDate, ResultReport, SavableData, SuzuContext, TileBatchTextureID};
use crate::object::effect;
use crate::object::util_object::*;
use crate::set_table_frame_cell_center;

use crate::object::simulation_ui::*;

use number_to_jk::number_to_jk;

struct DrawableEvaluationFlow {
    eval_frame: TableFrame,
    desc_text: Vec<VerticalText>,
    result_text: Vec<EffectableWrap<MovableWrap<VerticalText>>>,
    drwob_essential: DrawableObjectEssential,
}

impl DrawableEvaluationFlow {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        pos: numeric::Point2f,
        result_report: ResultReport,
        effect_clock_offset: Clock,
        depth: i8,
        t: Clock,
    ) -> Self {
        let eval_frame = TableFrame::new(
            ctx.resource,
            pos,
            TileBatchTextureID::OldStyleFrame,
            FrameData::new(vec![250.0, 250.0], vec![50.0; 4]),
            numeric::Vector2f::new(0.5, 0.5),
            0,
        );
        let init_crop = numeric::Rect::new(0.0, 0.0, 1.0, 0.0);

        let mut desc_text = Vec::new();
        let mut result_text = Vec::new();

        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::JpFude1),
            numeric::Vector2f::new(28.0, 28.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        for (index, s) in vec!["総合評価", "誤評価数", "配架完了", "客を待たせた時間"]
            .iter()
            .enumerate()
        {
            let mut vtext = VerticalText::new(
                s.to_string(),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                font_info.clone(),
            );

            set_table_frame_cell_center!(
                ctx.context,
                eval_frame,
                vtext,
                numeric::Vector2u::new(index as u32, 0)
            );

            desc_text.push(vtext);
        }

        let result_report_string_table = result_report.create_table();

        let mut eval_mistakes_vtext = EffectableWrap::new(
            MovableWrap::new(
                Box::new(VerticalText::new(
                    result_report_string_table.condition_eval_mistakes,
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    font_info.clone(),
                )),
                None,
                t,
            ),
            vec![effect::appear_bale_down_from_top(
                100,
                t + effect_clock_offset + 150,
            )],
        );
        ctx.savable_data.award_data.returning_check_mistake_count +=
            result_report.get_conition_eval_mistakes() as u16;

        eval_mistakes_vtext.set_crop(init_crop);
        set_table_frame_cell_center!(
            ctx.context,
            eval_frame,
            eval_mistakes_vtext,
            numeric::Vector2u::new(1, 1)
        );
        result_text.push(eval_mistakes_vtext);

	let mut total_eval_text = EffectableWrap::new(
            MovableWrap::new(
                Box::new(VerticalText::new(
                    result_report.generate_eval_str().to_string(),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    font_info.clone(),
                )),
                None,
                t,
            ),
            vec![effect::appear_bale_down_from_top(
                100,
                t + effect_clock_offset + 200,
            )],
        );

        total_eval_text.set_crop(init_crop);
        set_table_frame_cell_center!(
            ctx.context,
            eval_frame,
            total_eval_text,
            numeric::Vector2u::new(0, 1)
        );
        result_text.push(total_eval_text);

        let mut shelving_vtext = EffectableWrap::new(
            MovableWrap::new(
                Box::new(VerticalText::new(
                    result_report_string_table.shelving_is_done,
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    font_info.clone(),
                )),
                None,
                t,
            ),
            vec![effect::appear_bale_down_from_top(
                100,
                t + effect_clock_offset + 100,
            )],
        );

        shelving_vtext.set_crop(init_crop);
        set_table_frame_cell_center!(
            ctx.context,
            eval_frame,
            shelving_vtext,
            numeric::Vector2u::new(2, 1)
        );
        result_text.push(shelving_vtext);

        let mut waiting_vtext = EffectableWrap::new(
            MovableWrap::new(
                Box::new(VerticalText::new(
                    result_report_string_table.total_customers_waiting_time,
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    font_info.clone(),
                )),
                None,
                t,
            ),
            vec![effect::appear_bale_down_from_top(
                100,
                t + effect_clock_offset + 50,
            )],
        );
        waiting_vtext.set_crop(init_crop);

        set_table_frame_cell_center!(
            ctx.context,
            eval_frame,
            waiting_vtext,
            numeric::Vector2u::new(3, 1)
        );
        result_text.push(waiting_vtext);

        DrawableEvaluationFlow {
            eval_frame: eval_frame,
            desc_text: desc_text,
            result_text: result_text,
            drwob_essential: DrawableObjectEssential::new(true, depth),
        }
    }

    ///
    /// # 描画要求有り
    ///
    pub fn run_effect<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        for vtext in self.result_text.iter_mut() {
            if vtext.is_empty_effect() && vtext.is_stop() {
                continue;
            }

            vtext.effect(ctx.context, t);
            ctx.process_utility.redraw();
        }
    }
}

impl DrawableComponent for DrawableEvaluationFlow {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.eval_frame.draw(ctx)?;

            for vtext in self.desc_text.iter_mut() {
                vtext.draw(ctx)?;
            }

            for vtext in self.result_text.iter_mut() {
                vtext.draw(ctx)?;
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn hide(&mut self) {
        self.drwob_essential.visible = false;
    }

    #[inline(always)]
    fn appear(&mut self) {
        self.drwob_essential.visible = true;
    }

    #[inline(always)]
    fn is_visible(&self) -> bool {
        self.drwob_essential.visible
    }

    #[inline(always)]
    fn set_drawing_depth(&mut self, depth: i8) {
        self.drwob_essential.drawing_depth = depth;
    }

    #[inline(always)]
    fn get_drawing_depth(&self) -> i8 {
        self.drwob_essential.drawing_depth
    }
}

pub struct DrawableTaskResult {
    result_frame: TableFrame,
    fixed_text: Vec<VerticalText>,
    effect_text: Vec<EffectableWrap<MovableWrap<VerticalText>>>,
    meters: ResultMeter,
    background: SimpleObject,
    evaluation: DrawableEvaluationFlow,
    const_canvas: SubScreen,
    canvas: SubScreen,
}

impl DrawableTaskResult {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        rect_pos: numeric::Rect,
        background: SimpleObject,
        initial_save_data: SavableData,
        result_report: ResultReport,
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
            numeric::Vector2f::new(32.0, 32.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );
        let init_crop = numeric::Rect::new(0.0, 0.0, 1.0, 0.0);

        let result_frame = TableFrame::new(
            ctx.resource,
            numeric::Point2f::new(750.0, 80.0),
            TileBatchTextureID::OldStyleFrame,
            FrameData::new(vec![280.0, 270.0], vec![50.0; 3]),
            numeric::Vector2f::new(0.5, 0.5),
            0,
        );

        let mut fixed_text = Vec::new();
        let mut effect_text = Vec::new();

        let title_text = VerticalText::new(
            format!("{}", date.to_string()),
            numeric::Point2f::new(1100.0, 80.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            font_info_large,
        );
        fixed_text.push(title_text);

        let title_desc_text = VerticalText::new(
            "御仕事結果".to_string(),
            numeric::Point2f::new(1040.0, 300.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            font_info_large,
        );
        fixed_text.push(title_desc_text);

        let mut done_work_text = VerticalText::new(
            "御客人数".to_string(),
            numeric::Point2f::new(600.0, 100.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            font_info,
        );
        set_table_frame_cell_center!(
            ctx.context,
            result_frame,
            done_work_text,
            numeric::Vector2u::new(2, 0)
        );
        fixed_text.push(done_work_text);

        let done_work_num = task_result.done_works - initial_save_data.task_result.done_works;
        let mut done_work_num_text = EffectableWrap::new(
            MovableWrap::new(
                Box::new(VerticalText::new(
                    format!("{}人", number_to_jk(done_work_num as u64)),
                    numeric::Point2f::new(600.0, 100.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    font_info,
                )),
                None,
                t,
            ),
            vec![effect::appear_bale_down_from_top(100, t + 150)],
        );
        done_work_num_text.set_crop(init_crop);
        set_table_frame_cell_center!(
            ctx.context,
            result_frame,
            done_work_num_text,
            numeric::Vector2u::new(2, 1)
        );
        effect_text.push(done_work_num_text);
        ctx.savable_data.award_data.customer_count += done_work_num as u16;

        let mut money_desc_text = VerticalText::new(
            format!("収入"),
            numeric::Point2f::new(480.0, 100.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            font_info,
        );
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
                    format!(
                        "{}円",
                        number_to_jk(
                            (task_result.total_money - initial_save_data.task_result.total_money)
                                as u64
                        )
                    ),
                    numeric::Point2f::new(600.0, 100.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    font_info,
                )),
                None,
                t,
            ),
            vec![effect::appear_bale_down_from_top(100, t + 250)],
        );
        money_text.set_crop(init_crop);
        set_table_frame_cell_center!(
            ctx.context,
            result_frame,
            money_text,
            numeric::Vector2u::new(1, 1)
        );
        effect_text.push(money_text);

        let mut total_money_desc_text = VerticalText::new(
            format!("所持金"),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            font_info,
        );
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
                    format!(
                        "{}円",
                        number_to_jk(ctx.savable_data.task_result.total_money as u64)
                    ),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    font_info,
                )),
                None,
                t,
            ),
            vec![effect::appear_bale_down_from_top(100, t + 350)],
        );
        total_money_text.set_crop(init_crop);
        set_table_frame_cell_center!(
            ctx.context,
            result_frame,
            total_money_text,
            numeric::Vector2u::new(0, 1)
        );
        effect_text.push(total_money_text);

        let mut meters = ResultMeter::new(
            ctx,
            "評判".to_string(),
            numeric::Rect::new(350.0, 650.0, 500.0, 60.0),
            6.0,
            100.0,
            initial_save_data.suzunaan_status.reputation,
            1,
        );
        meters.set_goal(ctx, ctx.savable_data.suzunaan_status.reputation, 100);

        let evaluation = DrawableEvaluationFlow::new(
            ctx,
            numeric::Point2f::new(250.0, 80.0),
            result_report,
            350 as Clock,
            0,
            t,
        );

        let mut this = DrawableTaskResult {
            result_frame: result_frame,
            effect_text: effect_text,
            fixed_text: fixed_text,
            meters: meters,
            evaluation: evaluation,
            background: background,
            const_canvas: SubScreen::new(
                ctx.context,
                rect_pos,
                0,
                ggraphics::Color::from_rgba_u32(0),
            ),
            canvas: SubScreen::new(ctx.context, rect_pos, 0, ggraphics::Color::from_rgba_u32(0)),
        };
        this.standby_const_canvas(ctx);
        this
    }

    fn standby_const_canvas<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        sub_screen::stack_screen(ctx.context, &self.const_canvas);

        self.background.draw(ctx.context).unwrap();
        self.result_frame.draw(ctx.context).unwrap();

        for vtext in self.fixed_text.iter_mut() {
            vtext.draw(ctx.context).unwrap();
        }

        sub_screen::pop_screen(ctx.context);
        self.canvas.draw(ctx.context).unwrap();
    }

    pub fn run_effect<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        for vtext in self.effect_text.iter_mut() {
            if vtext.is_empty_effect() && vtext.is_stop() {
                continue;
            }

            ctx.process_utility.redraw();
            vtext.effect(ctx.context, t);
        }

        self.evaluation.run_effect(ctx, t);

        self.meters.effect(ctx);
    }
}

impl DrawableComponent for DrawableTaskResult {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.const_canvas.draw(ctx)?;

            for vtext in self.effect_text.iter_mut() {
                vtext.draw(ctx).unwrap();
            }

            self.meters.draw(ctx)?;
            self.evaluation.draw(ctx)?;

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
