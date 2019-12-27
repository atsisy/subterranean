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
    drwob_essential: DrawableObjectEssential,
}

impl TextBox {
    pub fn new(background: SimpleObject, font: FontInformation, t: Clock) -> Self {
        TextBox {
            current_text: SimpleText::new(
                MovableText::new("あ".to_string(),
                                 numeric::Point2f::new(0.0, 0.0),
                                 numeric::Vector2f::new(1.0, 1.0),
                                 0.0,
                                 0,
                                 move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                                 font, t),vec![]),
            background: background,
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }

    pub fn replace_text(&mut self, text: &str) {
        self.current_text.ref_wrapped_object().replace_text(text);
    }
}

impl DrawableComponent for TextBox {
    fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.drwob_essential.visible {
            self.background.draw(ctx)?;
            self.current_text.draw(ctx)?;
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

pub struct ScenarioText {
    text: String,
    fpc: u32,
    iterator: usize,
}

impl ScenarioText {
    pub fn new(text: &str, fpc: u32) -> Self {
        ScenarioText {
            text: text.to_string(),
            fpc: fpc,
            iterator: 0,
        }
    }

    pub fn new_fixed(text: &str) -> Self {
        ScenarioText {
            text: text.to_string(),
            fpc: 0,
            iterator: text.len(),
        }
    }

    pub fn slice_default(&self) -> Option<&str> {
        let mut i = 1;

        for (bytes, ch) in self.text.as_str().char_indices() {
            if i == self.iterator {
                return unsafe {
                    Some(self.text.as_str().slice_unchecked(0, bytes))
                };
            }
            i += 1;
        }

        None
    }

    pub fn update_iterator(&mut self) {
        self.iterator += self.fpc as usize;
        if self.iterator > self.text.chars().count() {
            self.iterator = self.text.chars().count();
        }
    }

    pub fn iterator_finish(&self) -> bool {
        self.iterator == self.text.len()
    }
}

struct RawScenarioSetting {
    tachie: Vec<u32>,
    scenario: Vec<ScenarioText>,
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
                table["fpc"].as_integer().unwrap() as u32));
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
    drwob_essential: DrawableObjectEssential,
}

impl ScenarioEvent {
    pub fn new(file_path: &str, game_data: &GameData, background: SimpleObject, t: Clock) -> Self {
        ScenarioEvent {
            scenario: Scenario::new(file_path, game_data),
            text_box: TextBox::new(background, FontInformation::new(
                game_data.get_font(FontID::DEFAULT),
                ggraphics::Scale {x: 30.0, y: 30.0}), t),
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }

    pub fn update_text(&mut self) {
        self.scenario.current_page_mut().update_iterator();
        if let Some(s) = self.scenario.current_page().slice_default() {
            self.text_box.replace_text(s);
        }
    }

    pub fn next_page(&mut self) {
        self.scenario.next_page();
    }
}

impl DrawableComponent for ScenarioEvent {
    fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        self.text_box.draw(ctx)?;
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
