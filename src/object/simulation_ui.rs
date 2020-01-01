use torifune::core::Clock;
use torifune::numeric;
use torifune::graphics::*;
use torifune::graphics::object::*;

use torifune::graphics::object::shape as tshape;

use ggez::graphics as ggraphics;

use crate::core::{TextureID, FontID, GameData};
use super::*;

struct Counter<T> {
    count: T,
}

impl<T: Clone + Copy + std::ops::AddAssign> Counter<T> {
    pub fn new(init: T) -> Self {
        Counter {
            count: init,
        }
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
    pub fn new(init: T, pos: numeric::Point2f, font_info: FontInformation,
                  display_method: Box<dyn Fn(T) -> String>, t: Clock) -> Self {
        DrawableCounter {
            counter: Counter::<T>::new(init),
            text: SimpleText::new(MovableText::new(display_method(init),
                                                   pos,
                                                   numeric::Vector2f::new(1.0, 1.0),
                                                   0.0,
                                                   0,
                                                   move_fn::halt(pos),
                                                   font_info, t),
                                  Vec::new()),
            display_method: display_method,
        }
    }
}

impl<T> DrawableComponent for DrawableCounter<T> {
    fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
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

impl<T: std::fmt::Display + std::ops::AddAssign + Clone + Copy + std::ops::AddAssign> DrawableCounter<T> {

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
        self.text.ref_wrapped_object().replace_text(&(self.display_method)(value))
    }
}

struct Meter {
    counter: Counter<f32>,
    max: f32,
    frame: tshape::Rectangle,
    empty_fill: tshape::Rectangle,
    count_fill: tshape::Rectangle,
    position: numeric::Point2f,
    drwob_essential: DrawableObjectEssential,
}

impl Meter {
    pub fn new(pos: numeric::Point2f,
               frame: numeric::Rect,
               frame_color: ggraphics::Color,
               count_frame: numeric::Rect,
               count_color: ggraphics::Color,
               remain_color: ggraphics::Color,
               init: f32,
               max: f32) -> Meter {
        Meter {
            counter: Counter::<f32>::new(init),
            max: max,
            frame: tshape::Rectangle::new(
                frame,
                ggez::graphics::DrawMode::Fill(ggraphics::FillOptions::DEFAULT),
                frame_color),
            empty_fill: tshape::Rectangle::new(
                count_frame,
                ggez::graphics::DrawMode::Fill(ggraphics::FillOptions::DEFAULT),
                count_color),
            count_fill: tshape::Rectangle::new(
                numeric::Rect::new(count_frame.x, count_frame.y, count_frame.w * (init / max), count_frame.h),
                ggez::graphics::DrawMode::Fill(ggraphics::FillOptions::DEFAULT),
                remain_color),
            position: pos,
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }

    pub fn get_counter(&self) -> &Counter<f32> {
        &self.counter
    }

    pub fn get_mut_counter(&mut self) -> &mut Counter<f32> {
        &mut self.counter
    }
    
    pub fn update(&mut self) {
        self.count_fill.get_bounds();
    }
}

impl DrawableComponent for Meter {
    fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        let mesh = ggraphics::MeshBuilder::new().rectangle(
            self.frame.get_mode(),
            self.frame.get_bounds(),
            self.frame.get_color()
        ).rectangle(
            self.empty_fill.get_mode(),
            self.empty_fill.get_bounds(),
            self.empty_fill.get_color()
        ).rectangle(
            self.count_fill.get_mode(),
            self.count_fill.get_bounds(),
            self.count_fill.get_color()
        ).build(ctx)?;

        ggraphics::draw(ctx, &mesh, ggraphics::DrawParam::default().dest(self.position))
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
    pub fn new(choice_text: Vec<&str>,
               textures: Vec<TextureID>,
               select_tid: TextureID,
               game_data: &GameData) -> Self {
        Choice {
            choice_text: choice_text.iter().map(|s| SimpleText::new(
                MovableText::new(s.to_string(),
                                 numeric::Point2f::new(0.0, 0.0),
                                 numeric::Vector2f::new(1.0, 1.0),
                                 0.0,
                                 0,
                                 move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                                 FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                      numeric::Vector2f::new(24.0, 24.0),
                                                      ggraphics::BLACK),
                                 0), Vec::new())).collect(),
            choice_texture: textures.iter().map(|tid| SimpleObject::new(
                MovableUniTexture::new(
                    game_data.ref_texture(*tid),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                    0), Vec::new())).collect(),
            selecting: SimpleObject::new(
                MovableUniTexture::new(
                    game_data.ref_texture(select_tid),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                    0), Vec::new()),
            select_index: 0,
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }
}


impl DrawableComponent for Choice {
    fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.choice_text.get(self.select_index).unwrap().draw(ctx)?;
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
}

impl SimulationStatus {
    pub fn new(ctx: &mut ggez::Context, pos: numeric::Rect, game_data: &GameData) -> Self {
        SimulationStatus {
            money_counter: DrawableCounter::<u32>::new(25000 as u32, numeric::Point2f::new(0.0, 0.0),
                                                       FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                                            numeric::Vector2f::new(24.0, 24.0),
                                                                            ggraphics::Color::from_rgba_u32(0xffffffff)),
                                                       Box::new(move |count| { format!("{}円", count) }), 0),
            tired_meter: Meter::new(numeric::Point2f::new(100.0, 10.0),
                                    numeric::Rect::new(0.0, 0.0, 200.0, 40.0),
                                    ggraphics::Color::from_rgba_u32(0x000000ff),
                                    numeric::Rect::new(10.0, 10.0, 180.0, 20.0),
                                    ggraphics::Color::from_rgba_u32(0x222222ff),
                                    ggraphics::Color::from_rgba_u32(0xddddddff),
                                    500.0, 1000.0),
            choice: Choice::new(vec!["test1", "test2"],
                                vec![TextureID::LotusBlue, TextureID::LotusPink],
                                TextureID::LotusYellow,
                                game_data),
            canvas: SubScreen::new(ctx, pos, 0, ggraphics::Color::from_rgba_u32(0xe6cde3ff)),
        }
    }

    pub fn update(&mut self) {
        self.money_counter.update_text();
    }
}

impl DrawableComponent for SimulationStatus {
    fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.canvas.begin_drawing(ctx);

            self.money_counter.draw(ctx)?;
            self.tired_meter.draw(ctx)?;

            self.choice.draw(ctx)?;
            
            self.canvas.end_drawing(ctx);
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
