use std::collections::LinkedList;
use std::collections::VecDeque;

use torifune::graphics::drawable::*;
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::graphics::object::*;
use torifune::numeric;

use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;

use super::*;
use crate::scene::scenario_scene::ScenarioContext;
use crate::core::{FontID, GameResource, SuzuContext, TextureID, TileBatchTextureID};
use crate::object::util_object::*;
use crate::parse_toml_file;
use crate::scene::{SceneID, SceneTransition};
use std::str::FromStr;

pub type ScenarioElementID = i32;
const SCENAIRO_DEFAULT_SHOP_WORK_ID: ScenarioElementID = -1;
const SCENAIRO_DEFAULT_GOINGOUT_WORK_ID: ScenarioElementID = -2;
const SCENAIRO_DEFAULT_REST_WORK_ID: ScenarioElementID = -3;

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
                    game_data.get_font(FontID::DEFAULT),
                    numeric::Vector2f::new(font_scale, font_scale),
                    color,
                ),
            },
        }
    }

    fn slice_text_bytes(&self, begin: usize, end: usize) -> &str {
        unsafe { self.text.as_str().get_unchecked(begin..end) }
    }

    fn end_with_indent(&self) -> bool {
        self.text.chars().last().unwrap() == '\n'
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
    drwob_essential: DrawableObjectEssential,
}

impl ScenarioTachie {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, tid_array: Vec<TextureID>, t: Clock) -> Self {
        // left
        let left_texture = if tid_array.len() > 0 {
            Some(SimpleObject::new(
                MovableUniTexture::new(
                    Box::new(UniTexture::new(
                        ctx.ref_texture(tid_array[0]),
                        numeric::Point2f::new(50.0, 100.0),
                        numeric::Vector2f::new(0.3, 0.3),
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

        let right_texture = if tid_array.len() > 1 {
            Some(SimpleObject::new(
                MovableUniTexture::new(
                    Box::new(UniTexture::new(
                        ctx.ref_texture(tid_array[1]),
                        numeric::Point2f::new(800.0, 50.0),
                        numeric::Vector2f::new(0.3, 0.3),
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
    seq_text: Vec<ScenarioTextSegment>,
    iterator: f32,
    current_segment_index: usize,
    total_length: usize,
    scenario_id: ScenarioElementID,
    next_scenario_id: ScenarioElementID,
    background_texture_id: TextureID,
    tachie_data: Option<Vec<TextureID>>,
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
                game_data.get_font(FontID::DEFAULT),
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

        let background_texture_id =
            TextureID::from_str(toml_scripts.get("background").unwrap().as_str().unwrap()).unwrap();

        let total_length: usize = seq_text.iter().fold(0, |sum, s| sum + s.str_len());

        let tachie_data = if let Some(tachie_table) = toml_scripts.get("tachie-data") {
            let mut tid_vec = Vec::new();
            if let Some(tid) = tachie_table.get("right") {
                tid_vec.push(TextureID::from_str(tid.as_str().unwrap()).unwrap());
            }

            if let Some(tid) = tachie_table.get("left") {
                tid_vec.push(TextureID::from_str(tid.as_str().unwrap()).unwrap());
            }

            Some(tid_vec)
        } else {
            None
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

    pub fn get_background_texture_id(&self) -> TextureID {
        self.background_texture_id
    }

    pub fn get_tachie_data(&self) -> Option<Vec<TextureID>> {
        self.tachie_data.clone()
    }
}

///
/// 選択肢のデータを保持する構造体
///
pub struct ChoicePatternData {
    text: Vec<String>,
    jump_scenario_id: Vec<ScenarioElementID>,
    scenario_id: ScenarioElementID,
    background_texture_id: Option<TextureID>,
    tachie_data: Option<Vec<TextureID>>,
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
            let mut tid_vec = Vec::new();
            if let Some(tid) = tachie_table.get("right") {
                tid_vec.push(TextureID::from_str(tid.as_str().unwrap()).unwrap());
            }

            if let Some(tid) = tachie_table.get("left") {
                tid_vec.push(TextureID::from_str(tid.as_str().unwrap()).unwrap());
            }

            Some(tid_vec)
        } else {
            None
        };

        ChoicePatternData {
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

    pub fn get_tachie_data(&self) -> Option<Vec<TextureID>> {
        self.tachie_data.clone()
    }
}

struct ChoicePanel {
    panel: UniTexture,
}

impl ChoicePanel {
    pub fn new(panel: UniTexture) -> Self {
        ChoicePanel { panel: panel }
    }
}

impl DrawableComponent for ChoicePanel {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        self.panel.draw(ctx)
    }

    fn hide(&mut self) {
        self.panel.hide();
    }

    fn appear(&mut self) {
        self.panel.appear();
    }

    fn is_visible(&self) -> bool {
        self.panel.is_visible()
    }

    fn set_drawing_depth(&mut self, depth: i8) {
        self.panel.set_drawing_depth(depth)
    }

    fn get_drawing_depth(&self) -> i8 {
        self.panel.get_drawing_depth()
    }
}

impl DrawableObject for ChoicePanel {
    impl_drawable_object_for_wrapped! {panel}
}

impl TextureObject for ChoicePanel {
    impl_texture_object_for_wrapped! {panel}
}

pub struct ChoiceBox {
    choice_text: Vec<String>,
    panels: Vec<ChoicePanel>,
    selecting: usize,
    canvas: SubScreen,
}

impl ChoiceBox {
    fn generate_choice_panel<'a>(
        ctx: &mut SuzuContext<'a>,
        size: usize,
        left_top: numeric::Vector2f,
        align: f32,
    ) -> Vec<ChoicePanel> {
        let mut choice_panels = Vec::new();
        let mut panel = TextureID::ChoicePanel1 as u32;
        let mut pos: numeric::Point2f = left_top.into();

        for _ in 0..size {
            choice_panels.push(ChoicePanel::new(UniTexture::new(
                ctx.ref_texture(TextureID::from_u32(panel).unwrap()),
                pos,
                numeric::Vector2f::new(0.5, 0.5),
                0.0,
                0,
            )));
            pos.x += choice_panels
                .last()
                .unwrap()
                .get_drawing_size(ctx.context)
                .x;
            pos.x += align;
            panel += 1;
        }

        choice_panels
    }

    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        pos_rect: numeric::Rect,
        choice_text: Vec<String>,
    ) -> Self {
        let mut panels = Self::generate_choice_panel(
            ctx,
            choice_text.len(),
            numeric::Vector2f::new(10.0, 10.0),
            10.0,
        );

        for panel in &mut panels {
            panel.set_color(ggraphics::Color::from_rgba_u32(0xaaaaaaff));
        }
        panels
            .get_mut(0)
            .unwrap()
            .set_color(ggraphics::Color::from_rgba_u32(0xffffffff));

        ChoiceBox {
            panels: panels,
            choice_text: choice_text,
            selecting: 0,
            canvas: SubScreen::new(ctx.context, pos_rect, 0, ggraphics::Color::from_rgba_u32(0)),
        }
    }

    pub fn get_selecting_index(&self) -> usize {
        self.selecting
    }

    pub fn get_selecting_str(&self) -> &str {
        self.choice_text
            .get(self.get_selecting_index())
            .as_ref()
            .unwrap()
    }

    pub fn move_right(&mut self) {
        if self.choice_text.len() > (self.selecting + 1) {
            self.panels
                .get_mut(self.selecting)
                .unwrap()
                .set_color(ggraphics::Color::from_rgba_u32(0xaaaaaaff));
            self.selecting += 1;
            self.panels
                .get_mut(self.selecting)
                .unwrap()
                .set_color(ggraphics::Color::from_rgba_u32(0xffffffff));
        }
    }

    pub fn move_left(&mut self) {
        if self.selecting > 0 {
            self.panels
                .get_mut(self.selecting)
                .unwrap()
                .set_color(ggraphics::Color::from_rgba_u32(0xaaaaaaff));
            self.selecting -= 1;
            self.panels
                .get_mut(self.selecting)
                .unwrap()
                .set_color(ggraphics::Color::from_rgba_u32(0xffffffff));
        }
    }

    pub fn cursor_select<'a>(&mut self, ctx: &mut SuzuContext<'a>, point: numeric::Point2f) {
	let rpoint = self.canvas.relative_point(point);
	
	for (index, panel) in self.panels.iter_mut().enumerate() {
	    if panel.contains(ctx.context, rpoint) {
		panel.set_color(ggraphics::Color::from_rgba_u32(0xffffffff));
		self.selecting = index;
	    } else {
		panel.set_color(ggraphics::Color::from_rgba_u32(0xaaaaaaff));
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

///
/// 選択肢のデータを保持する構造体
///
pub struct ScenarioFinishAndWaitData {
    scenario_id: ScenarioElementID,
    next_id: ScenarioElementID,
    background_texture_id: Option<TextureID>,
    tachie_data: Option<Vec<TextureID>>,
    opecode: String,
}

impl ScenarioFinishAndWaitData {
    pub fn from_toml_object(toml_scripts: &toml::value::Value, _: &GameResource) -> Self {
        let id = toml_scripts.get("id").unwrap().as_integer().unwrap() as i32;
	let next_id = toml_scripts.get("next-id").unwrap().as_integer().unwrap() as i32;
	let opecode = toml_scripts.get("opecode").unwrap().as_str().unwrap().to_string();

        let background_texture_id = if let Some(background_tid_str) = toml_scripts.get("background")
        {
            Some(TextureID::from_str(background_tid_str.as_str().unwrap()).unwrap())
        } else {
            None
        };

        let tachie_data = if let Some(tachie_table) = toml_scripts.get("tachie-data") {
            let mut tid_vec = Vec::new();
            if let Some(tid) = tachie_table.get("right") {
                tid_vec.push(TextureID::from_str(tid.as_str().unwrap()).unwrap());
            }

            if let Some(tid) = tachie_table.get("left") {
                tid_vec.push(TextureID::from_str(tid.as_str().unwrap()).unwrap());
            }

            Some(tid_vec)
        } else {
            None
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

    pub fn get_tachie_data(&self) -> Option<Vec<TextureID>> {
        self.tachie_data.clone()
    }

    pub fn get_next_id(&self) -> ScenarioElementID {
	self.next_id
    }

    pub fn get_opecode(&self) -> &str {
	self.opecode.as_str()
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
}

impl ScenarioElement {
    pub fn get_scenario_id(&self) -> ScenarioElementID {
        match self {
            Self::Text(text) => text.get_scenario_id(),
            Self::ChoiceSwitch(choice) => choice.get_scenario_id(),
            Self::SceneTransition(transition_data) => transition_data.2,
	    Self::FinishAndWait(data) => data.get_scenario_id(),
        }
    }

    pub fn get_background_texture(&self) -> Option<TextureID> {
        match self {
            Self::Text(text) => Some(text.get_background_texture_id()),
            Self::ChoiceSwitch(choice) => choice.get_background_texture_id(),
            Self::SceneTransition(_) => None,
	    Self::FinishAndWait(data) => data.get_background_texture_id(),
        }
    }

    pub fn get_tachie_info(&self) -> Option<Vec<TextureID>> {
        match self {
            Self::Text(text) => text.get_tachie_data(),
            Self::ChoiceSwitch(choice) => choice.get_tachie_data(),
            Self::SceneTransition(_) => None,
	    Self::FinishAndWait(data) => data.get_tachie_data(),
        }
    }
}

pub struct ScenarioElementPool {
    pool: Vec<ScenarioElement>,
}

impl ScenarioElementPool {
    pub fn new_empty() -> Self {
	ScenarioElementPool {
	    pool: Vec::new(),
	}
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
    element_id_stack: LinkedList<ScenarioElementID>,
    current_page: usize,
}

impl Scenario {
    pub fn new(file_path: &str, game_data: &GameResource) -> Self {
        let mut scenario = ScenarioElementPool::new_empty();

        let root = parse_toml_file!(file_path);

        let first_scenario_id = root["first-scenario-id"].as_integer().unwrap();

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
			    ScenarioFinishAndWaitData::from_toml_object(elem, game_data)
                        ));
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
            element_id_stack: LinkedList::new(),
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
        self.element_id_stack.push_back(scenario_element_id);
        let index = self.scenario.find_index_of_specified_scenario_id(scenario_element_id);
        self.current_page = index;
    }

    pub fn turn_back_scenario_offset(&mut self, offset: usize) {
        for _ in 0..offset {
            self.element_id_stack.pop_back();
        }

        let turn_backed_id = self.element_id_stack.back().unwrap();

        self.current_page = self.scenario.find_index_of_specified_scenario_id(*turn_backed_id);

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
    pub fn go_next_scenario_from_text_scenario(&mut self) {
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

    pub fn release_waiting(&mut self) {
	self.go_next_scenario_from_waiting();
    }
}

#[derive(PartialEq, Clone, Copy)]
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

        TextBox {
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
        }
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
        let mut pos = numeric::Point2f::new(50.0, 50.0);
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

    pub fn set_fixed_text(&mut self, text: &str, font_info: FontInformation) {
        self.text.clear();
        self.text.push_back(SimpleText::new(
            tobj::MovableText::new(
                Box::new(tobj::UniText::new(
                    text.to_string(),
                    numeric::Point2f::new(50.0, 50.0),
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
        self.text_box_status = TextBoxStatus::FixedText;
    }

    pub fn next_button_handler(&mut self) {
        self.head_line_number += 1;
        self.text_box_status = TextBoxStatus::UpdatingText;
    }

    pub fn reset_head_line(&mut self) {
        self.head_line_number = 0;
    }
}

impl DrawableComponent for TextBox {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.background.draw(ctx)?;

            self.appearance_frame.draw(ctx)?;

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
    canvas: SubScreen,
}

impl ScenarioBox {
    pub fn new<'a>(ctx: &mut SuzuContext, rect: numeric::Rect, t: Clock) -> Self {
        let background = tobj::SimpleObject::new(
            tobj::MovableUniTexture::new(
                Box::new(UniTexture::new(
                    ctx.ref_texture(TextureID::TextBackground),
                    numeric::Point2f::new(20.0, 20.0),
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
                numeric::Rect::new(0.0, 0.0, rect.w, rect.h),
                background,
                TileBatchTextureID::TaishoStyle1,
                3,
                t,
            ),
            choice_box: None,
            canvas: SubScreen::new(ctx.context, rect, 0, ggraphics::Color::from_rgba_u32(0x00)),
        }
    }

    pub fn contains(&self, point: numeric::Point2f) -> bool {
	let rpoint = self.canvas.relative_point(point);
	self.text_box.canvas.contains(rpoint)
    }

    pub fn new_choice<'a>(
        ctx: &mut SuzuContext<'a>,
        rect: numeric::Rect,
        choice_pattern: ChoicePatternData,
        font_info: FontInformation,
        t: Clock,
    ) -> Self {
        let background = tobj::SimpleObject::new(
            tobj::MovableUniTexture::new(
                Box::new(UniTexture::new(
                    ctx.ref_texture(TextureID::TextBackground),
                    numeric::Point2f::new(20.0, 20.0),
                    numeric::Vector2f::new(0.8, 0.8),
                    0.0,
                    0,
                )),
                None,
                0,
            ),
            Vec::new(),
        );
        let mut scenario_box = ScenarioBox {
            text_box: TextBox::new(
                ctx,
                numeric::Rect::new(0.0, 0.0, rect.w, rect.h),
                background,
                TileBatchTextureID::TaishoStyle1,
                3,
                t,
            ),
            choice_box: Some(ChoiceBox::new(
                ctx,
                numeric::Rect::new(40.0, 100.0, 1200.0, 150.0),
                choice_pattern.text.clone(),
            )),
            canvas: SubScreen::new(ctx.context, rect, 0, ggraphics::Color::from_rgba_u32(0x00)),
        };
        scenario_box.display_choice_box_text(font_info);

        scenario_box
    }

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
            let selected_text = self
                .choice_box
                .as_ref()
                .unwrap()
                .get_selecting_str()
                .to_string();
            self.text_box.set_fixed_text(&selected_text, font_info);
        }
    }

    pub fn reset_head_line(&mut self) {
        self.text_box.reset_head_line();
    }

    pub fn get_choice_selecting_index(&self) -> Option<usize> {
        if let Some(choice) = self.choice_box.as_ref() {
            Some(choice.get_selecting_index())
        } else {
            None
        }
    }
}

impl DrawableComponent for ScenarioBox {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.canvas.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.text_box.draw(ctx)?;

            if let Some(choice) = self.choice_box.as_mut() {
                choice.draw(ctx)?;
            }

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

#[derive(PartialEq, Clone, Copy)]
pub enum ScenarioEventStatus {
    Scenario = 0,
    Choice,
    SceneTransition,
    FinishAndWait,
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
}

impl ScenarioEvent {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        rect: numeric::Rect,
        file_path: &str,
        t: Clock,
    ) -> Self {
        let scenario = Scenario::new(file_path, ctx.resource);

        let event_background = if let Some(mut texture) =
            Self::update_event_background_sub(ctx, scenario.ref_current_element())
        {
            texture.fit_scale(ctx.context, numeric::Vector2f::new(rect.w, rect.h));
            Some(texture)
        } else {
            None
        };

        let event_tachie = Self::update_event_tachie_sub(ctx, scenario.ref_current_element(), t);

        let appr_frame = TileBatchFrame::new(
            ctx.resource,
            TileBatchTextureID::BlackFrame,
            numeric::Rect::new(0.0, 0.0, rect.w, rect.h),
            numeric::Vector2f::new(1.0, 1.0),
            0,
        );

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
        }
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
        }
    }

    pub fn update_event_tachie_sub<'a>(
        ctx: &mut SuzuContext<'a>,
        scenario_element: &ScenarioElement,
        t: Clock,
    ) -> Option<ScenarioTachie> {
        // ScenarioEventの立ち絵データ取り出し
        // ScenarioElementが立ち絵情報を持っていれば、取り出す
        if let Some(tachie_vec) = scenario_element.get_tachie_info() {
            Some(ScenarioTachie::new(ctx, tachie_vec, t))
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
        }
    }

    pub fn scenario_control_mut(&mut self) -> &mut Scenario {
        &mut self.scenario
    }

    ///
    /// 表示しているテキストや選択肢を更新するメソッド
    ///
    pub fn update_text<'a>(&mut self, ctx: &mut SuzuContext<'a>, scno_ctx: &mut ScenarioContext) {
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
                }
            }
            ScenarioElement::ChoiceSwitch(choice_pattern) => {
                // ChoiceBoxが表示されていない場合、新しくオブジェクトを生成する
                if !self.scenario_box.is_enable_choice_box() {
                    self.scenario_box.insert_choice_box(Some(ChoiceBox::new(
                        ctx,
                        numeric::Rect::new(40.0, 100.0, 1200.0, 150.0),
                        choice_pattern.text.clone(),
                    )));

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
                }
            }
            ScenarioElement::SceneTransition(transition_data) => {
                // SceneTransition状態に移行し、移行先を決定する
                self.status = ScenarioEventStatus::SceneTransition;
                self.transition_scene = Some(transition_data.0);
                self.transition_type = Some(transition_data.1);

                // 再描画要求
                ctx.process_utility.redraw();
            }
	    ScenarioElement::FinishAndWait(_) => {
		self.status = ScenarioEventStatus::FinishAndWait;
		scno_ctx.scenario_is_finish_and_wait = true;
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

    pub fn release_scenario_waiting(&mut self) {
	self.scenario.release_waiting();
    }

    pub fn set_fixed_text_to_scenario_box<'a>(&mut self, ctx: &mut SuzuContext<'a>, text: &str) {
	self.scenario_box
	    .text_box
	    .set_fixed_text(
		text,
		FontInformation::new(
		    ctx.resource.get_font(FontID::Cinema),
		    numeric::Vector2f::new(32.0, 32.0),
		    ggraphics::BLACK
		),
	    );
    }

    ///
    /// Action1キーが押されたときの、ScenarioEventの挙動
    ///
    pub fn key_down_action1<'a>(&mut self, ctx: &mut SuzuContext<'a>, _: Clock) {
        match self.scenario.ref_current_element_mut() {
            // 現在のScenarioElementがテキスト
            ScenarioElement::Text(scenario_text) => {
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
                }
            }
            ScenarioElement::ChoiceSwitch(_) => {
                self.scenario.go_next_scenario_from_choice_scenario(
                    self.scenario_box.get_choice_selecting_index().unwrap(),
                );
                self.update_event_background(ctx);
                self.update_event_tachie(ctx, 0);

                // choice_boxは消す
                self.scenario_box.insert_choice_box(None);
            }
            ScenarioElement::SceneTransition(_) => (),
	    ScenarioElement::FinishAndWait(_) => (),
        }
    }

    pub fn mouse_motion_handler<'a>(&mut self, ctx: &mut SuzuContext<'a>, point: numeric::Point2f) {
	let rpoint = self.canvas.relative_point(point);

	match self.scenario.ref_current_element_mut() {
            ScenarioElement::ChoiceSwitch(_) => {
		if let Some(choice) = self.scenario_box.choice_box.as_mut() {
		    let rpoint = self.scenario_box.canvas.relative_point(rpoint);
		    choice.cursor_select(ctx, rpoint);

		    self.scenario_box
			.display_choice_box_text(FontInformation::new(
			    ctx.resource.get_font(FontID::Cinema),
			    numeric::Vector2f::new(32.0, 32.0),
			    ggraphics::Color::from_rgba_u32(0x000000ff),
			));
		}
            },
	    _ => (),
        }
    }

    ///
    /// Rightキーが押されたときの、ScenarioEventの挙動
    ///
    pub fn key_down_right<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        if let Some(choice) = self.scenario_box.choice_box.as_mut() {
            choice.move_right();
            self.scenario_box
                .display_choice_box_text(FontInformation::new(
                    ctx.resource.get_font(FontID::Cinema),
                    numeric::Vector2f::new(32.0, 32.0),
                    ggraphics::Color::from_rgba_u32(0x000000ff),
                ));
        }
    }

    ///
    /// Leftキーが押されたときの、ScenarioEventの挙動
    ///
    pub fn key_down_left<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        if let Some(choice) = self.scenario_box.choice_box.as_mut() {
            choice.move_left();
            self.scenario_box
                .display_choice_box_text(FontInformation::new(
                    ctx.resource.get_font(FontID::Cinema),
                    numeric::Vector2f::new(32.0, 32.0),
                    ggraphics::Color::from_rgba_u32(0x000000ff),
                ));
        }
    }
}

impl DrawableComponent for ScenarioEvent {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
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
