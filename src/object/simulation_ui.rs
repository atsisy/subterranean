use torifune::core::Clock;
use torifune::numeric;
use torifune::graphics::*;
use torifune::graphics::object::*;

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
}

impl<T: std::ops::AddAssign + Copy + std::fmt::Display> DrawableCounter<T> {
    pub fn new(init: T, pos: numeric::Point2f, font_info: FontInformation, t: Clock) -> Self {
        DrawableCounter {
            counter: Counter::<T>::new(init),
            text: SimpleText::new(MovableText::new(format!("{}", init),
                                                   pos,
                                                   numeric::Vector2f::new(1.0, 1.0),
                                                   0.0,
                                                   0,
                                                   move_fn::halt(pos),
                                                   font_info, t),
                            Vec::new())
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
        self.text.ref_wrapped_object().replace_text(&format!("{}", value))
    }
}

pub struct SimulationStatus {
    money_counter: DrawableCounter<u32>,
    canvas: SubScreen,
}

impl SimulationStatus {
    pub fn new(ctx: &mut ggez::Context, pos: numeric::Rect, game_data: &GameData) -> Self {
        SimulationStatus {
            money_counter: DrawableCounter::<u32>::new(25000 as u32, numeric::Point2f::new(0.0, 0.0),
                                                       FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                                            numeric::Vector2f::new(24.0, 24.0),
                                                                            ggraphics::Color::from_rgba_u32(0xffffffff)), 0),
            canvas: SubScreen::new(ctx, pos, 0, ggraphics::Color::new(0.5, 0.3, 0.2, 1.0)),
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
