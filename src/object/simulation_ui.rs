use torifune::core::Clock;
use torifune::graphics::drawable::*;
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::graphics::object::*;
use torifune::numeric;

use torifune::graphics::object::shape as tshape;

use ggez::graphics as ggraphics;

use super::*;
use crate::core::{FontID, TextureID, TileBatchTextureID};
use crate::object::util_object::*;
use crate::set_table_frame_cell_center;
use torifune::roundup2f;

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

struct DrawableCounter<T> {
    counter: Counter<T>,
    text: SimpleText,
    display_method: Box<dyn Fn(T) -> String>,
}

impl<T: std::ops::AddAssign + Copy + std::fmt::Display> DrawableCounter<T> {
    pub fn new(
        init: T,
        pos: numeric::Point2f,
        font_info: FontInformation,
        display_method: Box<dyn Fn(T) -> String>,
        t: Clock,
    ) -> Self {
        DrawableCounter {
            counter: Counter::<T>::new(init),
            text: SimpleText::new(
                MovableText::new(
                    Box::new(tobj::UniText::new(
                        display_method(init),
                        pos,
                        numeric::Vector2f::new(1.0, 1.0),
                        0.0,
                        0,
                        font_info,
                    )),
                    None,
                    t,
                ),
                Vec::new(),
            ),
            display_method: display_method,
        }
    }
}

impl<T> DrawableComponent for DrawableCounter<T> {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        self.text.draw(ctx)
    }

    fn hide(&mut self) {
        self.text.hide();
    }

    fn appear(&mut self) {
        self.text.appear();
    }

    fn is_visible(&self) -> bool {
        self.text.is_visible()
    }

    fn set_drawing_depth(&mut self, depth: i8) {
        self.text.set_drawing_depth(depth);
    }

    fn get_drawing_depth(&self) -> i8 {
        self.text.get_drawing_depth()
    }
}

impl<T: std::fmt::Display + std::ops::AddAssign + Clone + Copy + std::ops::AddAssign>
    DrawableCounter<T>
{
    pub fn add(&mut self, value: T) {
        self.counter.add(value);
    }

    pub fn set_value(&mut self, value: T) {
        self.counter.set_value(value);
    }

    pub fn get_value(&self) -> T {
        self.counter.get_value()
    }

    pub fn update_text(&mut self) {
        let value = self.get_value();
        self.text.replace_text(&(self.display_method)(value))
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
	    self.count_fill.get_color()
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
            )
            .rectangle(
                self.empty_fill.get_mode(),
                self.empty_fill.get_bounds(),
                self.empty_fill.get_color(),
            )
            .rectangle(
                self.count_fill.get_mode(),
                self.count_fill.get_bounds(),
                self.count_fill.get_color(),
            )
            .build(ctx)?;

        ggraphics::draw(
            ctx,
            &mesh,
            ggraphics::DrawParam::default().dest(self.position),
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

struct Choice {
    choice_text: Vec<SimpleText>,
    choice_texture: Vec<SimpleObject>,
    selecting: SimpleObject,
    select_index: usize,
    drwob_essential: DrawableObjectEssential,
}

impl Choice {
    pub fn new<'a>(
	ctx: &mut SuzuContext<'a>,
        choice_text: Vec<&str>,
        textures: Vec<TextureID>,
        select_tid: TextureID,
    ) -> Self {
        Choice {
            choice_text: choice_text
                .iter()
                .map(|s| {
                    SimpleText::new(
                        MovableText::new(
                            Box::new(UniText::new(
                                s.to_string(),
                                numeric::Point2f::new(0.0, 0.0),
                                numeric::Vector2f::new(1.0, 1.0),
                                0.0,
                                0,
                                FontInformation::new(
                                    ctx.resource.get_font(FontID::DEFAULT),
                                    numeric::Vector2f::new(24.0, 24.0),
                                    ggraphics::BLACK,
                                ),
                            )),
                            None,
                            0,
                        ),
                        Vec::new(),
                    )
                })
                .collect(),
            choice_texture: textures
                .iter()
                .map(|tid| {
                    SimpleObject::new(
                        MovableUniTexture::new(
                            ctx.ref_texture(*tid),
                            numeric::Point2f::new(0.0, 0.0),
                            numeric::Vector2f::new(1.0, 1.0),
                            0.0,
                            0,
                            move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                            0,
                        ),
                        Vec::new(),
                    )
                })
                .collect(),
            selecting: SimpleObject::new(
                MovableUniTexture::new(
                    ctx.ref_texture(select_tid),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                    0,
                ),
                Vec::new(),
            ),
            select_index: 0,
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }
}

impl DrawableComponent for Choice {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.choice_text
                .get_mut(self.select_index)
                .unwrap()
                .draw(ctx)?;
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

pub struct SimulationStatus {
    money_counter: DrawableCounter<u32>,
    tired_meter: Meter,
    choice: Choice,
    canvas: SubScreen,
    background: MovableUniTexture,
}

impl SimulationStatus {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, pos: numeric::Rect) -> Self {
        SimulationStatus {
            money_counter: DrawableCounter::<u32>::new(
                25000 as u32,
                numeric::Point2f::new(0.0, 0.0),
                FontInformation::new(
                    ctx.resource.get_font(FontID::DEFAULT),
                    numeric::Vector2f::new(24.0, 24.0),
                    ggraphics::Color::from_rgba_u32(0xffffffff),
                ),
                Box::new(move |count| format!("{}円", count)),
                0,
            ),
            tired_meter: Meter::new(
                numeric::Point2f::new(800.0, 10.0),
                numeric::Rect::new(0.0, 0.0, 200.0, 40.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
                numeric::Rect::new(10.0, 10.0, 180.0, 20.0),
                ggraphics::Color::from_rgba_u32(0x222222ff),
                ggraphics::Color::from_rgba_u32(0xddddddff),
                500.0,
                1000.0,
            ),
            choice: Choice::new(
		ctx,
                vec!["test1", "test2"],
                vec![TextureID::LotusBlue, TextureID::LotusPink],
                TextureID::LotusYellow,
            ),
            canvas: SubScreen::new(
                ctx.context,
                pos,
                0,
                ggraphics::Color::from_rgba_u32(0xe6cde3ff),
            ),
            background: MovableUniTexture::new(
                ctx.ref_texture(TextureID::WafuTexture2),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                move_fn::stop(),
                0,
            ),
        }
    }

    pub fn update(&mut self) {
        self.money_counter.update_text();
    }
}

impl DrawableComponent for SimulationStatus {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.background.draw(ctx)?;
            self.money_counter.draw(ctx)?;
            self.tired_meter.draw(ctx)?;

            self.choice.draw(ctx)?;

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
            number_to_jk(ctx.savable_data.suzunaan_status.reputation as u64),
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
                number_to_jk(ctx.savable_data.task_result.total_money as u64)
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
                    ctx.savable_data.date.day,
                    ctx.savable_data.date.to_month_string_eng_short()
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
