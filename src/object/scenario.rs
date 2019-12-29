use torifune::graphics::object::*;
use torifune::graphics::*;
use torifune::numeric;

use std::str::FromStr;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use crate::core::{TextureID, FontID, GameData};
use super::*;

pub struct TextBox {
    current_text: SimpleText,
    background: SimpleObject,
    canvas: SubScreen,
}

impl TextBox {
    pub fn new(ctx: &mut ggez::Context, rect: numeric::Rect,
               mut background: SimpleObject, font: FontInformation,
               pos: numeric::Point2f, t: Clock) -> Self {
        background.fit_scale(ctx, numeric::Vector2f::new(rect.w, rect.h));
        background.set_position(numeric::Point2f::new(0.0, 0.0));
        TextBox {
            current_text: SimpleText::new(
                MovableText::new("".to_string(),
                                 pos,
                                 numeric::Vector2f::new(1.0, 1.0),
                                 0.0,
                                 0,
                                 move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                                 font, t),vec![]),
            background: background,
            canvas: SubScreen::new(ctx, rect, 0, ggraphics::Color::new(0.0, 0.0, 0.0, 0.0)),
        }
    }

    pub fn replace_text(&mut self, text: &str) {
        self.current_text.ref_wrapped_object().replace_text(text);
    }
}

impl DrawableComponent for TextBox {
    fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.canvas.begin_drawing(ctx);
            
            self.background.draw(ctx)?;
            self.current_text.draw(ctx)?;

            self.canvas.end_drawing(ctx);
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

pub struct ScenarioText {
    text: String,
    fpc: f32,
    iterator: f32,
}

impl ScenarioText {
    pub fn new(text: &str, fpc: f32) -> Self {
        ScenarioText {
            text: text.to_string(),
            fpc: fpc,
            iterator: 0.0,
        }
    }

    pub fn new_fixed(text: &str) -> Self {
        ScenarioText {
            text: text.to_string(),
            fpc: 0.0,
            iterator: text.bytes().count() as f32,
        }
    }

    fn current_iterator(&self) -> usize {
        self.iterator as usize
    }

    fn slice_text_bytes(&self, begin: usize, end: usize) -> &str {
        unsafe {
            self.text.as_str().get_unchecked(begin..end)
        }
    }
    
    pub fn slice_default(&self) -> &str {
        let mut reached_flag = false;

        let str_len = self.text.as_str().char_indices().count();
        let str_bytes = self.text.as_str().as_bytes().len();
        
        for (count, (bytes, ch)) in self.text.as_str().char_indices().enumerate() {
            if reached_flag {
                return self.slice_text_bytes(0, bytes as usize);
            }

            if count == self.current_iterator() {
                reached_flag = true;
            }
        }
        
        return self.slice_text_bytes(0, str_bytes as usize);
    }

    pub fn update_iterator(&mut self) {
        self.iterator += self.fpc;
        if self.current_iterator() > self.text.chars().count() {
            self.iterator = self.text.chars().count() as f32;
        }
    }

    pub fn iterator_finish(&self) -> bool {
        self.iterator == self.text.bytes().count() as f32
    }
}

pub struct Scenario {
    tachie: Vec<SimpleObject>,
    scenario: Vec<ScenarioText>,
    page: usize,
}

impl Scenario {
    pub fn new(file_path: &str, game_data: &GameData) -> Self {
        let mut tachie = Vec::new();
        let mut scenario = Vec::new();

        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => panic!("Failed to read: {}", file_path),
        };
        
        let root = content.parse::<toml::Value>().unwrap();
        
        let array = root["script"].as_array().unwrap();
        for elem in array {
            let table = elem.as_table().unwrap();
            scenario.push(ScenarioText::new(
                table["text"].as_str().unwrap(),
                table["fpc"].as_float().unwrap() as f32));
        }

        let tachie_array = root["using-tachie"].as_array().unwrap();
        for elem in tachie_array {
            let tid = TextureID::from_str(elem.as_str().unwrap());
            if let Ok(tid) = tid {
            tachie.push(SimpleObject::new(
                MovableUniTexture::new(
                    game_data.ref_texture(tid),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(0.1, 0.1),
                    0.0, 0, move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                    0), vec![]));
            }
        }

        Scenario {
            tachie: tachie,
            scenario: scenario,
            page: 0,
        }
    }

    pub fn next_page(&mut self) {
        if !self.final_page() {
            self.page += 1;
        }
    }

    pub fn final_page(&self) -> bool {
        self.scenario.len() - 1 == self.page
    }

    pub fn update_current_page(&mut self) {
        self.current_page_mut().update_iterator();
    }
    
    pub fn current_page(&self) -> &ScenarioText {
        &self.scenario.get(self.page).unwrap()
    }

    pub fn current_page_mut(&mut self) -> &mut ScenarioText {
        self.scenario.get_mut(self.page).unwrap()
    }
}

pub struct ScenarioEvent {
    scenario: Scenario,
    text_box: TextBox,
    canvas: SubScreen,
}

impl ScenarioEvent {
    pub fn new(ctx: &mut ggez::Context, rect: numeric::Rect, file_path: &str, game_data: &GameData, t: Clock) -> Self {
        let background = tobj::SimpleObject::new(
            tobj::MovableUniTexture::new(
                game_data.ref_texture(TextureID::TextBackground),
                numeric::Point2f::new(20.0, 20.0),
                numeric::Vector2f::new(0.8, 0.8),
                0.0,
                0,
                move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                0), Vec::new());
        ScenarioEvent {
            scenario: Scenario::new(file_path, game_data),
            text_box: TextBox::new(
                ctx,
                numeric::Rect::new(rect.x + 10.0, rect.y + 10.0, rect.w - 20.0, rect.h - 20.0),
                background, FontInformation::new(
                game_data.get_font(FontID::DEFAULT),
                ggraphics::Scale {x: 30.0, y: 30.0}),
                                   numeric::Point2f::new(100.0, 100.0), t),
            canvas: SubScreen::new(ctx, rect, 0, ggraphics::Color::new(0.0, 0.0, 0.0, 0.0)),
        }
    }

    pub fn update_text(&mut self) {
        self.scenario.current_page_mut().update_iterator();
        let s = self.scenario.current_page().slice_default();
        self.text_box.replace_text(s);
    }

    pub fn next_page(&mut self) {
        self.scenario.next_page();
    }
}

impl DrawableComponent for ScenarioEvent {
    fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.canvas.begin_drawing(ctx);

            self.text_box.draw(ctx)?;
            
            self.canvas.end_drawing(ctx);
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
