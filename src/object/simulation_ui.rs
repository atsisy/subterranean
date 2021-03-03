use torifune::core::Clock;
use torifune::graphics::drawable::*;
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::graphics::object::*;
use torifune::numeric;

use torifune::graphics::object::shape as tshape;

use ggez::graphics as ggraphics;

use super::*;
use crate::object::util_object::*;
use crate::set_table_frame_cell_center;
use crate::{
    core::{FontID, TextureID, TileBatchTextureID},
    scene::DrawRequest,
};
use torifune::{mintp, roundup2f};

use number_to_jk::number_to_jk;

struct Counter<T> {
    count: T,
}

impl<T: Clone + Copy + std::ops::AddAssign> Counter<T> {
    pub fn new(init: T) -> Self {
        Counter { count: init }
    }

    pub fn add(&mut self, value: T) {
        self.count += value;
    }

    pub fn set_value(&mut self, value: T) {
        self.count = value;
    }

    pub fn get_value(&self) -> T {
        self.count
    }
}

pub struct Meter {
    counter: Counter<f32>,
    max: f32,
    frame: tshape::Rectangle,
    empty_fill: tshape::Rectangle,
    count_fill: tshape::Rectangle,
    position: numeric::Point2f,
    drwob_essential: DrawableObjectEssential,
}

impl Meter {
    pub fn new(
        pos: numeric::Point2f,
        frame: numeric::Rect,
        frame_color: ggraphics::Color,
        count_frame: numeric::Rect,
        count_color: ggraphics::Color,
        remain_color: ggraphics::Color,
        init: f32,
        max: f32,
    ) -> Meter {
        Meter {
            counter: Counter::<f32>::new(init),
            max: max,
            frame: tshape::Rectangle::new(
                frame,
                ggez::graphics::DrawMode::Fill(ggraphics::FillOptions::DEFAULT),
                frame_color,
            ),
            empty_fill: tshape::Rectangle::new(
                count_frame,
                ggez::graphics::DrawMode::Fill(ggraphics::FillOptions::DEFAULT),
                count_color,
            ),
            count_fill: tshape::Rectangle::new(
                numeric::Rect::new(
                    count_frame.x,
                    count_frame.y,
                    count_frame.w * (init / max),
                    count_frame.h,
                ),
                ggez::graphics::DrawMode::Fill(ggraphics::FillOptions::DEFAULT),
                remain_color,
            ),
            position: pos,
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }

    pub fn add(&mut self, value: f32) {
        self.counter.add(value);

        let mut counter_rect = self.count_fill.get_bounds();
        counter_rect.w = self.empty_fill.get_bounds().w * (self.counter.get_value() / self.max);

        let new_fill = tshape::Rectangle::new(
            counter_rect,
            self.count_fill.get_mode(),
            self.count_fill.get_color(),
        );

        self.count_fill = new_fill;
    }

    pub fn get_value(&self) -> f32 {
        self.counter.get_value()
    }

    pub fn set_value(&mut self, value: f32) {
        if self.get_value() < self.max {
            self.counter.set_value(value);
        } else {
            self.counter.set_value(self.max);
        }
    }
}

impl DrawableComponent for Meter {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        let mesh = ggraphics::MeshBuilder::new()
            .rectangle(
                self.frame.get_mode(),
                self.frame.get_bounds(),
                self.frame.get_color(),
            ).expect("failed to create rectangle")
            .rectangle(
                self.empty_fill.get_mode(),
                self.empty_fill.get_bounds(),
                self.empty_fill.get_color(),
            ).expect("failed to create rectangle")
            .rectangle(
                self.count_fill.get_mode(),
                self.count_fill.get_bounds(),
                self.count_fill.get_color(),
            ).expect("failed to create rectangle")
            .build(ctx)?;

        ggraphics::draw(
            ctx,
            &mesh,
            ggraphics::DrawParam::default().dest(mintp!(self.position)),
        )
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

///
/// メニューに表示するやつ
///
pub struct ScenarioMenuContents {
    table_frame: TableFrame,
    desc_text: Vec<VerticalText>,
    reputation_text: VerticalText,
    money_text: VerticalText,
    day_text: UniText,
    kosuzu_level_text: VerticalText,
    drwob_essential: DrawableObjectEssential,
}

impl ScenarioMenuContents {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>) -> Self {
        let normal_scale_font = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(30.0, 30.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );

        let large_scale_font = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(40.0, 40.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );

        let table_frame = TableFrame::new(
            ctx.resource,
            numeric::Point2f::new(50.0, 150.0),
            TileBatchTextureID::OldStyleFrame,
            FrameData::new(vec![250.0, 250.0], vec![50.0; 3]),
            numeric::Vector2f::new(0.25, 0.25),
            0,
        );

        let mut desc_text = Vec::new();

        for (index, s) in vec!["評判", "習熟度", "所持金"].iter().enumerate() {
            let mut vtext = VerticalText::new(
                s.to_string(),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                normal_scale_font,
            );

            set_table_frame_cell_center!(
                ctx.context,
                table_frame,
                vtext,
                numeric::Vector2u::new(index as u32, 0)
            );

            desc_text.push(vtext);
        }

        let mut reputation_text = VerticalText::new(
            number_to_jk(ctx.take_save_data().suzunaan_status.reputation as u64),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            normal_scale_font,
        );

        set_table_frame_cell_center!(
            ctx.context,
            table_frame,
            reputation_text,
            numeric::Vector2u::new(0, 1)
        );

        let mut money_text = VerticalText::new(
            format!(
                "{}円",
                number_to_jk(ctx.take_save_data().task_result.total_money as u64)
            ),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            normal_scale_font,
        );

        set_table_frame_cell_center!(
            ctx.context,
            table_frame,
            money_text,
            numeric::Vector2u::new(2, 1)
        );

        let mut kosuzu_level_text = VerticalText::new(
            format!("{}", number_to_jk(0)),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            normal_scale_font,
        );

        set_table_frame_cell_center!(
            ctx.context,
            table_frame,
            kosuzu_level_text,
            numeric::Vector2u::new(1, 1)
        );

        ScenarioMenuContents {
            table_frame: table_frame,
            reputation_text: reputation_text,
            desc_text: desc_text,
            day_text: UniText::new(
                format!(
                    "{}, {}",
                    ctx.take_save_data().date.day,
                    ctx.take_save_data().date.to_month_string_eng_short()
                ),
                numeric::Point2f::new(40.0, 70.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                large_scale_font,
            ),
            money_text: money_text,
            kosuzu_level_text: kosuzu_level_text,
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }
}

impl DrawableComponent for ScenarioMenuContents {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.table_frame.draw(ctx).unwrap();
            self.day_text.draw(ctx).unwrap();

            for vtext in self.desc_text.iter_mut() {
                vtext.draw(ctx).unwrap();
            }

            self.reputation_text.draw(ctx).unwrap();
            self.money_text.draw(ctx).unwrap();

            self.kosuzu_level_text.draw(ctx).unwrap();
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

pub struct ScenarioMenu {
    canvas: SubScreen,
    contents: ScenarioMenuContents,
    background: UniTexture,
}

impl ScenarioMenu {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, size: numeric::Vector2f) -> Self {
        ScenarioMenu {
            canvas: SubScreen::new(
                ctx.context,
                numeric::Rect::new(0.0, 0.0, size.x, size.y),
                0,
                ggraphics::Color::from_rgba_u32(0xffffffff),
            ),
            background: UniTexture::new(
                ctx.ref_texture(TextureID::MenuArt1),
                numeric::Point2f::new(size.x - 1366.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
            ),
            contents: ScenarioMenuContents::new(ctx),
        }
    }
}

impl DrawableComponent for ScenarioMenu {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        sub_screen::stack_screen(ctx, &self.canvas);

        self.background.draw(ctx)?;
        self.contents.draw(ctx)?;

        sub_screen::pop_screen(ctx);
        self.canvas.draw(ctx)
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

pub struct ResultMeter {
    meter: Meter,
    desc_text: UniText,
    diff_text: Option<UniText>,
    current_value_text: UniText,
    drwob_essential: DrawableObjectEssential,
    goal: f32,
    diff_per_clock: f32,
}

impl ResultMeter {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        title: String,
        pos: numeric::Rect,
        padding: f32,
        max: f32,
        current: f32,
        depth: i8,
    ) -> Self {
        let meter = Meter::new(
            numeric::Point2f::new(pos.x, pos.y + 25.0),
            numeric::Rect::new(0.0, 0.0, pos.w, pos.h),
            ggraphics::Color::from_rgba_u32(0x362d33ff),
            numeric::Rect::new(
                padding,
                padding,
                pos.w - (2.0 * padding),
                pos.h - (2.0 * padding),
            ),
            ggraphics::Color::from_rgba_u32(0x463d43ff),
            ggraphics::Color::from_rgba_u32(0xb83f36ff),
            current,
            max,
        );

        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(20.0, 20.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let mut current_value_text = UniText::new(
            format!("{}", current),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            font_info.clone(),
        );
        current_value_text.set_position(numeric::Point2f::new(
            pos.right() - current_value_text.get_drawing_size(ctx.context).x,
            pos.y,
        ));

	let rect_pos = pos.point();
        let desc_text = UniText::new(
            title,
	    numeric::Point2f::new(rect_pos.x, rect_pos.y),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            depth,
            font_info,
        );

        ResultMeter {
            meter: meter,
            desc_text: desc_text,
            diff_text: None,
            current_value_text: current_value_text,
            goal: current,
            drwob_essential: DrawableObjectEssential::new(true, depth),
            diff_per_clock: 0.0,
        }
    }

    pub fn set_goal<'a>(&mut self, ctx: &mut SuzuContext<'a>, goal: f32, time: Clock)  {
        let current = self.meter.get_value();
        let diff = if goal > self.meter.max {
            self.meter.max - current
        } else if goal < 0.0 {
            -current
        } else {
            goal - current
        };

        self.apply_offset(ctx, diff, time);
    }

    pub fn apply_offset<'a>(&mut self, ctx: &mut SuzuContext<'a>, diff: f32, time: Clock) {
        let pos = self.desc_text.get_position();

        self.diff_text = Some(UniText::new(
            if diff >= 0.0 {
                format!("{:+}", diff)
            } else {
                format!("{}", diff)
            },
            numeric::Point2f::new(pos.x + 100.0, pos.y),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                ctx.resource.get_font(FontID::Cinema),
                numeric::Vector2f::new(20.0, 20.0),
                if diff >= 0.0 {
                    ggraphics::Color::from_rgba_u32(0x00ff00ff)
                } else {
                    ggraphics::Color::from_rgba_u32(0xff0000ff)
                },
            ),
        ));

        let next_goal = self.meter.get_value() + diff;
        self.goal = if next_goal > self.meter.max {
            self.meter.max
        } else {
            next_goal
        };

        self.diff_per_clock = (self.goal - self.meter.get_value()) / time as f32;
    }

    pub fn effect<'a>(&mut self, ctx: &mut SuzuContext<'a>) -> DrawRequest {
	if self.diff_per_clock == 0.0 {
	    return DrawRequest::Skip;
	}

        if (self.meter.get_value() - self.goal).abs() >= self.diff_per_clock.abs() {
            self.meter.add(self.diff_per_clock);
        } else {
	    self.meter.set_value(self.goal);
	}

	let before_x = self.current_value_text.get_drawing_size(ctx.context).x;
	
        self.current_value_text
            .replace_text(format!("{}", self.meter.get_value() as i32));
	
        let after_x = self.current_value_text.get_drawing_size(ctx.context).x;
        self.current_value_text
            .move_diff(numeric::Vector2f::new(before_x - after_x, 0.0));
	
	ctx.process_utility.redraw();
	return DrawRequest::Draw;
    }
}

impl DrawableComponent for ResultMeter {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.meter.draw(ctx)?;
            self.desc_text.draw(ctx)?;
            self.current_value_text.draw(ctx)?;

            if let Some(diff_text) = self.diff_text.as_mut() {
                diff_text.draw(ctx)?;
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
