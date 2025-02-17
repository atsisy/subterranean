use std::collections::VecDeque;

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
    yet_effect_text: VecDeque<EffectableWrap<MovableWrap<VerticalText>>>,
    effect_time_list: VecDeque<Clock>,
    now_effect_text: VecDeque<EffectableWrap<MovableWrap<VerticalText>>>,
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
	    ggraphics::FilterMode::Nearest,
            0,
        );
        let init_crop = numeric::Rect::new(0.0, 0.0, 1.0, 0.0);

        let mut desc_text = Vec::new();
        let mut yet_effect_text = VecDeque::new();

        let mut effect_time_list = VecDeque::new();
        effect_time_list.push_back(t + effect_clock_offset);
        effect_time_list.push_back(t + effect_clock_offset + 50);
        effect_time_list.push_back(t + effect_clock_offset + 100);
        effect_time_list.push_back(t + effect_clock_offset + 150);

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
            Vec::new(),
        );
        ctx.take_save_data_mut()
            .award_data
            .returning_check_mistake_count += result_report.get_conition_eval_mistakes() as u16;

        eval_mistakes_vtext.set_crop(init_crop);
        set_table_frame_cell_center!(
            ctx.context,
            eval_frame,
            eval_mistakes_vtext,
            numeric::Vector2u::new(1, 1)
        );

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
            Vec::new(),
        );

        total_eval_text.set_crop(init_crop);
        set_table_frame_cell_center!(
            ctx.context,
            eval_frame,
            total_eval_text,
            numeric::Vector2u::new(0, 1)
        );

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
            Vec::new(),
        );

        shelving_vtext.set_crop(init_crop);
        set_table_frame_cell_center!(
            ctx.context,
            eval_frame,
            shelving_vtext,
            numeric::Vector2u::new(2, 1)
        );

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
            Vec::new(),
        );
        waiting_vtext.set_crop(init_crop);

        set_table_frame_cell_center!(
            ctx.context,
            eval_frame,
            waiting_vtext,
            numeric::Vector2u::new(3, 1)
        );

        yet_effect_text.push_back(waiting_vtext);
        yet_effect_text.push_back(shelving_vtext);
        yet_effect_text.push_back(eval_mistakes_vtext);
        yet_effect_text.push_back(total_eval_text);

        DrawableEvaluationFlow {
            eval_frame: eval_frame,
            desc_text: desc_text,
            yet_effect_text: yet_effect_text,
            now_effect_text: VecDeque::new(),
            effect_time_list: effect_time_list,
            drwob_essential: DrawableObjectEssential::new(true, depth),
        }
    }

    ///
    /// # 描画要求有り
    ///
    pub fn run_effect<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        while !self.effect_time_list.is_empty() {
            if self.effect_time_list[0] <= t {
                let mut vtext = self.yet_effect_text.pop_front().unwrap();
                vtext.clear_effect();
                vtext.add_effect(vec![effect::appear_bale_down_from_top(50, t)]);
                self.now_effect_text.push_back(vtext);
                self.effect_time_list.pop_front();
            } else {
                break;
            }
        }

        for vtext in self.now_effect_text.iter_mut() {
            if vtext.is_empty_effect() && vtext.is_stop() {
                continue;
            }

            vtext.effect(ctx.context, t);
            ctx.process_utility.redraw();
        }
    }

    pub fn skip_effect<'a>(&mut self, t: Clock) {
        for vtext in self.now_effect_text.iter_mut() {
            vtext.clear_effect();
            vtext.set_crop(numeric::Rect::new(0.0, 0.0, 1.0, 1.0));
        }

        if let Some(next_effect_time) = self.effect_time_list.front() {
            let diff = next_effect_time - t;
            for effect_time in self.effect_time_list.iter_mut() {
                *effect_time -= diff;
            }
        }
    }

    pub fn is_done(&self) -> bool {
	for vtext in self.now_effect_text.iter() {
            if !vtext.is_empty_effect() || !vtext.is_stop() {
		return false;
            }

	}
	self.yet_effect_text.is_empty()
    }
}

impl DrawableComponent for DrawableEvaluationFlow {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.eval_frame.draw(ctx)?;

            for vtext in self.desc_text.iter_mut() {
                vtext.draw(ctx)?;
            }

            for vtext in self.yet_effect_text.iter_mut() {
                vtext.draw(ctx)?;
            }

            for vtext in self.now_effect_text.iter_mut() {
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
    effect_time_list: VecDeque<Clock>,
    fixed_text: Vec<VerticalText>,
    yet_effect_text: VecDeque<EffectableWrap<MovableWrap<VerticalText>>>,
    now_effect_text: VecDeque<EffectableWrap<MovableWrap<VerticalText>>>,
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

        let font_info_small = FontInformation::new(
            ctx.resource.get_font(FontID::JpFude1),
            numeric::Vector2f::new(24.0, 24.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );

        let init_crop = numeric::Rect::new(0.0, 0.0, 1.0, 0.0);

        let result_frame = TableFrame::new(
            ctx.resource,
            numeric::Point2f::new(750.0, 80.0),
            TileBatchTextureID::OldStyleFrame,
            FrameData::new(vec![280.0, 270.0], vec![50.0; 3]),
            numeric::Vector2f::new(0.5, 0.5),
	    ggraphics::FilterMode::Nearest,
            0,
        );

        let mut fixed_text = Vec::new();
        let mut effect_text = VecDeque::new();

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

        let mut effect_time_list = VecDeque::new();
        effect_time_list.push_back(t + 50);
        effect_time_list.push_back(t + 100);
        effect_time_list.push_back(t + 150);

        let task_result = ctx.take_save_data().task_result.clone();

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
            Vec::new(),
        );
        done_work_num_text.set_crop(init_crop);
        set_table_frame_cell_center!(
            ctx.context,
            result_frame,
            done_work_num_text,
            numeric::Vector2u::new(2, 1)
        );
        effect_text.push_back(done_work_num_text);
        ctx.take_save_data_mut()
            .award_data
            .add_customer_count(done_work_num as u16);

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
            Vec::new(),
        );
        money_text.set_crop(init_crop);
        set_table_frame_cell_center!(
            ctx.context,
            result_frame,
            money_text,
            numeric::Vector2u::new(1, 1)
        );
        effect_text.push_back(money_text);

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
                        number_to_jk(ctx.take_save_data().task_result.total_money as u64)
                    ),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    font_info_small,
                )),
                None,
                t,
            ),
            Vec::new(),
        );
        total_money_text.set_crop(init_crop);
        set_table_frame_cell_center!(
            ctx.context,
            result_frame,
            total_money_text,
            numeric::Vector2u::new(0, 1)
        );
        effect_text.push_back(total_money_text);

        let mut meters = ResultMeter::new(
            ctx,
            "評判".to_string(),
            numeric::Rect::new(350.0, 650.0, 500.0, 60.0),
            6.0,
            100.0,
            initial_save_data.suzunaan_status.get_current_reputation(),
            1,
        );

	let eval = result_report.generate_eval_result();
	ctx.take_save_data_mut().suzunaan_status.add_reputation(eval);
	let goal = ctx.take_save_data().suzunaan_status.get_current_reputation();
        let goal = if goal >= 0.0
        {
	    goal
        } else {
            0.0
        };

        meters.set_goal(ctx, goal, 100);

        let evaluation = DrawableEvaluationFlow::new(
            ctx,
            numeric::Point2f::new(250.0, 80.0),
            result_report,
            200 as Clock,
            0,
            t,
        );

        let mut this = DrawableTaskResult {
            effect_time_list: effect_time_list,
            result_frame: result_frame,
            yet_effect_text: effect_text,
            now_effect_text: VecDeque::new(),
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

    pub fn evaluation_flow_is_done(&self) -> bool {
	self.evaluation.is_done()
    }

    pub fn run_effect<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        while !self.effect_time_list.is_empty() {
            if self.effect_time_list[0] <= t {
                let mut vtext = self.yet_effect_text.pop_front().unwrap();
                vtext.clear_effect();
                vtext.add_effect(vec![effect::appear_bale_down_from_top(50, t)]);
                self.now_effect_text.push_back(vtext);
                self.effect_time_list.pop_front();
            } else {
                break;
            }
        }

        for vtext in self.now_effect_text.iter_mut() {
            if vtext.is_empty_effect() && vtext.is_stop() {
                continue;
            }

            ctx.process_utility.redraw();
            vtext.effect(ctx.context, t);
        }

        self.evaluation.run_effect(ctx, t);

        self.meters.effect(ctx);
    }

    pub fn click_handler<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        for vtext in self.now_effect_text.iter_mut() {
            vtext.clear_effect();
            vtext.set_crop(numeric::Rect::new(0.0, 0.0, 1.0, 1.0));
        }

        if let Some(next_effect_time) = self.effect_time_list.front() {
            let diff = next_effect_time - t;
            for effect_time in self.effect_time_list.iter_mut() {
                *effect_time -= diff;
            }
        } else {
            self.evaluation.skip_effect(t);
        }

	ctx.process_utility.redraw();
    }
}

impl DrawableComponent for DrawableTaskResult {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.const_canvas.draw(ctx)?;

            for vtext in self.yet_effect_text.iter_mut() {
                vtext.draw(ctx).unwrap();
            }

            for vtext in self.now_effect_text.iter_mut() {
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
