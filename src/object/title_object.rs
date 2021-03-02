use std::collections::HashMap;

use std::str::FromStr;

use ggez::graphics as ggraphics;
use ggez::input::mouse::MouseButton;

use numeric::Vector2f;
use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;

use torifune::core::Clock;
use torifune::graphics::drawable::*;
use torifune::graphics::object::*;
use torifune::numeric;

use crate::{core::{GameMode, WINDOW_SIZE_X, WINDOW_SIZE_Y}, flush_delay_event, flush_delay_event_and_redraw_check, object::util_object::{CheckBox, SeekBar, SelectButton, TextButtonTexture}, scene::DelayEventList};
use crate::scene::SceneID;
use crate::{
    core::{font_information_from_toml_value, FontID, SuzuContext, TextureID},
    scene::SceneTransition,
};

use super::{DarkEffectPanel, scenario::ScenarioEvent};

extern crate reqwest;

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
    SceneTransition((SceneID, SceneTransition, Option<GameMode>)),
    BuiltinEvent(TitleBuiltinCommand),
}

impl TitleContentsEvent {
    pub fn from_toml_value(toml_value: &toml::Value) -> Option<Self> {
        let s = toml_value["event-type"].as_str().unwrap();

        match s {
            "SceneTransition" => {
                let next_scene_str = toml_value["next-scene"].as_str().expect("error");
                let (next_scene, game_mode) = 
		    match SceneID::from_str(next_scene_str) {
			Ok(id) => (id, None),
			Err(_) => match next_scene_str {
			    "ScenarioStory" => (SceneID::Scenario, Some(GameMode::story())),
			    "ScenarioTA300000" => (SceneID::Scenario, Some(GameMode::time_attack(300000))),
			    "ScenarioTA500000" => (SceneID::Scenario, Some(GameMode::time_attack(500000))),
			    "ScenarioTA1000000" => (SceneID::Scenario, Some(GameMode::time_attack(1000000))),
			    _ => panic!("invalid next-scene-str field"),
			}
		    };
		
                let next_trans_str = toml_value["transition-method"].as_str().expect("error");
                let next_trans =
                    SceneTransition::from_str(next_trans_str).expect("Unknown next scene");
                Some(TitleContentsEvent::SceneTransition((
                    next_scene, next_trans, game_mode,
                )))
            }
            "NextContents" => {
                let next_scene_str = toml_value["next-contents-name"]
                    .as_str()
                    .expect("error")
                    .to_string();
                Some(TitleContentsEvent::NextContents(next_scene_str))
            }
            "BuiltinCommand" => {
                let command = toml_value["builtin-command"].as_str().expect("error");
                Some(TitleContentsEvent::BuiltinEvent(
                    TitleBuiltinCommand::from_str(command).unwrap(),
                ))
            }
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
                Vector2f::new(1.0, 1.0),
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

        let font_info = font_information_from_toml_value(ctx.resource, &root["font_info"]);

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
    pub fn new<'a>(_ctx: &mut SuzuContext<'a>, name: String, data: TitleSoundPlayerData) -> Self {
        let main_text = VerticalText::new(
            data.text.to_string(),
            data.position,
            Vector2f::new(1.0, 1.0),
            0.0,
            0,
            data.font_info,
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

    pub fn dragging_handler<'a>(
        &mut self,
        _ctx: &mut SuzuContext<'a>,
        _point: numeric::Point2f,
        _offset: Vector2f,
    ) {
    }

    pub fn mouse_button_down_handler<'a>(
        &mut self,
        _ctx: &mut SuzuContext<'a>,
        _point: numeric::Point2f,
    ) {
    }

    pub fn mouse_button_up_handler(&mut self) {}
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

impl DrawableObject for TitleSoundPlayer {
    impl_drawable_object_for_wrapped! {main_text}
}

impl TextureObject for TitleSoundPlayer {
    impl_texture_object_for_wrapped! {main_text}
}

type DynamicTitleSoundPlayer = MovableWrap<TitleSoundPlayer>;

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum GameConfigElement {
    BGMVolume,
    SEVolume,
}

struct TemporaryConfigData {
    bgm_volume: f32,
    se_volume: f32,
    pause_when_inactive: bool,
}

impl TemporaryConfigData {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>) -> Self {
        TemporaryConfigData {
            bgm_volume: ctx.config.get_bgm_volume(),
            se_volume: ctx.config.get_se_volume(),
	    pause_when_inactive: ctx.config.is_pause_when_inactive(),
        }
    }
}

pub struct ConfigPanel {
    canvas: sub_screen::SubScreen,
    background: DarkEffectPanel,
    hrzn_text_list: Vec<UniText>,
    sb_dynamic_text: HashMap<GameConfigElement, UniText>,
    header_text: UniText,
    bgm_volume_bar: SeekBar,
    se_volume_bar: SeekBar,
    checkbox: CheckBox,
    apply_button: SelectButton,
    cancel_button: SelectButton,
    original_config_data: TemporaryConfigData,
}

impl ConfigPanel {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, pos_rect: numeric::Rect, depth: i8, t: Clock) -> Self {
        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(32.0, 32.0),
            ggraphics::Color::from_rgba_u32(0xbbbbbbff),
        );

        let mut background = DarkEffectPanel::new(
            ctx.context,
            numeric::Rect::new(0.0, 0.0, WINDOW_SIZE_X as f32, WINDOW_SIZE_X as f32),
            t,
        );
	background.set_alpha(0.5);

        let header_text = UniText::new(
            "設定".to_string(),
            numeric::Point2f::new(650.0, 80.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            font_info,
        );

        let mut hrzn_text_list = Vec::new();

        let hrzn_text_font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(29.0, 29.0),
            ggraphics::Color::from_rgba_u32(0xbbbbbbff),
        );

        for (s, p) in vec![("BGM音量", 180.0), ("SE音量", 280.0)] {
            let text = UniText::new(
                s.to_string(),
                numeric::Point2f::new(200.0, p),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                hrzn_text_font_info.clone(),
            );

            hrzn_text_list.push(text);
        }

        let mut sb_dynamic_text = HashMap::new();

        sb_dynamic_text.insert(
            GameConfigElement::BGMVolume,
            UniText::new(
                format!("{}%", (ctx.config.get_bgm_volume() * 100.0).round()),
                numeric::Point2f::new(400.0, 180.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                hrzn_text_font_info.clone(),
            ),
        );

        sb_dynamic_text.insert(
            GameConfigElement::SEVolume,
            UniText::new(
                format!("{}%", (ctx.config.get_se_volume() * 100.0).round()),
                numeric::Point2f::new(400.0, 280.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                hrzn_text_font_info.clone(),
            ),
        );

        let text_texture = Box::new(TextButtonTexture::new(
            ctx,
            numeric::Point2f::new(0.0, 0.0),
            "適用".to_string(),
            hrzn_text_font_info.clone(),
            8.0,
            ggraphics::Color::from_rgba_u32(0x362d33ff),
            0,
        ));

        let apply_button = SelectButton::new(
            ctx,
            numeric::Rect::new(650.0, 600.0, 100.0, 50.0),
            text_texture,
        );

        let text_texture = Box::new(TextButtonTexture::new(
            ctx,
            numeric::Point2f::new(0.0, 0.0),
            "中止".to_string(),
            hrzn_text_font_info.clone(),
            8.0,
            ggraphics::Color::from_rgba_u32(0x362d33ff),
            0,
        ));

        let cancel_button = SelectButton::new(
            ctx,
            numeric::Rect::new(850.0, 600.0, 100.0, 50.0),
            text_texture,
        );

	let pause_text = UniText::new(
            "店番中の非アクティブ時にポーズ".to_string(),
            numeric::Point2f::new(200.0, 400.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            hrzn_text_font_info.clone(),
        );
	hrzn_text_list.push(pause_text);
        let choice_box_texture = Box::new(UniTexture::new(
            ctx.ref_texture(TextureID::CheckCircle),
            numeric::Point2f::new(200.0, 440.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
        ));
        let check_box = CheckBox::new(
            ctx,
            numeric::Rect::new(200.0, 440.0, 50.0, 50.0),
            choice_box_texture,
            ctx.config.is_pause_when_inactive(),
            0,
        );

        ConfigPanel {
            header_text: header_text,
            sb_dynamic_text: sb_dynamic_text,
            canvas: sub_screen::SubScreen::new(
                ctx.context,
                pos_rect,
                depth,
                ggraphics::Color::from_rgba_u32(0),
            ),
            background: background,
            hrzn_text_list: hrzn_text_list,
            bgm_volume_bar: SeekBar::new(
                ctx,
                numeric::Rect::new(200.0, 210.0, 450.0, 40.0),
                10.0,
                100.0,
                0.0,
                ctx.config.get_bgm_volume() * 100.0,
                0,
            ),
            se_volume_bar: SeekBar::new(
                ctx,
                numeric::Rect::new(200.0, 310.0, 450.0, 40.0),
                10.0,
                100.0,
                0.0,
                ctx.config.get_se_volume() * 100.0,
                0,
            ),
            apply_button: apply_button,
            cancel_button: cancel_button,
            original_config_data: TemporaryConfigData::new(ctx),
            checkbox: check_box,
        }
    }

    fn update_seek_bar_value(&mut self) {
        let bgm_volume = self.bgm_volume_bar.get_current_value() as i32;
        let se_volume = self.se_volume_bar.get_current_value() as i32;

        self.sb_dynamic_text
            .get_mut(&GameConfigElement::BGMVolume)
            .unwrap()
            .replace_text(format!("{}%", bgm_volume));
        self.sb_dynamic_text
            .get_mut(&GameConfigElement::SEVolume)
            .unwrap()
            .replace_text(format!("{}%", se_volume));
    }

    fn recover_original_config<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        let original_bgm = self.original_config_data.bgm_volume * 100.0;
        let original_se = self.original_config_data.se_volume * 100.0;
	let original_pause = self.original_config_data.pause_when_inactive;

        ctx.change_bgm_volume(original_bgm);
        ctx.change_se_volume(original_se);
	ctx.config.set_pause_when_inactive(original_pause);

        self.bgm_volume_bar.set_value(ctx, original_bgm);
        self.se_volume_bar.set_value(ctx, original_se);
	self.checkbox.try_check(original_pause);
    }

    pub fn get_name(&self) -> String {
        "config-panel".to_string()
    }

    pub fn mouse_button_down<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: MouseButton,
        point: numeric::Point2f,
        _t: Clock,
    ) {
        match button {
            MouseButton::Left => {
                let rpoint = self.canvas.relative_point(point);

                self.bgm_volume_bar.start_dragging_check(ctx, rpoint);
                self.se_volume_bar.start_dragging_check(ctx, rpoint);
            }
            _ => (),
        }
    }

    pub fn mouse_button_up<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        _t: Clock,
    ) -> Option<TitleContentsEvent> {
        self.bgm_volume_bar.release_handler();
        self.se_volume_bar.release_handler();
	
        let rpoint = self.canvas.relative_point(point);
	self.checkbox.click_handler(rpoint);

        if self.apply_button.contains(ctx.context, rpoint) {
	    crate::core::crypt::encrypt_game_binary();
            ctx.change_bgm_volume(self.bgm_volume_bar.get_current_value());
            ctx.change_se_volume(self.se_volume_bar.get_current_value());
	    ctx.config.set_pause_when_inactive(self.checkbox.checked_now());
	    ctx.config.save_config();
            return Some(TitleContentsEvent::NextContents("init-menu".to_string()));
        }

        if self.cancel_button.contains(ctx.context, rpoint) {
            self.recover_original_config(ctx);
            return Some(TitleContentsEvent::NextContents("init-menu".to_string()));
        }

        None
    }

    pub fn mouse_dragging_handler<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        _button: MouseButton,
        point: numeric::Point2f,
        _t: Clock,
    ) {
        let rpoint = self.canvas.relative_point(point);

        self.bgm_volume_bar.dragging_handler(ctx, rpoint);
        self.se_volume_bar.dragging_handler(ctx, rpoint);

        self.update_seek_bar_value();

        ctx.change_bgm_volume(self.bgm_volume_bar.get_current_value());
        ctx.change_se_volume(self.se_volume_bar.get_current_value());
    }
}

impl DrawableComponent for ConfigPanel {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.background.draw(ctx)?;

            self.header_text.draw(ctx)?;
            self.bgm_volume_bar.draw(ctx)?;
            self.se_volume_bar.draw(ctx)?;

            for text in self.hrzn_text_list.iter_mut() {
                text.draw(ctx)?;
            }

            for (_, text) in self.sb_dynamic_text.iter_mut() {
                text.draw(ctx)?;
            }

            self.apply_button.draw(ctx)?;
            self.cancel_button.draw(ctx)?;

            self.checkbox.draw(ctx)?;

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


pub struct UpdatePanel {
    canvas: sub_screen::SubScreen,
    background: DarkEffectPanel,
    header_text: UniText,
    main_message: UniText,
    apply_button: SelectButton,
    cancel_button: SelectButton,
    event_list: DelayEventList<Self>,
    latest_version: Option<f64>,
}

impl UpdatePanel {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, pos_rect: numeric::Rect, depth: i8, t: Clock) -> Self {
        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(32.0, 32.0),
            ggraphics::Color::from_rgba_u32(0xbbbbbbff),
        );

        let mut background = DarkEffectPanel::new(
            ctx.context,
            numeric::Rect::new(0.0, 0.0, WINDOW_SIZE_X as f32, WINDOW_SIZE_X as f32),
            t,
        );
	background.set_alpha(0.5);

	let hrzn_text_font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(29.0, 29.0),
            ggraphics::Color::from_rgba_u32(0xbbbbbbff),
        );

        let header_text = UniText::new(
            "更新".to_string(),
            numeric::Point2f::new(650.0, 80.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            font_info,
        );

	let main_message = UniText::new(
	    "".to_string(),
            numeric::Point2f::new(650.0, 200.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            font_info,
	);

        let text_texture = Box::new(TextButtonTexture::new(
            ctx,
            numeric::Point2f::new(0.0, 0.0),
            "更新".to_string(),
            hrzn_text_font_info.clone(),
            8.0,
            ggraphics::Color::from_rgba_u32(0x362d33ff),
            0,
        ));

        let apply_button = SelectButton::new(
            ctx,
            numeric::Rect::new(650.0, 600.0, 100.0, 50.0),
            text_texture,
        );

        let text_texture = Box::new(TextButtonTexture::new(
            ctx,
            numeric::Point2f::new(0.0, 0.0),
            "逆戻".to_string(),
            hrzn_text_font_info.clone(),
            8.0,
            ggraphics::Color::from_rgba_u32(0x362d33ff),
            0,
        ));

        let cancel_button = SelectButton::new(
            ctx,
            numeric::Rect::new(850.0, 600.0, 100.0, 50.0),
            text_texture,
        );
	
        UpdatePanel {
            header_text: header_text,
            canvas: sub_screen::SubScreen::new(
                ctx.context,
                pos_rect,
                depth,
                ggraphics::Color::from_rgba_u32(0),
            ),
	    main_message: main_message,
            background: background,
            apply_button: apply_button,
            cancel_button: cancel_button,
	    event_list: DelayEventList::new(),
	    latest_version: None,
        }
    }
    
    pub fn get_name(&self) -> String {
        "update-panel".to_string()
    }

    pub fn mouse_button_down<'a>(
        &mut self,
        _ctx: &mut SuzuContext<'a>,
        button: MouseButton,
	_point: numeric::Point2f,
        _t: Clock,
    ) {
        match button {
            MouseButton::Left => {
		()
            }
            _ => (),
        }
    }

    pub fn mouse_button_up<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        t: Clock,
    ) -> Option<TitleContentsEvent> {
        let rpoint = self.canvas.relative_point(point);

        if self.apply_button.contains(ctx.context, rpoint) {
	    if let Some(version) = self.latest_version {
		self.replace_main_message(ctx, "更新中・・・");
		
		self.event_list.add_event(
		    Box::new(move |slf: &mut UpdatePanel, ctx, _t| {
			match slf.exec_update(version) {
			    Ok(_) => slf.replace_main_message(ctx, &format!("更新完了 ver.{}", version)),
			    Err(_) => slf.replace_main_message(ctx, "更新失敗 時間を置いて再度お試しください"),
			}
			slf.latest_version = None;
			
		    }
		), t + 1);
	    }
        }

        if self.cancel_button.contains(ctx.context, rpoint) {
            return Some(TitleContentsEvent::NextContents("init-menu".to_string()));
        }

        None
    }

    pub fn check_update<'a>(&mut self, _ctx: &mut SuzuContext<'a>, _t: Clock) -> Result<f64, ()> {
	let resp = match reqwest::blocking::get("https://boxed-sumire.fun/suzu/update.toml") {
	    Ok(resp) => match resp.text() {
		Ok(text) => text,
		Err(_) => return Err(()),
	    },
	    Err(_) => return Err(()),
	};
	
	let root = match resp.parse::<toml::Value>() {
	    Ok(root) => root,
	    Err(_) => return Err(()),
	};
	
	if let Some(latest_version) = root["latest"].as_float() {
	    if latest_version > crate::core::VERSION {
		Ok(latest_version)
	    } else {
		Err(())
	    }
	} else {
	    Err(())
	}
    }

    pub fn exec_update(&self, version: f64) -> Result<(), ()> {
	if version < crate::core::VERSION {
	    return Err(());
	}

	let bytes = match reqwest::blocking::get(&format!("https://boxed-sumire.fun/suzu/suzu-{}.encrypted", version)) {
	    Ok(resp) => match resp.bytes() {
		Ok(bytes) => bytes,
		Err(_) => return Err(()),
	    },
	    Err(_) => return Err(()),
	};

	crate::core::crypt::decrypt_game_binary(&bytes.to_vec());
	Ok(())
    }
    
    pub fn flush_delayed_event<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
	flush_delay_event_and_redraw_check!(self, self.event_list, ctx, t, { });
    }

    pub fn notify_switched<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
	self.replace_main_message(ctx, "更新確認中・・・");

	self.event_list.add_event(
            Box::new(move |slf: &mut UpdatePanel, ctx, t| {
		match slf.check_update(ctx, t) {
		    Ok(version) => {
			slf.latest_version = Some(version);
			slf.replace_main_message(
			    ctx,
			    &format!("ver.{} → ver.{}の更新が見つかりました", crate::core::VERSION, version)
			);
		    },
		    Err(()) => {
			slf.replace_main_message(ctx, &format!("最新版です ver.{}", crate::core::VERSION));
		    },
		}
            }),
	    t + 2,
	);
    }

    pub fn replace_main_message<'a>(&mut self, ctx: &mut SuzuContext<'a>, s: &str) {
	self.main_message.replace_text(s.to_string());
	self.main_message.make_center(
	    ctx.context,
	    numeric::Point2f::new(WINDOW_SIZE_X as f32 / 2.0, WINDOW_SIZE_Y as f32 / 2.0)
	);
    }
}

impl DrawableComponent for UpdatePanel {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.background.draw(ctx)?;

            self.header_text.draw(ctx)?;
	    self.main_message.draw(ctx)?;

            self.apply_button.draw(ctx)?;
            self.cancel_button.draw(ctx)?;

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


pub struct Gallery {
    canvas: sub_screen::SubScreen,
    background: DarkEffectPanel,
    header_text: UniText,
    event_list: DelayEventList<Self>,
    gallery_list: Vec<UniTexture>,
    scenario_event: ScenarioEvent,
    gallery_index: i64,
}

impl Gallery {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, pos_rect: numeric::Rect, depth: i8, t: Clock) -> Self {
        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(32.0, 32.0),
            ggraphics::Color::from_rgba_u32(0xbbbbbbff),
        );

        let mut background = DarkEffectPanel::new(
            ctx.context,
            numeric::Rect::new(0.0, 0.0, WINDOW_SIZE_X as f32, WINDOW_SIZE_X as f32),
            t,
        );
	background.set_alpha(0.5);

        let header_text = UniText::new(
            "供養".to_string(),
            numeric::Point2f::new(650.0, 80.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            font_info,
        );

	let mut gallery_list = Vec::new();

	{
	    let mut texture = UniTexture::new(
		ctx.ref_texture(TextureID::KosuzuTachie1),
		numeric::Point2f::new(0.0, 0.0),
		numeric::Vector2f::new(0.227, 0.227),
		0.0,
		0
	    );
	    texture.make_center(ctx.context, numeric::Point2f::new(WINDOW_SIZE_X as f32 / 2.0, 300.0));
	    gallery_list.push(texture);
	}

	{
	    let mut texture = UniTexture::new(
		ctx.ref_texture(TextureID::AkyuTachieDefault),
		numeric::Point2f::new(0.0, 0.0),
		numeric::Vector2f::new(0.22, 0.22),
		0.0,
		0
	    );
	    texture.make_center(ctx.context, numeric::Point2f::new(WINDOW_SIZE_X as f32 / 2.0, 320.0));
	    gallery_list.push(texture);
	}

	{
	    let mut texture = UniTexture::new(
		ctx.ref_texture(TextureID::Mob1TachieDefault),
		numeric::Point2f::new(0.0, 0.0),
		numeric::Vector2f::new(0.235, 0.235),
		0.0,
		0
	    );
	    texture.make_center(ctx.context, numeric::Point2f::new(WINDOW_SIZE_X as f32 / 2.0, 300.0));
	    gallery_list.push(texture);
	}

	let mut g = Gallery {
            header_text: header_text,
            canvas: sub_screen::SubScreen::new(
                ctx.context,
                pos_rect,
                depth,
                ggraphics::Color::from_rgba_u32(0),
            ),
            background: background,
	    event_list: DelayEventList::new(),
	    gallery_list: gallery_list,
	    scenario_event: ScenarioEvent::new(
		ctx,
		numeric::Rect::new(0.0, 0.0, 1366.0, 748.0),
		"/scenario/empty.toml",
		true,
		t
	    ),
	    gallery_index: 0,
        };

	g.update_gallery_text(ctx);
	g
    }
    
    pub fn get_name(&self) -> String {
        "gallery".to_string()
    }

    pub fn current_gallery_image(&mut self) -> Option<&mut dyn DrawableComponent> {
	let index = self.gallery_index.abs() % 3;

	match index {
	    0 | 1 | 2 => {
		Some(self.gallery_list.get_mut(index as usize).unwrap() as &mut dyn DrawableComponent)
	    }
	    _=> None,
	}
    }

    pub fn update_gallery_text<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
	let index = self.gallery_index.abs() % 3;

	match index {
	    0 => {
		self.scenario_event.set_fixed_text_to_scenario_box(ctx, "本居小鈴の立ち絵\n一番最初に書きました。ちゃんと練習もしました。".to_string());
	    }
	    1 => {
		self.scenario_event.set_fixed_text_to_scenario_box(ctx, "稗田阿求の立ち絵\n二番目に書きました。なんか上手く描けた気がする。".to_string());
	    }
	    2 => {
		self.scenario_event.set_fixed_text_to_scenario_box(ctx, "モブの立ち絵1号\nロリっぽく描こうとしたら結構可愛くなりました。".to_string());
	    }
	    _=> (),
	}
    }

    pub fn mouse_button_up<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        _point: numeric::Point2f,
        _t: Clock,
    ) -> Option<TitleContentsEvent> {
	if false {
            return Some(TitleContentsEvent::NextContents("init-menu".to_string()));
        }

	self.gallery_index += 1;

	self.update_gallery_text(ctx);

        None
    }
    
    pub fn flush_delayed_event<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
	flush_delay_event_and_redraw_check!(self, self.event_list, ctx, t, { });
    }
}

impl DrawableComponent for Gallery {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.background.draw(ctx)?;

            self.header_text.draw(ctx)?;

	    if let Some(obj) = self.current_gallery_image() {
		obj.draw(ctx)?;
	    }

	    self.scenario_event.draw(ctx)?;

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


pub enum TitleContents {
    InitialMenu(VTextList),
    TitleSoundPlayer(DynamicTitleSoundPlayer),
    ConfigPanel(ConfigPanel),
    UpdatePanel(UpdatePanel),
    Gallery(Gallery),
}

impl TitleContents {
    pub fn from_toml_object<'a>(
        ctx: &mut SuzuContext<'a>,
        toml_src: &toml::Value,
	t: Clock,
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
                Some(TitleContents::InitialMenu(VTextList::new(menu_data, 0)))
            }
            "TitleSoundPlayer" => {
                let data = TitleSoundPlayerData::from_toml(ctx, details_source_file);
                let sound_player = MovableWrap::new(
                    Box::new(TitleSoundPlayer::new(ctx, name.to_string(), data)),
                    None,
                    0,
                );
                Some(TitleContents::TitleSoundPlayer(sound_player))
            }
            "ConfigPanel" => Some(TitleContents::ConfigPanel(ConfigPanel::new(
                ctx,
                numeric::Rect::new(0.0, 0.0, 1366.0, 768.0),
                0,
		t
            ))),
	    "UpdatePanel" => Some(TitleContents::UpdatePanel(UpdatePanel::new(
                ctx,
                numeric::Rect::new(0.0, 0.0, 1366.0, 768.0),
                0,
		t
            ))),
	    "Gallery" => Some(TitleContents::Gallery(Gallery::new(
                ctx,
                numeric::Rect::new(0.0, 0.0, 1366.0, 768.0),
                0,
		t
            ))),
            _ => None,
        }
    }

    pub fn get_content_name(&self) -> String {
        match self {
            TitleContents::InitialMenu(menu) => menu.contents_name.to_string(),
            TitleContents::TitleSoundPlayer(player) => player.get_name(),
            TitleContents::ConfigPanel(panel) => panel.get_name(),
	    TitleContents::UpdatePanel(panel) => panel.get_name(),
	    TitleContents::Gallery(gallery) => gallery.get_name(),
        }
    }

    pub fn update<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        match self {
            TitleContents::InitialMenu(_) => (),
            TitleContents::TitleSoundPlayer(_) => (),
            TitleContents::ConfigPanel(_) => (),
	    TitleContents::UpdatePanel(panel) => panel.flush_delayed_event(ctx, t),
	    TitleContents::Gallery(gallery) => gallery.flush_delayed_event(ctx, t),
        }
    }

    pub fn notify_switched<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        match self {
            TitleContents::InitialMenu(_) => (),
            TitleContents::TitleSoundPlayer(_) => (),
            TitleContents::ConfigPanel(_) => (),
	    TitleContents::UpdatePanel(panel) => panel.notify_switched(ctx, t),
	    TitleContents::Gallery(_) => (),
        }	
    }
}

impl DrawableComponent for TitleContents {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        match self {
            TitleContents::InitialMenu(contents) => contents.draw(ctx),
            TitleContents::TitleSoundPlayer(contents) => contents.draw(ctx),
            TitleContents::ConfigPanel(panel) => panel.draw(ctx),
	    TitleContents::UpdatePanel(panel) => panel.draw(ctx),
	    TitleContents::Gallery(gallery) => gallery.draw(ctx),
        }
    }

    fn hide(&mut self) {
        match self {
            TitleContents::InitialMenu(contents) => contents.hide(),
            TitleContents::TitleSoundPlayer(contents) => contents.hide(),
            TitleContents::ConfigPanel(panel) => panel.hide(),
	    TitleContents::UpdatePanel(panel) => panel.hide(),
	    TitleContents::Gallery(gallery) => gallery.hide(),
        }
    }

    fn appear(&mut self) {
        match self {
            TitleContents::InitialMenu(contents) => contents.appear(),
            TitleContents::TitleSoundPlayer(contents) => contents.appear(),
            TitleContents::ConfigPanel(panel) => panel.appear(),
	    TitleContents::UpdatePanel(panel) => panel.appear(),
	    TitleContents::Gallery(gallery) => gallery.appear(),
        }
    }

    fn is_visible(&self) -> bool {
        match self {
            TitleContents::InitialMenu(contents) => contents.is_visible(),
            TitleContents::TitleSoundPlayer(contents) => contents.is_visible(),
            TitleContents::ConfigPanel(panel) => panel.is_visible(),
	    TitleContents::UpdatePanel(panel) => panel.is_visible(),
	    TitleContents::Gallery(gallery) => gallery.is_visible(),
        }
    }

    fn set_drawing_depth(&mut self, depth: i8) {
        match self {
            TitleContents::InitialMenu(contents) => contents.set_drawing_depth(depth),
            TitleContents::TitleSoundPlayer(contents) => contents.set_drawing_depth(depth),
            TitleContents::ConfigPanel(panel) => panel.set_drawing_depth(depth),
	    TitleContents::UpdatePanel(panel) => panel.set_drawing_depth(depth),
	    TitleContents::Gallery(gallery) => gallery.set_drawing_depth(depth),
        }
    }

    fn get_drawing_depth(&self) -> i8 {
        match self {
            TitleContents::InitialMenu(contents) => contents.get_drawing_depth(),
            TitleContents::TitleSoundPlayer(contents) => contents.get_drawing_depth(),
            TitleContents::ConfigPanel(panel) => panel.get_drawing_depth(),
	    TitleContents::UpdatePanel(panel) => panel.get_drawing_depth(),
	    TitleContents::Gallery(gallery) => gallery.get_drawing_depth(),
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

    pub fn from_file<'a>(ctx: &mut SuzuContext<'a>, file_path: &str, t: Clock) -> Self {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => panic!("Failed to read: {}", file_path),
        };
	
        let root = content.parse::<toml::Value>().unwrap();
        let contents_list = root["contents-list"].as_array().unwrap();

        let mut contents_set = HashMap::new();

        for content in contents_list {
            let title_content = TitleContents::from_toml_object(ctx, content, t).unwrap();
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
