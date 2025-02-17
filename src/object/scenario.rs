use std::collections::LinkedList;
use std::collections::VecDeque;

use torifune::graphics::object::sub_screen;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::graphics::object::*;
use torifune::numeric;
use torifune::{graphics::drawable::*, sound::SoundHandler};

use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;

use super::*;
use crate::{core::ScenarioSceneSaveData, parse_toml_file};
use crate::scene::scenario_scene::ScenarioContext;
use crate::scene::{SceneID, SceneTransition};
use crate::{core::SoundID, object::util_object::*};
use crate::{
    core::{FontID, GameResource, SuzuContext, TextureID, TileBatchTextureID},
    scene::DrawRequest,
};
use std::str::FromStr;

pub type ScenarioElementID = i32;

pub struct ScenarioTextAttribute {
    pub fpc: f32,
    pub font_info: FontInformation,
}

pub struct ScenarioTextSegment {
    text: String,
    attribute: ScenarioTextAttribute,
}

impl ScenarioTextSegment {
    pub fn new(text: &str, fpc: f32, font_info: FontInformation) -> Self {
        ScenarioTextSegment {
            text: text.to_string(),
            attribute: ScenarioTextAttribute {
                fpc: fpc,
                font_info: font_info,
            },
        }
    }

    pub fn from_toml_using_default(
        obj: &toml::value::Table,
        game_data: &GameResource,
        default: &ScenarioTextAttribute,
    ) -> Self {
        let text = if let Some(text_path) = obj.get("text_src_path").as_ref() {
            std::fs::read_to_string(text_path.as_str().unwrap()).unwrap()
        } else {
            obj.get("text")
                .expect("You must insert text or text_src_path field.")
                .as_str()
                .unwrap()
                .to_string()
        };

        let fpc = if let Some(fpc) = obj.get("fpc") {
            fpc.as_float().unwrap() as f32
        } else {
            default.fpc
        };

        let font_scale = if let Some(font_scale) = obj.get("font_scale") {
            font_scale.as_float().unwrap() as f32
        } else {
            default.font_info.scale.x
        };

        let color = if let Some(color) = obj.get("color") {
            ggraphics::Color::from_rgba_u32(color.as_integer().unwrap() as u32)
        } else {
            default.font_info.color
        };

        ScenarioTextSegment {
            text: text,
            attribute: ScenarioTextAttribute {
                fpc: fpc,
                font_info: FontInformation::new(
                    game_data.get_font(FontID::Cinema),
                    numeric::Vector2f::new(font_scale, font_scale),
                    color,
                ),
            },
        }
    }

    fn slice_text_bytes(&self, begin: usize, end: usize) -> &str {
        unsafe { self.text.as_str().get_unchecked(begin..end) }
    }

    fn last_line_length(&self) -> usize {
        let mut length: usize = 0;
        let it = self.text.chars().rev();

        for ch in it {
            if ch == '\n' {
                break;
            }
            length += 1
        }

        length
    }

    pub fn slice(&self, length: usize) -> &str {
        let mut reached_flag = false;

        let str_bytes = self.text.as_str().as_bytes().len();

        for (count, (bytes, _)) in self.text.as_str().char_indices().enumerate() {
            if reached_flag {
                return self.slice_text_bytes(0, bytes as usize);
            }

            if count == length {
                reached_flag = true;
            }
        }

        return self.slice_text_bytes(0, str_bytes as usize);
    }

    pub fn get_fpc(&self) -> f32 {
        self.attribute.fpc
    }

    pub fn str_len(&self) -> usize {
        self.text.chars().count()
    }

    pub fn get_font_info(&self) -> &FontInformation {
        &self.attribute.font_info
    }

    pub fn get_font_scale(&self) -> numeric::Vector2f {
        self.attribute.font_info.scale
    }

    pub fn get_attribute(&self) -> &ScenarioTextAttribute {
        &self.attribute
    }
}

pub struct ScenarioTachie {
    left: Option<SimpleObject>,
    right: Option<SimpleObject>,
    inner_right: Option<SimpleObject>,
    drwob_essential: DrawableObjectEssential,
}

impl ScenarioTachie {
    fn tachie_texture_scale(id: TextureID) -> numeric::Vector2f {
        match id {
            TextureID::KosuzuTachie1 => numeric::Vector2f::new(0.3, 0.3),
            TextureID::AkyuTachieDefault => numeric::Vector2f::new(0.248, 0.248),
	    TextureID::NitoriTachieDefault => numeric::Vector2f::new(0.248, 0.248),
	    TextureID::NitoriTachieSunGlass => numeric::Vector2f::new(0.248, 0.248),
            _ => numeric::Vector2f::new(1.0, 1.0),
        }
    }

    pub fn new<'a>(ctx: &mut SuzuContext<'a>, tachie_data: TachieData, t: Clock) -> Self {
        // left
        let left_texture = if tachie_data.left.is_some() {
            let texture_id = tachie_data.left.unwrap();
            Some(SimpleObject::new(
                MovableUniTexture::new(
                    Box::new(UniTexture::new(
                        ctx.ref_texture(texture_id),
                        numeric::Point2f::new(70.0, 88.0),
                        Self::tachie_texture_scale(texture_id),
                        0.0,
                        0,
                    )),
                    None,
                    t,
                ),
                Vec::new(),
            ))
        } else {
            None
        };

        let inner_right_texture = if tachie_data.inner_right.is_some() {
            let texture_id = tachie_data.inner_right.unwrap();
            Some(SimpleObject::new(
                MovableUniTexture::new(
                    Box::new(UniTexture::new(
                        ctx.ref_texture(texture_id),
                        numeric::Point2f::new(680.0, 88.0),
                        Self::tachie_texture_scale(texture_id),
                        0.0,
                        0,
                    )),
                    None,
                    t,
                ),
                Vec::new(),
            ))
        } else {
            None
        };

        let right_texture = if tachie_data.right.is_some() {
            let texture_id = tachie_data.right.unwrap();
            Some(SimpleObject::new(
                MovableUniTexture::new(
                    Box::new(UniTexture::new(
                        ctx.ref_texture(texture_id),
                        numeric::Point2f::new(
                            if inner_right_texture.is_some() {
                                880.0
                            } else {
                                820.0
                            },
                            60.0,
                        ),
                        Self::tachie_texture_scale(texture_id),
                        0.0,
                        0,
                    )),
                    None,
                    t,
                ),
                Vec::new(),
            ))
        } else {
            None
        };

        ScenarioTachie {
            inner_right: inner_right_texture,
            left: left_texture,
            right: right_texture,
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }
}

impl DrawableComponent for ScenarioTachie {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.drwob_essential.visible {
            if let Some(texture) = self.left.as_mut() {
                texture.draw(ctx)?;
            }

            if let Some(texture) = self.right.as_mut() {
                texture.draw(ctx)?;
            }

            if let Some(texture) = self.inner_right.as_mut() {
                texture.draw(ctx)?;
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

#[derive(Clone, Debug)]
pub struct TachieData {
    right: Option<TextureID>,
    inner_right: Option<TextureID>,
    left: Option<TextureID>,
}

impl TachieData {
    pub fn new_empty() -> TachieData {
        TachieData {
            inner_right: None,
            right: None,
            left: None,
        }
    }

    pub fn is_none(&self) -> bool {
        self.right.is_none() && self.left.is_none() && self.inner_right.is_none()
    }
}

pub struct ScenarioText {
    seq_text: Vec<ScenarioTextSegment>,
    iterator: f32,
    current_segment_index: usize,
    total_length: usize,
    scenario_id: ScenarioElementID,
    next_scenario_id: ScenarioElementID,
    background_texture_id: Option<TextureID>,
    tachie_data: TachieData,
}

impl ScenarioText {
    pub fn new(toml_scripts: &toml::value::Value, game_data: &GameResource) -> Self {
        let id = toml_scripts.get("id").unwrap().as_integer().unwrap() as i32;
        let next_id = toml_scripts.get("next-id").unwrap().as_integer().unwrap() as i32;

        let toml_default_attribute = toml_scripts
            .get("default-text-attribute")
            .unwrap()
            .as_table()
            .unwrap();
        let default_font_scale = toml_default_attribute["font_scale"].as_float().unwrap() as f32;

        let default = ScenarioTextAttribute {
            fpc: toml_default_attribute["fpc"].as_float().unwrap() as f32,
            font_info: FontInformation::new(
                game_data.get_font(FontID::Cinema),
                numeric::Vector2f::new(default_font_scale, default_font_scale),
                ggraphics::Color::from_rgba_u32(
                    toml_default_attribute["color"].as_integer().unwrap() as u32,
                ),
            ),
        };

        let mut seq_text = Vec::<ScenarioTextSegment>::new();

        for elem in toml_scripts.get("text").unwrap().as_array().unwrap() {
            if let toml::Value::Table(scenario) = elem {
                seq_text.push(ScenarioTextSegment::from_toml_using_default(
                    scenario, game_data, &default,
                ));
            }
        }

        let background_texture_id = match toml_scripts.get("background") {
            Some(background) => Some(TextureID::from_str(background.as_str().unwrap()).unwrap()),
            None => None,
        };

        let total_length: usize = seq_text.iter().fold(0, |sum, s| sum + s.str_len());

        let tachie_data = if let Some(tachie_table) = toml_scripts.get("tachie-data") {
            TachieData {
                right: if let Some(tid) = tachie_table.get("right") {
                    Some(TextureID::from_str(tid.as_str().unwrap()).unwrap())
                } else {
                    None
                },
                inner_right: if let Some(tid) = tachie_table.get("inner-right") {
                    Some(TextureID::from_str(tid.as_str().unwrap()).unwrap())
                } else {
                    None
                },
                left: if let Some(tid) = tachie_table.get("left") {
                    Some(TextureID::from_str(tid.as_str().unwrap()).unwrap())
                } else {
                    None
                },
            }
        } else {
            TachieData {
                inner_right: None,
                right: None,
                left: None,
            }
        };

        ScenarioText {
            seq_text: seq_text,
            iterator: 0.0,
            current_segment_index: 0,
            total_length: total_length,
            scenario_id: id,
            next_scenario_id: next_id,
            background_texture_id: background_texture_id,
            tachie_data: tachie_data,
        }
    }

    fn current_iterator(&self) -> usize {
        self.iterator as usize
    }

    // 表示する文字数を更新する
    pub fn update_iterator(&mut self) {
        let current_segment = self.seq_text.get(self.current_segment_index).unwrap();
        self.iterator += current_segment.get_fpc();

        if self.iterator as usize >= self.total_length {
            self.iterator = self.total_length as f32;
        }
    }

    pub fn reset_segment(&mut self) {
        self.current_segment_index = 0;
    }

    pub fn set_current_segment(&mut self, index: usize) {
        self.current_segment_index = index;
    }

    pub fn iterator_finish(&self) -> bool {
        self.iterator as usize == self.total_length
    }

    pub fn seq_text_iter(&self) -> std::slice::Iter<ScenarioTextSegment> {
        self.seq_text.iter()
    }

    pub fn get_scenario_id(&self) -> ScenarioElementID {
        self.scenario_id
    }

    pub fn reset(&mut self) {
        self.iterator = 0.0;
        self.current_segment_index = 0;
    }

    pub fn get_background_texture_id(&self) -> Option<TextureID> {
        self.background_texture_id
    }

    pub fn get_tachie_data(&self) -> TachieData {
        self.tachie_data.clone()
    }
}

pub struct ScenarioSwitch {
    opecode: String,
    self_id: ScenarioElementID,
    yes_branch: ScenarioElementID,
    no_branch: ScenarioElementID,
}

impl ScenarioSwitch {
    pub fn from_toml_object(toml_scripts: &toml::value::Value) -> Self {
	ScenarioSwitch {
	    self_id: toml_scripts["id"].as_integer().unwrap() as i32,
	    opecode: toml_scripts["opecode"].as_str().unwrap().to_string(),
	    yes_branch: toml_scripts["yes"].as_integer().unwrap() as i32,
	    no_branch: toml_scripts["no"].as_integer().unwrap() as i32,
	}
    }

    pub fn get_opecode(&self) -> &str {
	self.opecode.as_str()
    }

    pub fn get_yes_branch(&self) -> ScenarioElementID {
	self.yes_branch
    }

    pub fn get_no_branch(&self) -> ScenarioElementID {
	self.no_branch
    }

    pub fn get_self_scenario_id(&self) -> ScenarioElementID {
	self.self_id
    }
}

///
/// 選択肢のデータを保持する構造体
///
pub struct ChoicePatternData {
    header_text: String,
    text: Vec<String>,
    jump_scenario_id: Vec<ScenarioElementID>,
    scenario_id: ScenarioElementID,
    background_texture_id: Option<TextureID>,
    tachie_data: TachieData,
}

impl ChoicePatternData {
    pub fn from_toml_object(toml_scripts: &toml::value::Value, _: &GameResource) -> Self {
        let id = toml_scripts.get("id").unwrap().as_integer().unwrap() as i32;

        let mut choice_pattern_array = Vec::new();
        let mut jump_scenario_array = Vec::new();

        for elem in toml_scripts
            .get("choice-pattern")
            .unwrap()
            .as_array()
            .unwrap()
        {
            choice_pattern_array.push(elem.get("pattern").unwrap().as_str().unwrap().to_string());
            jump_scenario_array
                .push(elem.get("jump-id").unwrap().as_integer().unwrap() as ScenarioElementID);
        }

        let background_texture_id = if let Some(background_tid_str) = toml_scripts.get("background")
        {
            Some(TextureID::from_str(background_tid_str.as_str().unwrap()).unwrap())
        } else {
            None
        };

        let tachie_data = if let Some(tachie_table) = toml_scripts.get("tachie-data") {
            TachieData {
                right: if let Some(tid) = tachie_table.get("right") {
                    Some(TextureID::from_str(tid.as_str().unwrap()).unwrap())
                } else {
                    None
                },
                inner_right: if let Some(tid) = tachie_table.get("inner-right") {
                    Some(TextureID::from_str(tid.as_str().unwrap()).unwrap())
                } else {
                    None
                },
                left: if let Some(tid) = tachie_table.get("left") {
                    Some(TextureID::from_str(tid.as_str().unwrap()).unwrap())
                } else {
                    None
                },
            }
        } else {
            TachieData::new_empty()
        };

        ChoicePatternData {
            header_text: toml_scripts
                .get("header_text")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            text: choice_pattern_array,
            jump_scenario_id: jump_scenario_array,
            scenario_id: id,
            background_texture_id: background_texture_id,
            tachie_data: tachie_data,
        }
    }

    pub fn get_scenario_id(&self) -> ScenarioElementID {
        self.scenario_id
    }

    pub fn get_background_texture_id(&self) -> Option<TextureID> {
        self.background_texture_id
    }

    pub fn get_tachie_data(&self) -> TachieData {
        self.tachie_data.clone()
    }
}

pub struct ChoiceBox {
    header_text: String,
    choice_text: Vec<String>,
    panels: Vec<FramedButton>,
    selecting: Option<usize>,
    canvas: SubScreen,
}

impl ChoiceBox {
    fn generate_choice_panel<'a>(
        ctx: &mut SuzuContext<'a>,
        choice_text: &Vec<String>,
        left_top: numeric::Vector2f,
        align: f32,
    ) -> Vec<FramedButton> {
        let mut choice_panels = Vec::new();
        let mut pos: numeric::Point2f = left_top.into();

        for s in choice_text.iter() {
            let button = util_object::FramedButton::create_design1(
                ctx,
                pos,
                s.as_str(),
                numeric::Vector2f::new(28.0, 28.0),
            );
            pos.x += button.get_area().w + align;
            choice_panels.push(button);
        }

        choice_panels
    }

    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        pos_rect: numeric::Rect,
        header_text: String,
        choice_text: Vec<String>,
    ) -> Self {
        let mut panels =
            Self::generate_choice_panel(ctx, &choice_text, numeric::Vector2f::new(10.0, 0.0), 10.0);

        let mut width = 10.0 * panels.len() as f32;

        for panel in &mut panels {
            width += panel.get_area().w;
            panel.make_this_none_status(ctx);
        }

        ChoiceBox {
            header_text: header_text,
            panels: panels,
            choice_text: choice_text,
            selecting: None,
            canvas: SubScreen::new(
                ctx.context,
                numeric::Rect::new(pos_rect.x, pos_rect.y, width, pos_rect.h),
                0,
                ggraphics::Color::from_rgba_u32(0),
            ),
        }
    }

    pub fn get_selecting_index(&self) -> Option<usize> {
        self.selecting.clone()
    }

    pub fn get_selecting_str(&self) -> Option<&str> {
        if let Some(index) = self.get_selecting_index() {
            if let Some(s) = self.choice_text.get(index) {
                return Some(s);
            }
        }

        None
    }

    pub fn cursor_select<'a>(&mut self, ctx: &mut SuzuContext<'a>, point: numeric::Point2f) {
        let rpoint = self.canvas.relative_point(point);

        self.selecting = None;

        for (index, panel) in self.panels.iter_mut().enumerate() {
            if panel.contains(rpoint) {
                panel.make_this_hovered_status(ctx);
                self.selecting = Some(index);
            } else {
                panel.make_this_none_status(ctx);
            }
        }
    }
}

impl DrawableComponent for ChoiceBox {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            for panel in &mut self.panels {
                panel.draw(ctx)?;
            }

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

impl DrawableObject for ChoiceBox {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for ChoiceBox {
    impl_texture_object_for_wrapped! {canvas}
}

///
/// 選択肢のデータを保持する構造体
///
pub struct ScenarioFinishAndWaitData {
    scenario_id: ScenarioElementID,
    next_id: ScenarioElementID,
    background_texture_id: Option<TextureID>,
    tachie_data: TachieData,
    opecode: String,
}

impl ScenarioFinishAndWaitData {
    pub fn from_toml_object(toml_scripts: &toml::value::Value, _: &GameResource) -> Self {
        let id = toml_scripts.get("id").unwrap().as_integer().unwrap() as i32;
        let next_id = toml_scripts.get("next-id").unwrap().as_integer().unwrap() as i32;
        let opecode = toml_scripts
            .get("opecode")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();

        let background_texture_id = if let Some(background_tid_str) = toml_scripts.get("background")
        {
            Some(TextureID::from_str(background_tid_str.as_str().unwrap()).unwrap())
        } else {
            None
        };

        let tachie_data = if let Some(tachie_table) = toml_scripts.get("tachie-data") {
            TachieData {
                right: if let Some(tid) = tachie_table.get("right") {
                    Some(TextureID::from_str(tid.as_str().unwrap()).unwrap())
                } else {
                    None
                },
                inner_right: if let Some(tid) = tachie_table.get("inner-right") {
                    Some(TextureID::from_str(tid.as_str().unwrap()).unwrap())
                } else {
                    None
                },
                left: if let Some(tid) = tachie_table.get("left") {
                    Some(TextureID::from_str(tid.as_str().unwrap()).unwrap())
                } else {
                    None
                },
            }
        } else {
            TachieData::new_empty()
        };

        ScenarioFinishAndWaitData {
            scenario_id: id,
            next_id: next_id,
            background_texture_id: background_texture_id,
            tachie_data: tachie_data,
            opecode: opecode,
        }
    }

    pub fn get_scenario_id(&self) -> ScenarioElementID {
        self.scenario_id
    }

    pub fn get_background_texture_id(&self) -> Option<TextureID> {
        self.background_texture_id
    }

    pub fn get_tachie_data(&self) -> TachieData {
        self.tachie_data.clone()
    }

    pub fn get_next_id(&self) -> ScenarioElementID {
        self.next_id
    }

    pub fn get_opecode(&self) -> &str {
        self.opecode.as_str()
    }
}

pub struct ScheduleStartEssential {
    scenario_id: ScenarioElementID,
    background_texture_id: Option<TextureID>,
    tachie_data: TachieData,
}

// pub struct ScheduleStartData {
//     scenario_id: ScenarioElementID,
//     background_texture_id: Option<TextureID>,
//     tachie_data: TachieData,
// }

pub enum ScenarioBuiltinCommand {
    ScheduleStart(ScheduleStartEssential),
}

impl ScenarioBuiltinCommand {
    pub fn from_toml_object(toml_scripts: &toml::value::Value) -> Self {
        match toml_scripts
            .get("opecode")
            .unwrap()
            .as_str()
            .expect("invalid ScenarioBuiltinCommand opecode format")
        {
            "StartSchedule" => {
                let id = toml_scripts.get("id").unwrap().as_integer().unwrap() as i32;
                let background_texture_id =
                    if let Some(background_tid_str) = toml_scripts.get("background") {
                        Some(TextureID::from_str(background_tid_str.as_str().unwrap()).unwrap())
                    } else {
                        None
                    };

                let tachie_data = if let Some(tachie_table) = toml_scripts.get("tachie-data") {
                    TachieData {
                        right: if let Some(tid) = tachie_table.get("right") {
                            Some(TextureID::from_str(tid.as_str().unwrap()).unwrap())
                        } else {
                            None
                        },
                        inner_right: if let Some(tid) = tachie_table.get("inner-right") {
                            Some(TextureID::from_str(tid.as_str().unwrap()).unwrap())
                        } else {
                            None
                        },
                        left: if let Some(tid) = tachie_table.get("left") {
                            Some(TextureID::from_str(tid.as_str().unwrap()).unwrap())
                        } else {
                            None
                        },
                    }
                } else {
                    TachieData::new_empty()
                };

                Self::ScheduleStart(ScheduleStartEssential {
                    scenario_id: id,
                    background_texture_id: background_texture_id,
                    tachie_data: tachie_data,
                })
            }
            _ => panic!("Invalid ScenarioBuiltinCommand opecode"),
        }
    }

    pub fn get_scenario_id(&self) -> ScenarioElementID {
        match self {
            ScenarioBuiltinCommand::ScheduleStart(data) => data.scenario_id,
        }
    }

    pub fn get_background_texture_id(&self) -> Option<TextureID> {
        match self {
            ScenarioBuiltinCommand::ScheduleStart(data) => data.background_texture_id.clone(),
        }
    }

    pub fn get_tachie_info(&self) -> TachieData {
        match self {
            ScenarioBuiltinCommand::ScheduleStart(data) => data.tachie_data.clone(),
        }
    }
}

///
/// SceneIDとSceneElementIDのペア
/// ScenarioElementIDを持っていて、SceneEventの切り替えと同じインターフェースの
/// シーンの切り替えが実現できる
///
#[derive(Clone, Copy)]
pub struct ScenarioTransitionData(SceneID, SceneTransition, ScenarioElementID);

pub enum ScenarioElement {
    Text(ScenarioText),
    ChoiceSwitch(ChoicePatternData),
    SceneTransition(ScenarioTransitionData),
    FinishAndWait(ScenarioFinishAndWaitData),
    BuiltinCommand(ScenarioBuiltinCommand),
    Switch(ScenarioSwitch),
}

impl ScenarioElement {
    pub fn get_scenario_id(&self) -> ScenarioElementID {
        match self {
            Self::Text(text) => text.get_scenario_id(),
            Self::ChoiceSwitch(choice) => choice.get_scenario_id(),
            Self::SceneTransition(transition_data) => transition_data.2,
            Self::FinishAndWait(data) => data.get_scenario_id(),
            Self::BuiltinCommand(command) => command.get_scenario_id(),
	    Self::Switch(switch) => switch.get_self_scenario_id(),
        }
    }

    pub fn get_background_texture(&self) -> Option<TextureID> {
        match self {
            Self::Text(text) => text.get_background_texture_id(),
            Self::ChoiceSwitch(choice) => choice.get_background_texture_id(),
            Self::SceneTransition(_) => None,
            Self::FinishAndWait(data) => data.get_background_texture_id(),
            Self::BuiltinCommand(command) => command.get_background_texture_id(),
	    Self::Switch(_) => None,
        }
    }

    pub fn get_tachie_info(&self) -> TachieData {
        match self {
            Self::Text(text) => text.get_tachie_data(),
            Self::ChoiceSwitch(choice) => choice.get_tachie_data(),
            Self::SceneTransition(_) => TachieData::new_empty(),
            Self::FinishAndWait(data) => data.get_tachie_data(),
            Self::BuiltinCommand(command) => command.get_tachie_info(),
	    Self::Switch(_) => TachieData::new_empty(),
        }
    }
}

pub struct ScenarioElementPool {
    pool: Vec<ScenarioElement>,
}

impl ScenarioElementPool {
    pub fn new_empty() -> Self {
        ScenarioElementPool { pool: Vec::new() }
    }

    pub fn add(&mut self, elem: ScenarioElement) {
        self.pool.push(elem);
    }

    ///
    /// 次のScenarioElementIDから、ScenarioElementのインデックスを得るメソッド
    ///
    pub fn find_index_of_specified_scenario_id(&self, scenario_id: ScenarioElementID) -> usize {
        for (index, elem) in self.pool.iter().enumerate() {
            if scenario_id == elem.get_scenario_id() {
                return index;
            }
        }

        0
    }

    pub fn len(&self) -> usize {
        self.pool.len()
    }

    pub fn seq_access(&self, index: usize) -> Option<&ScenarioElement> {
        self.pool.get(index)
    }

    pub fn seq_access_mut(&mut self, index: usize) -> Option<&mut ScenarioElement> {
        self.pool.get_mut(index)
    }
}

pub struct Scenario {
    scenario: ScenarioElementPool,
    element_id_stack: Vec<ScenarioElementID>,
    current_page: usize,
}

impl Scenario {
    pub fn new<'a>(
	ctx: &mut SuzuContext<'a>,
	file_path: &str,
	save_data: Option<&ScenarioSceneSaveData>,
    ) -> Self {
        let game_data = &ctx.resource;

        let mut scenario = ScenarioElementPool::new_empty();

        let root = if file_path == "/scenario/day_7_23.toml" {
	    first_day_scenario
		.parse::<toml::Value>()
		.expect("Failed to parse toml file")
	} else if file_path == "/scenario/time_attack_first.toml" {
	    time_attack_first_day_scenario
		.parse::<toml::Value>()
		.expect("Failed to parse toml file")
	} else if file_path == "/scenario/time_attack_default.toml" {
	    time_attack_default_day_scenario
		.parse::<toml::Value>()
		.expect("Failed to parse toml file")
	} else if file_path == "/scenario/time_attack_week_first.toml" {
	    time_attack_week_first_scenario
		.parse::<toml::Value>()
		.expect("Failed to parse toml file")
	} else if file_path ==  "/scenario/no_enough_hp.toml" {
	    no_enough_hp_scenario
		.parse::<toml::Value>()
		.expect("Failed to parse toml file")	    
	} else {
	    parse_toml_file!(ctx.context, file_path)
	};

        let first_scenario_id = 
	    if let Some(save_data) = save_data {
		println!("first id -> {}", save_data.scenario_id);
		save_data.scenario_id as i64
	    } else {
		root["first-scenario-id"].as_integer().unwrap()
	    };

        let array = root["scenario-group"].as_array().unwrap();

        for elem in array {
            if let Some(type_info) = elem.get("type") {
                match type_info.as_str().unwrap() {
                    "scenario" => {
                        scenario.add(ScenarioElement::Text(ScenarioText::new(elem, game_data)));
                    }
                    "choice" => {
                        scenario.add(ScenarioElement::ChoiceSwitch(
                            ChoicePatternData::from_toml_object(elem, game_data),
                        ));
                    }
                    "wait" => {
                        scenario.add(ScenarioElement::FinishAndWait(
                            ScenarioFinishAndWaitData::from_toml_object(elem, game_data),
                        ));
                    }
                    "builtin" => {
                        scenario.add(ScenarioElement::BuiltinCommand(
                            ScenarioBuiltinCommand::from_toml_object(elem),
                        ));
                    }
		    "switch" => {
			scenario.add(ScenarioElement::Switch(ScenarioSwitch::from_toml_object(elem)));
		    }
                    _ => eprintln!("Error"),
                }
            } else {
                eprintln!("Error");
            }
        }

        // シーン切り替えのScenarioElementをロード
        let scene_transition = root["scene-transition"].as_table().unwrap();
        scenario.add(ScenarioElement::SceneTransition(ScenarioTransitionData(
            SceneID::Scenario,
            SceneTransition::SwapTransition,
            scene_transition
                .get("scenario")
                .unwrap()
                .as_integer()
                .unwrap() as i32,
        )));
        scenario.add(ScenarioElement::SceneTransition(ScenarioTransitionData(
            SceneID::SuzunaShop,
            SceneTransition::SwapTransition,
            scene_transition.get("dream").unwrap().as_integer().unwrap() as i32,
        )));
        scenario.add(ScenarioElement::SceneTransition(ScenarioTransitionData(
            SceneID::Save,
            SceneTransition::StackingTransition,
            scene_transition.get("save").unwrap().as_integer().unwrap() as i32,
        )));

        let mut scenario = Scenario {
            scenario: scenario,
            element_id_stack: Vec::new(),
            current_page: 0,
        };

        scenario.update_current_page_index(first_scenario_id as ScenarioElementID);
        scenario
    }

    ///
    /// 次のScenarioElementIDを記録して、かつ、そのインデックスを求め、現在のシナリオとしてセットするメソッド
    ///
    pub fn update_current_page_index(&mut self, scenario_element_id: ScenarioElementID) {
        // 次のScenarioElementIDから、ScenarioElementのインデックスを得る
        self.element_id_stack.push(scenario_element_id);
        let index = self
            .scenario
            .find_index_of_specified_scenario_id(scenario_element_id);
        self.current_page = index;
    }

    pub fn turn_back_scenario_offset(&mut self, offset: usize) {
        for _ in 0..offset {
            self.element_id_stack.pop();
        }

        let turn_backed_id = self.element_id_stack.last().unwrap();

        self.current_page = self
            .scenario
            .find_index_of_specified_scenario_id(*turn_backed_id);

        // シナリオを初期化する
        match self.ref_current_element_mut() {
            ScenarioElement::Text(obj) => {
                obj.reset();
            }
            _ => (),
        }
    }

    ///
    /// Scenarioの状態遷移を行うメソッド
    /// このメソッドは通常のテキストから他の状態に遷移する際に呼び出す
    ///
    pub fn go_next_scenario_from_waiting(&mut self) {
        // 次のScenarioElementIDは、ScenarioTextがフィールドとして保持しているので取り出す
        let next_id = match self.ref_current_element_mut() {
            ScenarioElement::FinishAndWait(data) => data.get_next_id(),
            _ => {
                panic!("Error: go_next_scenario_from_waiting");
            }
        };

        self.update_current_page_index(next_id);

        // 次がシナリオなら初期化する
        match self.ref_current_element_mut() {
            ScenarioElement::Text(obj) => {
                obj.reset();
            }
            _ => (),
        }
    }

    ///
    /// Scenarioの状態遷移を行うメソッド
    /// このメソッドは通常のテキストから他の状態に遷移する際に呼び出す
    ///
    pub fn go_next_scenario_from_text_scenario<'a>(&mut self) {
        // 次のScenarioElementIDは、ScenarioTextがフィールドとして保持しているので取り出す
        let next_id = match self.ref_current_element_mut() {
            ScenarioElement::Text(obj) => obj.next_scenario_id,
            _ => {
                panic!("Error: go_next_scenario_from_text_scenario");
            }
        };

        self.update_current_page_index(next_id);

        // 次がシナリオなら初期化する
        match self.ref_current_element_mut() {
            ScenarioElement::Text(obj) => {
                obj.reset();
            }
            _ => (),
        }
    }

    ///
    /// Scenarioの状態遷移を行うメソッド
    /// このメソッドは選択肢から他の状態に遷移する際に呼び出す
    ///
    pub fn go_next_scenario_from_choice_scenario(&mut self, select_index: usize) {
        // 次のScenarioElementIDは、選択肢から選ぶ
        let next_id = match self.ref_current_element_mut() {
            ScenarioElement::ChoiceSwitch(obj) => {
                // 選択中の選択肢のジャンプ先を取得する
                *obj.jump_scenario_id.get(select_index).unwrap()
            }
            _ => {
                panic!("Error: go_next_scenario_from_text_scenario");
            }
        };

        self.update_current_page_index(next_id);

        // シナリオを初期化する
        match self.ref_current_element_mut() {
            ScenarioElement::Text(obj) => {
                obj.reset();
            }
            _ => (),
        }
    }

    pub fn final_page(&self) -> bool {
        self.scenario.len() - 1 == self.current_page
    }

    pub fn update_current_page(&mut self) {
        match self.ref_current_element_mut() {
            ScenarioElement::Text(scenario_text) => {
                scenario_text.update_iterator();
            }
            _ => (),
        }
    }

    pub fn ref_current_element(&self) -> &ScenarioElement {
        self.scenario.seq_access(self.current_page).unwrap()
    }

    pub fn ref_current_element_mut(&mut self) -> &mut ScenarioElement {
        self.scenario.seq_access_mut(self.current_page).unwrap()
    }

    pub fn get_waiting_opecode(&self) -> Option<&str> {
        match self.ref_current_element() {
            ScenarioElement::FinishAndWait(data) => Some(data.get_opecode()),
            _ => None,
        }
    }

    pub fn release_waiting<'a>(&mut self) {
        self.go_next_scenario_from_waiting();
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TextBoxStatus {
    WaitNextLineKey,
    UpdatingText,
    FixedText,
}

pub struct TextBox {
    box_lines: usize,
    buffered_text: VecDeque<SimpleText>,
    head_line_number: u32,
    text: VecDeque<SimpleText>,
    line_arrow: UniTexture,
    text_box_status: TextBoxStatus,
    appearance_frame: TileBatchFrame,
    complete_and_wait_current_line: bool,
    background: SimpleObject,
    canvas: SubScreen,
    const_canvas: SubScreen,
}

impl TextBox {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        rect: numeric::Rect,
        mut background: SimpleObject,
        tile_batch_texture_id: TileBatchTextureID,
        box_lines: usize,
        _t: Clock,
    ) -> Self {
        background.fit_scale(ctx.context, numeric::Vector2f::new(rect.w, rect.h));
        background.set_position(numeric::Point2f::new(0.0, 0.0));

        let appr_frame = TileBatchFrame::new(
            ctx.resource,
            tile_batch_texture_id,
            numeric::Rect::new(0.0, 0.0, rect.w, rect.h),
            numeric::Vector2f::new(0.75, 0.75),
            0,
        );

        let mut line_arrow = UniTexture::new(
            ctx.ref_texture(TextureID::NextLineIcon),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
        );
        line_arrow.fit_scale(ctx.context, numeric::Vector2f::new(28.0, 28.0));
        line_arrow.hide();

        let mut text_box = TextBox {
            box_lines: box_lines,
            buffered_text: VecDeque::new(),
            head_line_number: 0,
            text: VecDeque::new(),
            text_box_status: TextBoxStatus::UpdatingText,
            background: background,
            appearance_frame: appr_frame,
            complete_and_wait_current_line: false,
            line_arrow: line_arrow,
            canvas: SubScreen::new(
                ctx.context,
                rect,
                0,
                ggraphics::Color::from_rgba_u32(0xffffffff),
            ),
            const_canvas: SubScreen::new(
                ctx.context,
                numeric::Rect::new(0.0, 0.0, rect.w, rect.h),
                0,
                ggraphics::Color::from_rgba_u32(0x00ffffff),
            ),
        };
        text_box.draw_const_canvas(ctx);
        text_box
    }

    fn draw_const_canvas<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        sub_screen::stack_screen(ctx.context, &self.const_canvas);

        self.background.draw(ctx.context).unwrap();
        self.appearance_frame.draw(ctx.context).unwrap();

        sub_screen::pop_screen(ctx.context);
    }

    // ScenarioTextSegmentを改行で分割しVec<SimpleText>に変換する
    pub fn text_from_segment(segment: &ScenarioTextSegment, length: usize) -> Vec<SimpleText> {
        let mut text_lines = Vec::new();

        for line in segment.slice(length).lines() {
            text_lines.push(SimpleText::new(
                tobj::MovableText::new(
                    Box::new(tobj::UniText::new(
                        line.to_string(),
                        numeric::Point2f::new(0.0, 0.0),
                        numeric::Vector2f::new(1.0, 1.0),
                        0.0,
                        0,
                        segment.attribute.font_info,
                    )),
                    None,
                    0,
                ),
                Vec::new(),
            ));
        }

        text_lines
    }

    pub fn update_scenario_text<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        scenario: &ScenarioText,
    ) -> usize {
        let before_lines = self.text.len() + self.buffered_text.len();

        // 表示するテキストバッファをクリア。これで、新しくテキストを詰めていく
        self.text.clear();
        self.buffered_text.clear();

        // 表示する文字数を取得。この値を減算していき文字数チェックを行う
        let mut remain = scenario.current_iterator() as i32;

        let mut segs: Vec<(i32, &ScenarioTextSegment)> = Vec::new();

        // buffered または textにpushされるテキストを保持するTextSegmentを取得
        for seg in scenario.seq_text_iter() {
            // このテキストセグメントの文字数を取得
            let seg_str_len = seg.str_len() as i32;
            // セグメントの切り出す文字数を計算
            let slice_len = if seg_str_len < remain {
                seg_str_len
            } else {
                remain
            };

            segs.push((slice_len, seg));

            if remain < seg_str_len {
                break;
            }

            remain -= seg_str_len;
        }

        let mut text_lines = VecDeque::new();
        for (slice_len, seg) in segs.iter() {
            for line in Self::text_from_segment(seg, *slice_len as usize) {
                text_lines.push_back(line);
            }
        }

        if !self.complete_and_wait_current_line {
            if text_lines.len() > before_lines && text_lines.len() > self.box_lines {
                self.text_box_status = TextBoxStatus::WaitNextLineKey;
                self.complete_and_wait_current_line = true;
                text_lines.pop_back();
            }
        } else {
            self.complete_and_wait_current_line = false;
        }

        for _ in 0..self.box_lines {
            if let Some(text) = text_lines.pop_back() {
                self.text.push_front(text);
            }
        }

        self.buffered_text = text_lines;

        // ボックスに入ったSimpleTextの位置を設定
        let mut pos = numeric::Point2f::new(60.0, 60.0);
        for line in &mut self.text {
            line.set_position(pos);
            pos.y += line.get_font_scale().y;
        }

        if self.text_box_status == TextBoxStatus::WaitNextLineKey || scenario.iterator_finish() {
            self.line_arrow.appear();
            let last_text_drawing_area = self.text.back().unwrap().get_drawing_area(ctx.context);
            let pos = numeric::Point2f::new(
                last_text_drawing_area.x + last_text_drawing_area.w,
                last_text_drawing_area.y,
            );
            self.line_arrow.set_position(pos);
        }

        segs.len() - 1
    }

    pub fn set_fixed_text(&mut self, text: String, font_info: FontInformation) {
        self.text.clear();
        self.text.push_back(SimpleText::new(
            tobj::MovableText::new(
                Box::new(tobj::UniText::new(
                    text,
                    numeric::Point2f::new(60.0, 60.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    font_info,
                )),
                None,
                0,
            ),
            Vec::new(),
        ));

        self.set_text_box_status(TextBoxStatus::FixedText);
    }

    pub fn next_button_handler(&mut self) {
        self.head_line_number += 1;
        self.set_text_box_status(TextBoxStatus::UpdatingText);
    }

    pub fn reset_head_line(&mut self) {
        self.head_line_number = 0;
    }

    pub fn set_text_box_status(&mut self, status: TextBoxStatus) {
        self.text_box_status = status;
    }
}

impl DrawableComponent for TextBox {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.const_canvas.draw(ctx)?;
            for d in &mut self.text {
                d.draw(ctx)?;
            }

            self.line_arrow.draw(ctx)?;

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

pub struct ScenarioBox {
    pub text_box: TextBox,
    pub choice_box: Option<ChoiceBox>,
    drwob_essential: DrawableObjectEssential,
}

impl ScenarioBox {
    pub fn new<'a>(ctx: &mut SuzuContext, rect: numeric::Rect, t: Clock) -> Self {
        let background = tobj::SimpleObject::new(
            tobj::MovableUniTexture::new(
                Box::new(UniTexture::new(
                    ctx.ref_texture(TextureID::TextBackground),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                )),
                None,
                0,
            ),
            Vec::new(),
        );
        ScenarioBox {
            text_box: TextBox::new(
                ctx,
                rect,
                background,
                TileBatchTextureID::TaishoStyle1,
                3,
                t,
            ),
            choice_box: None,
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }

    pub fn contains(&self, point: numeric::Point2f) -> bool {
        self.text_box.canvas.contains(point)
    }

    // pub fn new_choice<'a>(
    //     ctx: &mut SuzuContext<'a>,
    //     rect: numeric::Rect,
    //     choice_pattern: ChoicePatternData,
    //     font_info: FontInformation,
    //     t: Clock,
    // ) -> Self {
    //     let background = tobj::SimpleObject::new(
    //         tobj::MovableUniTexture::new(
    //             Box::new(UniTexture::new(
    //                 ctx.ref_texture(TextureID::TextBackground),
    //                 numeric::Point2f::new(20.0, 20.0),
    //                 numeric::Vector2f::new(0.8, 0.8),
    //                 0.0,
    //                 0,
    //             )),
    //             None,
    //             0,
    //         ),
    //         Vec::new(),
    //     );

    // 	let mut choice_box = ChoiceBox::new(
    //         ctx,
    //         numeric::Rect::new(400.0, 120.0, 1200.0, 150.0),
    // 	    choice_pattern.header_text.clone(),
    //         choice_pattern.text.clone(),
    //     );
    // 	choice_box.make_center(ctx.context, numeric::Point2f::new(rect.w / 2.0, rect.h / 2.0));

    //     let mut scenario_box = ScenarioBox {
    //         text_box: TextBox::new(
    //             ctx,
    //             numeric::Rect::new(0.0, 0.0, rect.w, rect.h),
    //             background,
    //             TileBatchTextureID::TaishoStyle1,
    //             3,
    //             t,
    //         ),
    //         choice_box: Some(choice_box),
    //         canvas: SubScreen::new(ctx.context, rect, 0, ggraphics::Color::from_rgba_u32(0x00)),
    //     };
    //     scenario_box.display_choice_box_text(font_info);

    //     scenario_box
    // }

    pub fn get_text_box_status(&self) -> TextBoxStatus {
        self.text_box.text_box_status
    }

    pub fn update_scenario_text<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        scenario: &ScenarioText,
    ) -> usize {
        self.text_box.update_scenario_text(ctx, scenario)
    }

    pub fn is_enable_choice_box(&self) -> bool {
        self.choice_box.is_some()
    }

    pub fn insert_choice_box(&mut self, choice_box: Option<ChoiceBox>) {
        self.choice_box = choice_box;
    }

    pub fn display_choice_box_text(&mut self, font_info: FontInformation) {
        if self.choice_box.is_some() {
            // テキストボックスに選択肢の文字列を表示する
            let header_text = self.choice_box.as_ref().unwrap().header_text.as_str();
            let _selected_text =
                if let Some(s) = self.choice_box.as_ref().unwrap().get_selecting_str() {
                    s.to_string()
                } else {
                    "".to_string()
                };
            //self.text_box.set_fixed_text(&format!("{}\n{}", header_text, selected_text), font_info);
            self.text_box
                .set_fixed_text(header_text.to_string(), font_info);
        }
    }

    pub fn reset_head_line(&mut self) {
        self.text_box.reset_head_line();
    }

    pub fn get_choice_selecting_index(&self) -> Option<usize> {
        if let Some(choice) = self.choice_box.as_ref() {
            choice.get_selecting_index()
        } else {
            None
        }
    }
}

impl DrawableComponent for ScenarioBox {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.text_box.draw(ctx)?;

            if let Some(choice) = self.choice_box.as_mut() {
                choice.draw(ctx)?;
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

#[derive(PartialEq, Clone, Copy)]
pub enum ScenarioEventStatus {
    Scenario = 0,
    Choice,
    SceneTransition,
    FinishAndWait,
    StartSchedule,
    BuiltinSwitch,
}

pub struct ScenarioEvent {
    scenario: Scenario,
    scenario_box: ScenarioBox,
    canvas: SubScreen,
    status: ScenarioEventStatus,
    transition_scene: Option<SceneID>,
    transition_type: Option<SceneTransition>,
    background: Option<UniTexture>,
    tachie: Option<ScenarioTachie>,
    appearance_frame: TileBatchFrame,
    redraw_request: DrawRequest,
    se_handlers: [Option<SoundHandler>; 1],
}

impl ScenarioEvent {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        rect: numeric::Rect,
        file_path: &str,
	save_data: Option<&ScenarioSceneSaveData>,
        hide_shadow: bool,
        t: Clock,
    ) -> Self {
        let scenario = Scenario::new(ctx, file_path, save_data);

        let event_background = if let Some(mut texture) =
            Self::update_event_background_sub(ctx, scenario.ref_current_element())
        {
            texture.fit_scale(ctx.context, numeric::Vector2f::new(rect.w, rect.h));
            Some(texture)
        } else {
            None
        };

        let event_tachie = Self::update_event_tachie_sub(ctx, scenario.ref_current_element(), t);

        let mut appr_frame = TileBatchFrame::new(
            ctx.resource,
            TileBatchTextureID::BlackFrame,
            numeric::Rect::new(0.0, 0.0, rect.w, rect.h),
            numeric::Vector2f::new(1.0, 1.0),
            0,
        );
        if hide_shadow {
            appr_frame.hide();
        }

        ScenarioEvent {
            scenario: scenario,
            scenario_box: ScenarioBox::new(
                ctx,
                numeric::Rect::new(20.0, rect.bottom() - 250.0, 1326.0, 235.0),
                t,
            ),
            canvas: SubScreen::new(ctx.context, rect, 0, ggraphics::Color::from_rgba_u32(0x00)),
            status: ScenarioEventStatus::Scenario,
            transition_scene: None,
            transition_type: None,
            background: event_background,
            appearance_frame: appr_frame,
            tachie: event_tachie,
            redraw_request: DrawRequest::InitDraw,
            se_handlers: [None],
        }
    }

    pub fn replace_scenario<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        scno_ctx: &mut ScenarioContext,
        scenario_path: &str,
        t: Clock,
    ) {
	let save_data = ctx.take_save_data_mut().get_scenario_save_data();
        self.scenario = Scenario::new(ctx, scenario_path, save_data.as_ref());
        self.status = ScenarioEventStatus::Scenario;
        self.key_down_action1(ctx, None, t);
        self.update_text(ctx, Some(scno_ctx));
        self.redraw_request = DrawRequest::Draw;
    }

    pub fn update_event_background_sub<'a>(
        ctx: &mut SuzuContext<'a>,
        scenario_element: &ScenarioElement,
    ) -> Option<UniTexture> {
        // ScenarioEventの背景を設定
        // ScenarioElementが背景情報を持っていれば、設定を行う
        if let Some(texture_id) = scenario_element.get_background_texture() {
            // 持っていたので、テクスチャを生成し、画面にフィットさせる
            Some(UniTexture::new(
                ctx.ref_texture(texture_id),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
            ))
        } else {
            None
        }
    }

    pub fn update_event_background<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        // 現在のScenarioElementに背景がある場合、背景を変更
        // そうでない場合は、何もしない
        if let Some(mut texture) =
            Self::update_event_background_sub(ctx, self.scenario.ref_current_element())
        {
            let canvas_size = self.canvas.get_drawing_size(ctx.context);
            texture.fit_scale(ctx.context, canvas_size);
            self.background = Some(texture);
            self.redraw_request = DrawRequest::Draw;
        }
    }

    pub fn update_event_tachie_sub<'a>(
        ctx: &mut SuzuContext<'a>,
        scenario_element: &ScenarioElement,
        t: Clock,
    ) -> Option<ScenarioTachie> {
        // ScenarioEventの立ち絵データ取り出し
        // ScenarioElementが立ち絵情報を持っていれば、取り出す
        let tachie_data = scenario_element.get_tachie_info();

        if !tachie_data.is_none() {
            Some(ScenarioTachie::new(ctx, tachie_data, t))
        } else {
            None
        }
    }

    pub fn update_event_tachie<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        // 現在のScenarioElementに立ち絵がある場合、立ち絵データを取り込み
        // そうでない場合は、何もしない
        let scenario_tachie =
            Self::update_event_tachie_sub(ctx, self.scenario.ref_current_element(), t);
        if scenario_tachie.is_some() {
            self.tachie = scenario_tachie;
            self.redraw_request = DrawRequest::Draw;
        }
    }

    pub fn scenario_control_mut(&mut self) -> &mut Scenario {
        self.redraw_request = DrawRequest::Draw;
        &mut self.scenario
    }

    ///
    /// 表示しているテキストや選択肢を更新するメソッド
    ///
    pub fn update_text<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        scno_ctx: Option<&mut ScenarioContext>,
    ) {
        match self.scenario.ref_current_element_mut() {
            ScenarioElement::Text(scenario_text) => {
                if self.scenario_box.get_text_box_status() == TextBoxStatus::UpdatingText {
                    // 表示する文字数を更新
                    scenario_text.update_iterator();

                    // 何行目までのテキストが表示されたか？
                    let current_segment =
                        self.scenario_box.update_scenario_text(ctx, &scenario_text);

                    // どこまで表示したかを更新
                    scenario_text.set_current_segment(current_segment);

                    // 再描画要求
                    ctx.process_utility.redraw();
                    self.redraw_request = DrawRequest::Draw;
                }
            }
            ScenarioElement::ChoiceSwitch(choice_pattern) => {
                // ChoiceBoxが表示されていない場合、新しくオブジェクトを生成する
                if !self.scenario_box.is_enable_choice_box() {
                    let mut choice_box = ChoiceBox::new(
                        ctx,
                        numeric::Rect::new(400.0, 100.0, 1200.0, 150.0),
                        choice_pattern.header_text.clone(),
                        choice_pattern.text.clone(),
                    );

                    let scenario_box_p = self.scenario_box.text_box.canvas.get_position();
                    choice_box.make_center(
                        ctx.context,
                        numeric::Point2f::new(1326.0 / 2.0, scenario_box_p.y + 180.0),
                    );
                    self.scenario_box.insert_choice_box(Some(choice_box));

                    // テキストボックスに選択肢の文字列を表示する
                    self.scenario_box
                        .display_choice_box_text(FontInformation::new(
                            ctx.resource.get_font(FontID::Cinema),
                            numeric::Vector2f::new(32.0, 32.0),
                            ggraphics::Color::from_rgba_u32(0x000000ff),
                        ));
                    // 状態を選択中に変更
                    self.status = ScenarioEventStatus::Choice;

                    // 再描画要求
                    ctx.process_utility.redraw();
                    self.redraw_request = DrawRequest::Draw;
                }
            }
            ScenarioElement::SceneTransition(transition_data) => {
                // SceneTransition状態に移行し、移行先を決定する
                self.status = ScenarioEventStatus::SceneTransition;
                self.transition_scene = Some(transition_data.0);
                self.transition_type = Some(transition_data.1);

                // 再描画要求
                ctx.process_utility.redraw();
                self.redraw_request = DrawRequest::Draw;
            }
            ScenarioElement::FinishAndWait(_) => {
                if let Some(scno_ctx) = scno_ctx {
                    self.status = ScenarioEventStatus::FinishAndWait;
                    scno_ctx.scenario_is_finish_and_wait = true;
                }
            }
            ScenarioElement::BuiltinCommand(ope) => match ope {
                ScenarioBuiltinCommand::ScheduleStart(_) => {
                    self.status = ScenarioEventStatus::StartSchedule;
                }
            },
	    ScenarioElement::Switch(switch) => {
		let next_id = match switch.get_opecode() {
		    "cleared" => {
			if ctx.take_save_data().game_cleared() {
			    switch.get_yes_branch()
			} else {
			    switch.get_no_branch()
			}
		    },
		    _ => panic!("Scenario Script BUG"),
		};

		self.scenario.update_current_page_index(next_id);
		
		// 次がシナリオなら初期化する
		match self.scenario.ref_current_element_mut() {
		    ScenarioElement::Text(obj) => {
			obj.reset();
		    }
		    _ => (),
		}
		self.update_event_tachie(ctx, 0);
	    }
        }
    }

    pub fn contains_scenario_text_box(&self, point: numeric::Point2f) -> bool {
        let rpoint = self.canvas.relative_point(point);
        self.scenario_box.contains(rpoint)
    }

    pub fn make_scenario_event(&mut self) {
        self.status = ScenarioEventStatus::Scenario;
    }

    pub fn go_next_line(&mut self) {
        self.redraw_request = DrawRequest::Draw;
        self.scenario_box.text_box.next_button_handler();
    }

    pub fn get_scene_transition(&self) -> Option<SceneID> {
        self.transition_scene
    }

    pub fn get_scene_transition_type(&self) -> Option<SceneTransition> {
        self.transition_type
    }

    pub fn get_status(&self) -> ScenarioEventStatus {
        self.status
    }

    pub fn get_scenario_waiting_opecode(&self) -> Option<&str> {
        self.scenario.get_waiting_opecode()
    }

    pub fn release_scenario_waiting<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        self.redraw_request = DrawRequest::Draw;
        self.scenario.release_waiting();

        self.update_event_background(ctx);
        self.update_event_tachie(ctx, 0);
    }

    pub fn set_fixed_text_to_scenario_box<'a>(&mut self, ctx: &mut SuzuContext<'a>, text: String) {
        self.redraw_request = DrawRequest::Draw;
        self.scenario_box.text_box.set_fixed_text(
            text,
            FontInformation::new(
                ctx.resource.get_font(FontID::Cinema),
                numeric::Vector2f::new(32.0, 32.0),
                ggraphics::Color::BLACK,
            ),
        );
    }

    ///
    /// Action1キーが押されたときの、ScenarioEventの挙動
    ///
    pub fn key_down_action1<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        click_point: Option<numeric::Point2f>,
        _: Clock,
    ) {
        match self.scenario.ref_current_element_mut() {
            // 現在のScenarioElementがテキスト
            ScenarioElement::Text(scenario_text) => {
                self.redraw_request = DrawRequest::Draw;
                self.scenario_box.text_box.line_arrow.hide();

                // 最後まで到達していた場合、新しいScenarioElementに遷移し、テキストボックスをリセット
                if scenario_text.iterator_finish() {
                    self.scenario.go_next_scenario_from_text_scenario();
                    self.update_event_background(ctx);
                    self.update_event_tachie(ctx, 0);
                    self.scenario_box.reset_head_line();
                    return;
                }

                if self.scenario_box.is_enable_choice_box() {
                    self.make_scenario_event();

                    // choice_boxは消す
                    self.scenario_box.insert_choice_box(None);
                } else {
                    // すでにchoice_boxがNoneなら、text_boxの行を進める動作
                    self.go_next_line();

                    if self.se_handlers[0].is_none()
                        || !ctx.is_se_playing(self.se_handlers[0].unwrap())
                    {
                        self.se_handlers[0] = Some(ctx.play_sound_as_se(SoundID::SeMessage, None));
                    }
                }
            }
            ScenarioElement::ChoiceSwitch(_) => {
                if click_point.is_none() {
                    return;
                }

                let rpoint = self.canvas.relative_point(click_point.unwrap());

                self.scenario_box
                    .choice_box
                    .as_mut()
                    .unwrap()
                    .cursor_select(ctx, rpoint);
                let maybe_index = self.scenario_box.get_choice_selecting_index();
                if maybe_index.is_none() {
                    return;
                }

                self.scenario
                    .go_next_scenario_from_choice_scenario(maybe_index.unwrap());
                self.update_event_background(ctx);
                self.update_event_tachie(ctx, 0);

                self.scenario_box
                    .text_box
                    .set_text_box_status(TextBoxStatus::UpdatingText);

                // choice_boxは消す
                self.scenario_box.insert_choice_box(None);
                self.redraw_request = DrawRequest::Draw;
            }
            ScenarioElement::SceneTransition(_) => (),
            ScenarioElement::FinishAndWait(_) => (),
            ScenarioElement::BuiltinCommand(_) => (),
	    ScenarioElement::Switch(_) => (),
        }
    }

    pub fn mouse_motion_handler<'a>(&mut self, ctx: &mut SuzuContext<'a>, point: numeric::Point2f) {
        let rpoint = self.canvas.relative_point(point);

        match self.scenario.ref_current_element_mut() {
            ScenarioElement::ChoiceSwitch(_) => {
                if let Some(choice) = self.scenario_box.choice_box.as_mut() {
                    choice.cursor_select(ctx, rpoint);

                    self.scenario_box
                        .display_choice_box_text(FontInformation::new(
                            ctx.resource.get_font(FontID::Cinema),
                            numeric::Vector2f::new(32.0, 32.0),
                            ggraphics::Color::from_rgba_u32(0x000000ff),
                        ));
                    self.redraw_request = DrawRequest::Draw;
		    ctx.process_utility.redraw();
                }
            }
            _ => (),
        }
    }

    pub fn get_scenario_id_for_saving(&self) -> i32 {
	let len = self.scenario.element_id_stack.len();
	self.scenario.element_id_stack[len - 2 as usize]
    }
}

impl DrawableComponent for ScenarioEvent {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            if self.redraw_request.is_not_skip() {
                self.redraw_request = DrawRequest::Skip;
                sub_screen::stack_screen(ctx, &self.canvas);

                if let Some(background) = self.background.as_mut() {
                    background.draw(ctx)?;
                }

                if let Some(tachie) = self.tachie.as_mut() {
                    tachie.draw(ctx)?;
                }

                self.scenario_box.draw(ctx)?;

                self.appearance_frame.draw(ctx)?;

                sub_screen::pop_screen(ctx);
            }
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

const first_day_scenario: &str = "first-scenario-id = 6

[scene-transition]
scenario = 1
dream = 2
save = 4

[[scenario-group]]
type = \"choice\"
header_text = \"エピソードとチュートリアルを短縮しますか？\"
background = \"SightBackground1\"	
id = 6

   [[scenario-group.choice-pattern]]
   pattern = \"はい\"
   jump-id = 7
   [[scenario-group.choice-pattern]]
   pattern = \"いいえ\"
   jump-id = 8

[[scenario-group]]
type = \"wait\"
id = 7
next-id = 9
opecode = \"DisableTutorial\"
background = \"SightBackground1\"

[[scenario-group]]
type = \"scenario\"
id = 8
next-id = 201
background = \"SightBackground1\"

[scenario-group.tachie-data]
right = \"KosuzuTachie1\"

   [scenario-group.default-text-attribute]
   fpc = 2.0
   font_scale = 32.0
   color = 0x000000ff

   [[scenario-group.text]]
   text = \"さてと、昨日の続きでも読もうかな。\"

[[scenario-group]]
type = \"scenario\"
id = 201
next-id = 202
background = \"SightBackground1\"

[scenario-group.tachie-data]
right = \"KosuzuTachie1\"
left = \"NitoriTachieSunGlass\"

   [scenario-group.default-text-attribute]
   fpc = 2.0
   font_scale = 32.0
   color = 0x000000ff

   [[scenario-group.text]]
   text_src_path = \"./resources/scenario/7_23/1.txt\"

[[scenario-group]]
type = \"scenario\"
id = 202
next-id = 203
background = \"TownBackground\"

[scenario-group.tachie-data]
left = \"NitoriTachieDefault\"

   [scenario-group.default-text-attribute]
   fpc = 2.0
   font_scale = 32.0
   color = 0x000000ff

   [[scenario-group.text]]
   text_src_path = \"./resources/scenario/7_23/2.txt\"

[[scenario-group]]
type = \"scenario\"
id = 203
next-id = 101
background = \"SightBackground1\"

[scenario-group.tachie-data]
right = \"KosuzuTachie1\"

   [scenario-group.default-text-attribute]
   fpc = 2.0
   font_scale = 32.0
   color = 0x000000ff

   [[scenario-group.text]]
   text_src_path = \"./resources/scenario/7_23/3.txt\"

[[scenario-group]]
type = \"wait\"
id = 9
next-id = 20
opecode = \"ShowStatusScreen\"
background = \"SightBackground1\"

[[scenario-group]]
type = \"scenario\"
id = 10
next-id = 101
background = \"SightBackground1\"

[scenario-group.tachie-data]
right = \"KosuzuTachie1\"

   [scenario-group.default-text-attribute]
   fpc = 2.0
   font_scale = 32.0
   color = 0x000000ff

   [[scenario-group.text]]
   text_src_path = \"./resources/scenario/7_23/1.txt\"

[[scenario-group]]
type = \"scenario\"
id = 101
next-id = 11
background = \"SightBackground1\"

[[scenario-group.text]]
text = \"\"\"

ちょっと何張り切ってんのよ。

あ、阿求じゃない。ちょっとカクカクシカジカで・・・。

・・・

ふーん。大金を稼がないといけないことは分かったけど大丈夫なの？

まあ何回かお店の手伝いはしたことはあるから。

どうも心配ネ。ここは私が稗田家に伝わる商売術を伝授するわよ。

稗田家に商売術なんて伝わってるのかしら。

\"\"\"


[scenario-group.tachie-data]
right = \"KosuzuTachie1\"
left = \"AkyuTachieDefault\"

   [scenario-group.default-text-attribute]
   fpc = 2.0
   font_scale = 32.0
   color = 0x000000ff

[[scenario-group]]
type = \"wait\"
id = 11
next-id = 12
opecode = \"ShowStatusScreen\"
background = \"SightBackground1\"

[[scenario-group]]
type = \"wait\"
id = 12
next-id = 13
opecode = \"ShowAd\"
background = \"SightBackground1\"

[[scenario-group]]
type = \"scenario\"
id = 13
next-id = 14
background = \"SightBackground1\"

[[scenario-group.text]]
text = \"\"\"

まずこれネ。お店の評判は重要よ。お客さんがたくさん来てくれるかもしれないわ。
もし悪評が広まれば商売上がったりネ。\"\"\"


[scenario-group.tachie-data]
right = \"KosuzuTachie1\"
inner-right = \"AkyuTachieDefault\"

   [scenario-group.default-text-attribute]
   fpc = 2.0
   font_scale = 32.0
   color = 0x000000ff

[[scenario-group]]
type = \"wait\"
id = 14
next-id = 15
opecode = \"ShowAdAgency\"
background = \"SightBackground1\"

[[scenario-group]]
type = \"scenario\"
id = 15
next-id = 16
background = \"SightBackground1\"

[[scenario-group.text]]
text = \"\"\"

次にこれ。人間の里で商売してるのはあなただけじゃない。
みんな自分の店の宣伝をしようとしているわ。
その手段として他のお店に宣伝をお願いすることもあるのよ。
鈴奈庵も例外じゃないわ。
鈴奈庵の評判が良ければ大きなお店から宣伝をお願いされるかも。
宣伝をすることでもられるお金も立派なお給料よ。
\"\"\"

[scenario-group.tachie-data]
right = \"KosuzuTachie1\"
inner-right = \"AkyuTachieDefault\"

   [scenario-group.default-text-attribute]
   fpc = 2.0
   font_scale = 32.0
   color = 0x000000ff

[[scenario-group]]
type = \"wait\"
id = 16
next-id = 17
opecode = \"ShowSchedule\"
background = \"SightBackground1\"

[[scenario-group]]
type = \"scenario\"
id = 17
next-id = 18
background = \"SightBackground1\"

[[scenario-group.text]]
text = \"\"\"

次はこれを見て。まだ今週の予定は決めてないみたいネ。
週の始まりにはその週の予定を決めましょう。\"\"\"

[scenario-group.tachie-data]
right = \"KosuzuTachie1\"
inner-right = \"AkyuTachieDefault\"

   [scenario-group.default-text-attribute]
   fpc = 2.0
   font_scale = 32.0
   color = 0x000000ff

[[scenario-group]]
type = \"wait\"
id = 18
next-id = 19
opecode = \"ShowMain\"
background = \"SightBackground1\"

[[scenario-group]]
type = \"scenario\"
id = 19
next-id = 20
background = \"SightBackground1\"

[[scenario-group.text]]
text = \"\"\"

最後にこれ。今の所持金だったりお店の状態が分かるわ。
これを参考にしながら計画的にお仕事を頑張りなさい。

分かったような分からないような。

習うより慣れろよ。それじゃ、また今度鈴奈庵に来るわネ。

一体何しに来たんだ。\"\"\"

[scenario-group.tachie-data]
right = \"KosuzuTachie1\"
inner-right = \"AkyuTachieDefault\"

   [scenario-group.default-text-attribute]
   fpc = 2.0
   font_scale = 32.0
   color = 0x000000ff

[[scenario-group]]
type = \"wait\"
id = 20
next-id = 21
opecode = \"ScheduleCheck\"
background = \"SightBackground1\"

	   [scenario-group.tachie-data]
	   right = \"KosuzuTachie1\"

[[scenario-group]]
type = \"wait\"
id = 21
next-id = 22
opecode = \"ShowAd\"
background = \"SightBackground1\"

[[scenario-group]]
type = \"choice\"
header_text = \"宣伝の依頼もやらないとネ\"
id = 22
   [scenario-group.tachie-data]
   right = \"KosuzuTachie1\"

   [[scenario-group.choice-pattern]]
   pattern = \"完了\"
   jump-id = 23

[[scenario-group]]
type = \"wait\"
id = 23
next-id = 24
opecode = \"ShowAdAgency\"
background = \"SightBackground1\"

[[scenario-group]]
type = \"choice\"
header_text = \"宣伝の受注もしないとネ\"
id = 24
   [scenario-group.tachie-data]
   right = \"KosuzuTachie1\"

   [[scenario-group.choice-pattern]]
   pattern = \"完了\"
   jump-id = 25

[[scenario-group]]
type = \"wait\"
id = 25
next-id = 26
opecode = \"ShowMain\"
background = \"SightBackground1\"

[[scenario-group]]
type = \"choice\"
background = \"SightBackground1\"
header_text = \"サァ準備ができたわよ\"
id = 26
   [scenario-group.tachie-data]
   right = \"KosuzuTachie1\"

   [[scenario-group.choice-pattern]]
   pattern = \"行動開始\"
   jump-id = 27
   [[scenario-group.choice-pattern]]
   pattern = \"保存\"
   jump-id = 4


[[scenario-group]]
type = \"builtin\"
opecode = \"StartSchedule\"
id = 27
background = \"SightBackground1\"
";

const time_attack_first_day_scenario: &str = "first-scenario-id = 10

[scene-transition]
scenario = 1
dream = 2
save = 4

[[scenario-group]]
type = \"scenario\"
id = 10
next-id = 12
background = \"SightBackground1\"

[scenario-group.tachie-data]
right = \"KosuzuTachie1\"

   [scenario-group.default-text-attribute]
   fpc = 2.0
   font_scale = 32.0
   color = 0x000000ff

   [[scenario-group.text]]
   text = \"\"\"*助言*
熟練モードへようこそ。
熟練モードでは通常モードとは異なり、60日間
働くことになります。特に目標金額は無く、この期限の間
にいくら稼げるかがテーマになります。
かなり長時間かかると思われますので保存を忘れずに
まったりお楽しみください。\"\"\"

[[scenario-group]]
type = \"choice\"
header_text = \"サァ準備ができたわよ\"
id = 11
background = \"SightBackground1\"
   [scenario-group.tachie-data]
   right = \"KosuzuTachie1\"

   [[scenario-group.choice-pattern]]
   pattern = \"行動開始\"
   jump-id = 13
   [[scenario-group.choice-pattern]]
   pattern = \"保存\"
   jump-id = 4

[[scenario-group]]
type = \"choice\"
header_text = \"宣伝の依頼もやらないとネ\"
id = 14
   [scenario-group.tachie-data]
   right = \"KosuzuTachie1\"

   [[scenario-group.choice-pattern]]
   pattern = \"完了\"
   jump-id = 18

[[scenario-group]]
type = \"wait\"
id = 18
next-id = 17
opecode = \"ShowAdAgency\"
background = \"SightBackground1\"

[[scenario-group]]
type = \"choice\"
header_text = \"宣伝の受注もしないとネ\"
id = 17
   [scenario-group.tachie-data]
   right = \"KosuzuTachie1\"

   [[scenario-group.choice-pattern]]
   pattern = \"完了\"
   jump-id = 16

[[scenario-group]]
type = \"wait\"
id = 16
next-id = 11
opecode = \"ShowMain\"
background = \"SightBackground1\"


[[scenario-group]]
type = \"builtin\"
opecode = \"StartSchedule\"
id = 13
background = \"SightBackground1\"

[[scenario-group]]
type = \"wait\"
id = 12
next-id = 20
opecode = \"ShowStatusScreen\"
background = \"SightBackground1\"

[[scenario-group]]
type = \"wait\"
id = 20
next-id = 15
opecode = \"ScheduleCheck\"
background = \"SightBackground1\"

[[scenario-group]]
type = \"wait\"
id = 15
next-id = 14
opecode = \"ShowAd\"
background = \"SightBackground1\"

[scenario-group.tachie-data]
right = \"KosuzuTachie1\"
";

const time_attack_default_day_scenario: &str = "first-scenario-id = 10

[scene-transition]
scenario = 1
dream = 2
save = 4

[[scenario-group]]
type = \"scenario\"
id = 10
next-id = 12
background = \"SightBackground1\"

[scenario-group.tachie-data]
right = \"KosuzuTachie1\"

   [scenario-group.default-text-attribute]
   fpc = 2.0
   font_scale = 32.0
   color = 0x000000ff

   [[scenario-group.text]]
   text = \"サァ、今日も働くわヨ\"

[[scenario-group]]
type = \"choice\"
header_text = \"サァ準備ができたわよ\"
id = 11
background = \"SightBackground1\"
   [scenario-group.tachie-data]
   right = \"KosuzuTachie1\"

   [[scenario-group.choice-pattern]]
   pattern = \"行動開始\"
   jump-id = 13
   [[scenario-group.choice-pattern]]
   pattern = \"保存\"
   jump-id = 4

[[scenario-group]]
type = \"choice\"
header_text = \"宣伝の依頼もやらないとネ\"
id = 14
   [scenario-group.tachie-data]
   right = \"KosuzuTachie1\"

   [[scenario-group.choice-pattern]]
   pattern = \"完了\"
   jump-id = 18

[[scenario-group]]
type = \"wait\"
id = 18
next-id = 17
opecode = \"ShowAdAgency\"
background = \"SightBackground1\"

[[scenario-group]]
type = \"choice\"
header_text = \"宣伝の受注もしないとネ\"
id = 17
   [scenario-group.tachie-data]
   right = \"KosuzuTachie1\"

   [[scenario-group.choice-pattern]]
   pattern = \"完了\"
   jump-id = 16

[[scenario-group]]
type = \"wait\"
id = 16
next-id = 11
opecode = \"ShowMain\"
background = \"SightBackground1\"


[[scenario-group]]
type = \"builtin\"
opecode = \"StartSchedule\"
id = 13
background = \"SightBackground1\"

[[scenario-group]]
type = \"wait\"
id = 12
next-id = 20
opecode = \"ShowStatusScreen\"
background = \"SightBackground1\"

[[scenario-group]]
type = \"wait\"
id = 20
next-id = 14
opecode = \"ShowAd\"
background = \"SightBackground1\"

[scenario-group.tachie-data]
right = \"KosuzuTachie1\"
";

const time_attack_week_first_scenario: &str = "first-scenario-id = 10

[scene-transition]
scenario = 1
dream = 2
save = 4

[[scenario-group]]
type = \"scenario\"
id = 10
next-id = 12
background = \"SightBackground1\"

[scenario-group.tachie-data]
right = \"KosuzuTachie1\"

   [scenario-group.default-text-attribute]
   fpc = 2.0
   font_scale = 32.0
   color = 0x000000ff

   [[scenario-group.text]]
   text = \"サァ、今日も働くわヨ\"

[[scenario-group]]
type = \"choice\"
header_text = \"サァ準備ができたわよ\"
id = 11
background = \"SightBackground1\"
   [scenario-group.tachie-data]
   right = \"KosuzuTachie1\"

   [[scenario-group.choice-pattern]]
   pattern = \"行動開始\"
   jump-id = 13
   [[scenario-group.choice-pattern]]
   pattern = \"保存\"
   jump-id = 4

[[scenario-group]]
type = \"choice\"
header_text = \"宣伝の依頼もやらないとネ\"
id = 14
   [scenario-group.tachie-data]
   right = \"KosuzuTachie1\"

   [[scenario-group.choice-pattern]]
   pattern = \"完了\"
   jump-id = 18

[[scenario-group]]
type = \"wait\"
id = 18
next-id = 17
opecode = \"ShowAdAgency\"
background = \"SightBackground1\"

[[scenario-group]]
type = \"choice\"
header_text = \"宣伝の受注もしないとネ\"
id = 17
   [scenario-group.tachie-data]
   right = \"KosuzuTachie1\"

   [[scenario-group.choice-pattern]]
   pattern = \"完了\"
   jump-id = 16

[[scenario-group]]
type = \"wait\"
id = 16
next-id = 11
opecode = \"ShowMain\"
background = \"SightBackground1\"


[[scenario-group]]
type = \"builtin\"
opecode = \"StartSchedule\"
id = 13
background = \"SightBackground1\"

[[scenario-group]]
type = \"wait\"
id = 12
next-id = 20
opecode = \"ShowStatusScreen\"
background = \"SightBackground1\"

[[scenario-group]]
type = \"wait\"
id = 20
next-id = 15
opecode = \"ScheduleCheck\"
background = \"SightBackground1\"

[[scenario-group]]
type = \"wait\"
id = 15
next-id = 14
opecode = \"ShowAd\"
background = \"SightBackground1\"

[scenario-group.tachie-data]
right = \"KosuzuTachie1\"
";

const no_enough_hp_scenario: &str = "first-scenario-id = 10

[scene-transition]
scenario = 1
dream = 2
save = 4

[[scenario-group]]
type = \"scenario\"
id = 10
next-id = 11
background = \"SightBackground1\"

[scenario-group.tachie-data]
right = \"KosuzuTachie1\"
left = \"KosuzuTachie1\"

   [scenario-group.default-text-attribute]
   fpc = 2.0
   font_scale = 32.0
   color = 0x000000ff

   [[scenario-group.text]]
   text = \"なんか疲れちゃったカモ。今日はお休みにしましょ。\"

[[scenario-group]]
type = \"wait\"
id = 11
next-id = 12
opecode = \"NextDay\"
background = \"SightBackground1\"";
