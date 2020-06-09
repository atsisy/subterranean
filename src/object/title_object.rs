use std::collections::HashMap;

use std::str::FromStr;

use ggez::graphics as ggraphics;

use torifune::graphics::drawable::*;
use torifune::graphics::object::*;
use torifune::numeric;

use crate::core::{font_information_from_toml_value, SuzuContext};
use crate::scene::SceneID;

#[derive(Clone, Copy)]
pub enum TitleBuiltinCommand {
    Exit,
}

impl FromStr for TitleBuiltinCommand {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "exit" => Ok(TitleBuiltinCommand::Exit),
            _ => Err(()),
        }
    }
}

#[derive(Clone)]
pub enum TitleContentsEvent {
    NextContents(String),
    SceneTransition(SceneID),
    BuiltinEvent(TitleBuiltinCommand),
}

impl TitleContentsEvent {
    pub fn from_toml_value(toml_value: &toml::Value) -> Option<Self> {
        let s = toml_value["event-type"].as_str().unwrap();

        match s {
            "SceneTransition" => {
                let next_scene_str = toml_value["next-scene"].as_str().expect("error");
                let next_scene = SceneID::from_str(next_scene_str).expect("Unknown next scene");
                Some(TitleContentsEvent::SceneTransition(next_scene))
            },
            "NextContents" => {
                let next_scene_str = toml_value["next-contents-name"]
                    .as_str()
                    .expect("error")
                    .to_string();
                Some(TitleContentsEvent::NextContents(next_scene_str))
            },
	    "BuiltinCommand" => {
		let command = toml_value["builtin-command"]
                    .as_str()
                    .expect("error");
                Some(TitleContentsEvent::BuiltinEvent(TitleBuiltinCommand::from_str(command).unwrap()))
	    },
            _ => None,
        }
    }
}

pub struct TextMenuEntryData {
    text: String,
    content_event: TitleContentsEvent,
}

impl TextMenuEntryData {
    pub fn from_toml_value(toml_value: &toml::Value) -> Self {
        TextMenuEntryData {
            text: toml_value["text"].as_str().unwrap().to_string(),
            content_event: TitleContentsEvent::from_toml_value(toml_value).unwrap(),
        }
    }
}

pub struct TextMenuData {
    contents_name: String,
    position: numeric::Point2f,
    padding: f32,
    entries_data: Vec<TextMenuEntryData>,
    normal_font_info: FontInformation,
    large_font_info: FontInformation,
}

impl TextMenuData {
    pub fn from_file<'a>(
        ctx: &mut SuzuContext<'a>,
        contents_name: String,
        file_path: &str,
    ) -> Self {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => panic!("Failed to read: {}", file_path),
        };

        let root = content.parse::<toml::Value>().unwrap();
        let entry_data_set = root["each_entry_data"].as_array().unwrap();
        let mut entries_data = Vec::new();

        for entry_data in entry_data_set {
            let data = TextMenuEntryData::from_toml_value(entry_data);
            entries_data.push(data);
        }

        let toml_position_table = root["position"].as_table().unwrap();
        let position = numeric::Point2f::new(
            toml_position_table["x"].as_float().unwrap() as f32,
            toml_position_table["y"].as_float().unwrap() as f32,
        );

        let padding = root["padding"].as_float().unwrap() as f32;

        let normal_font_info = font_information_from_toml_value(ctx.resource, &root["normal_font"]);
        let large_font_info = font_information_from_toml_value(ctx.resource, &root["large_font"]);

        TextMenuData {
            contents_name: contents_name,
            entries_data: entries_data,
            position: position,
            padding: padding,
            normal_font_info: normal_font_info,
            large_font_info: large_font_info,
        }
    }
}

pub struct VTextList {
    contents_name: String,
    vtext_list: Vec<VerticalText>,
    menu_entries_data: Vec<TextMenuEntryData>,
    normal_font: FontInformation,
    large_font: FontInformation,
    drwob_essential: DrawableObjectEssential,
}

impl VTextList {
    pub fn new<'a>(text_menu_data: TextMenuData, drawing_depth: i8) -> Self {
        let mut vtext_list = Vec::new();
        let mut position = text_menu_data.position;

        let normal_font_info = text_menu_data.normal_font_info.clone();
        let large_font_info = text_menu_data.large_font_info.clone();

        for content_data in text_menu_data.entries_data.iter().rev() {
            let text = content_data.text.to_string();

            let vtext = VerticalText::new(
                text,
                position,
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                normal_font_info.clone(),
            );

            vtext_list.push(vtext);
            position.x += normal_font_info.scale.x + text_menu_data.padding;
        }

        VTextList {
            contents_name: text_menu_data.contents_name,
            menu_entries_data: text_menu_data.entries_data,
            vtext_list: vtext_list,
            normal_font: normal_font_info,
            large_font: large_font_info,
            drwob_essential: DrawableObjectEssential::new(true, drawing_depth),
        }
    }

    pub fn update_highlight<'a>(&mut self, ctx: &mut SuzuContext<'a>, point: numeric::Point2f) {
        for vtext in self.vtext_list.iter_mut() {
            if vtext.contains(ctx.context, point) {
                vtext.set_color(ggraphics::Color::from_rgba_u32(0xddddddff));
            } else {
                vtext.set_color(ggraphics::Color::from_rgba_u32(0xbbbbbbff));
            }
        }
    }

    pub fn click_handler<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
    ) -> Option<TitleContentsEvent> {
        for (index, vtext) in self.vtext_list.iter_mut().rev().enumerate() {
            if vtext.contains(ctx.context, point) {
                // クリックしていたメニューのエントリに対応するイベントを取り出し、返す
                // イベントのハンドリングは上位に任せる
                let event = self.menu_entries_data.get(index).unwrap();
                return Some(event.content_event.clone());
            }
        }

        None
    }
}

impl DrawableComponent for VTextList {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            for vtext in self.vtext_list.iter_mut() {
                vtext.draw(ctx)?;
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

pub struct TitleSoundPlayerData {
    font_info: FontInformation,
    position: numeric::Point2f,
    text: String,
}

impl TitleSoundPlayerData {
    pub fn from_toml<'a>(ctx: &mut SuzuContext<'a>, path: &str) -> Self {
	let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => panic!("Failed to read: {}", path),
        };
	
        let root = content.parse::<toml::Value>().unwrap();

	let text = root["text"].as_str().unwrap();

	let position_table = root["position"].as_table().unwrap();
	
        let position = numeric::Point2f::new(
            position_table["x"].as_float().unwrap() as f32,
            position_table["y"].as_float().unwrap() as f32,
        );

        let font_info = font_information_from_toml_value(
	    ctx.resource,
	    &root["font_info"]
	);

	TitleSoundPlayerData {
	    font_info: font_info,
	    position: position,
	    text: text.to_string(),
	}
    }
}

pub struct TitleSoundPlayer {
    main_text: VerticalText,
    name: String,
    drwob_essential: DrawableObjectEssential,
}

impl TitleSoundPlayer {
    pub fn new<'a>(
	name: String,
	data: TitleSoundPlayerData,
    ) -> Self {
	let main_text = VerticalText::new(
	    data.text.to_string(),
	    data.position,
	    numeric::Vector2f::new(1.0, 1.0),
	    0.0,
	    0,
	    data.font_info
	);

	TitleSoundPlayer {
	    main_text: main_text,
	    name: name,
	    drwob_essential: DrawableObjectEssential::new(true, 0),
	}
    }

    pub fn get_name(&self) -> String {
	self.name.to_string()
    }
}

impl DrawableComponent for TitleSoundPlayer {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
	    self.main_text.draw(ctx)?;
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

pub enum TitleContents {
    InitialMenu(VTextList),
    TitleSoundPlayer(TitleSoundPlayer),
}

impl TitleContents {
    pub fn from_toml_object<'a>(
        ctx: &mut SuzuContext<'a>,
        toml_src: &toml::Value,
    ) -> Option<TitleContents> {
        let name = toml_src
            .get("name")
            .expect("name field is missing")
            .as_str()
            .unwrap();

        let contents_type = toml_src
            .get("type")
            .expect("type field is missing")
            .as_str()
            .unwrap();

        let details_source_file = toml_src
            .get("src")
            .expect("src field is missing")
            .as_str()
            .unwrap();

        match contents_type {
            "VTextList" => {
                let menu_data = TextMenuData::from_file(ctx, name.to_string(), details_source_file);
                Some(
                    TitleContents::InitialMenu(VTextList::new(menu_data, 0)),
                )
            },
	    "TitleSoundPlayer" => {
		let data = TitleSoundPlayerData::from_toml(ctx, details_source_file);
		let sound_player = TitleSoundPlayer::new(name.to_string(), data);
		Some(
		    TitleContents::TitleSoundPlayer(sound_player),
		)
	    },
            _ => None,
        }
    }

    pub fn get_content_name(&self) -> String {
        match self {
            TitleContents::InitialMenu(menu) => menu.contents_name.to_string(),
	    TitleContents::TitleSoundPlayer(player) => player.get_name(),
        }
    }
}

impl DrawableComponent for TitleContents {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        match self {
            TitleContents::InitialMenu(contents) => contents.draw(ctx),
	    TitleContents::TitleSoundPlayer(contents) => contents.draw(ctx),
        }
    }

    fn hide(&mut self) {
        match self {
            TitleContents::InitialMenu(contents) => contents.hide(),
	    TitleContents::TitleSoundPlayer(contents) => contents.hide(),
        }
    }

    fn appear(&mut self) {
        match self {
            TitleContents::InitialMenu(contents) => contents.appear(),
	    TitleContents::TitleSoundPlayer(contents) => contents.appear(),
        }
    }

    fn is_visible(&self) -> bool {
        match self {
            TitleContents::InitialMenu(contents) => contents.is_visible(),
	    TitleContents::TitleSoundPlayer(contents) => contents.is_visible(),
        }
    }

    fn set_drawing_depth(&mut self, depth: i8) {
        match self {
            TitleContents::InitialMenu(contents) => contents.set_drawing_depth(depth),
	    TitleContents::TitleSoundPlayer(contents) => contents.set_drawing_depth(depth),
        }
    }

    fn get_drawing_depth(&self) -> i8 {
        match self {
            TitleContents::InitialMenu(contents) => contents.get_drawing_depth(),
	    TitleContents::TitleSoundPlayer(contents) => contents.get_drawing_depth(),
        }
    }
}

pub struct TitleContentsSet {
    contents_set: HashMap<String, TitleContents>,
}

impl TitleContentsSet {
    pub fn new() -> Self {
        TitleContentsSet {
            contents_set: HashMap::new(),
        }
    }

    pub fn from_file<'a>(ctx: &mut SuzuContext<'a>, file_path: &str) -> Self {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => panic!("Failed to read: {}", file_path),
        };

        let root = content.parse::<toml::Value>().unwrap();
        let contents_list = root["contents-list"].as_array().unwrap();

        let mut contents_set = HashMap::new();

        for content in contents_list {
            let title_content = TitleContents::from_toml_object(ctx, content).unwrap();
            contents_set.insert(title_content.get_content_name(), title_content);
        }

        TitleContentsSet {
            contents_set: contents_set,
        }
    }

    pub fn add(&mut self, key: String, contents: TitleContents) -> &mut TitleContentsSet {
        self.contents_set.insert(key, contents);
        self
    }

    pub fn remove_pickup(&mut self, key: &str) -> Option<TitleContents> {
        self.contents_set.remove(key)
    }
}
