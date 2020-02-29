use std::rc::Rc;
use std::cell::RefCell;

use torifune::device as tdev;
use torifune::graphics::object::*;

use tdev::{VirtualKey, KeyboardEvent};
use torifune::core::Clock;
use torifune::graphics::*;
use torifune::core::Updatable;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::graphics::object::VerticalText;
use torifune::graphics::object::menu;
use ggez::input as ginput;
use ggez::graphics as ggraphics;

use ginput::mouse::MouseButton;
use torifune::numeric;
use torifune::debug;

use crate::core::{GameData, FontID};
use super::*;
use crate::object::*;
use crate::object::map_object::*;
use crate::core::map_parser as mp;
use crate::object::map_object::EventTrigger;
use crate::object::scenario::*;
use crate::object::move_fn;
use crate::scene::suzuna_scene::TaskResult;

use number_to_jk::number_to_jk;

struct CharacterGroup {
    group: Vec<GeneralCharacter>,
    drwob_essential: DrawableObjectEssential,
}

impl CharacterGroup {
    pub fn new() -> Self {
        CharacterGroup {
            group: Vec::new(),
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }

    #[inline(always)]
    pub fn add(&mut self, character: GeneralCharacter) {
        self.group.push(character);
    }

    #[inline(always)]
    pub fn remove_if<F>(&mut self, f: F)
    where F: Fn(&GeneralCharacter) -> bool {
        self.group.retain(|e| !f(e));
    }

    pub fn len(&self) -> usize {
        self.group.len()
    }

    pub fn move_and_collision_check(&mut self, ctx: &mut ggez::Context, camera: &numeric::Rect,
                           tile_map: &mp::StageObjectMap, t: Clock) {
        self.group
            .iter_mut()
            .for_each(|character| {
                character.move_map_current_speed_y();

                // 当たり判定の前に描画位置を決定しないとバグる。この仕様も直すべき
                character.get_mut_character_object().update_display_position(camera);
                
                ShopScene::check_collision_vertical(ctx, character.get_mut_character_object(), tile_map, t);
                character.get_mut_character_object().update_display_position(camera);
                

                character.move_map_current_speed_x();
                character.get_mut_character_object().update_display_position(camera);
                ShopScene::check_collision_horizon(ctx, character.get_mut_character_object(), tile_map, t);
                character.get_mut_character_object().update_display_position(camera);
            });
    }

    pub fn iter(&self) -> std::slice::Iter<GeneralCharacter> {
        self.group.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<GeneralCharacter> {
        self.group.iter_mut()
    }
}

impl DrawableComponent for CharacterGroup {
    #[inline(always)]
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        self.group.iter_mut().for_each(|e| { e.get_mut_character_object().obj_mut().draw(ctx).unwrap(); });
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

///
/// ## マップ上のデータをまとめる構造体
///
/// ### tile_map
/// tilesetで構成された描画可能なマップ
///
/// ### event_map
/// マップ上のイベントをまとめておく構造体
///
/// ### scenario_box
/// マップ上に表示されるテキストボックス
///
struct MapData {
    pub tile_map: mp::StageObjectMap,
    pub event_map: MapEventList,
    pub scenario_box: Option<ScenarioBox>,
}

impl MapData {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData, map_id: u32, camera: Rc<RefCell<numeric::Rect>>) -> Self {
	let map_constract_data = game_data.get_map_data(map_id).unwrap();
	
	MapData {
	    tile_map: mp::StageObjectMap::new(ctx,
					      &map_constract_data.map_file_path,
					      camera.clone(), numeric::Rect::new(0.0, 0.0, 1366.0, 768.0),
					      numeric::Vector2f::new(6.0, 6.0)),
	    event_map: MapEventList::from_file(&map_constract_data.event_map_file_path),
	    scenario_box: None,
	}
    }

    pub fn check_event_panel(&mut self, trigger: EventTrigger,
			     point: numeric::Point2f, _t: Clock) -> Option<&MapEventElement> {
	let tile_size = self.tile_map.get_tile_drawing_size();
	self.event_map.check_event(
	    trigger,
	    numeric::Point2i::new(
		(point.x as f32 / tile_size.x) as i32,
		(point.y as f32 / tile_size.y) as i32,
	    ))
    }
}


///
/// メニューに表示するやつ
///
pub struct ShelvingDetailContents {
    canvas: MovableWrap<SubScreen>,
    menu_rect: numeric::Rect,
    title: VerticalText,
    book_info: Vec<VerticalText>,
}

impl ShelvingDetailContents {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData,
	       menu_rect: numeric::Rect, t: Clock) -> Self {
	let title = VerticalText::new(
	    "配架中".to_string(),
	    numeric::Point2f::new(200.0, 30.0),
	    numeric::Vector2f::new(1.0, 1.0),
	    0.0,
	    0,
	    FontInformation::new(
		game_data.get_font(FontID::JpFude1),
		numeric::Vector2f::new(30.0, 30.0),
		ggraphics::Color::from_rgba_u32(0xff)
	    )
	);

	ShelvingDetailContents {
	    canvas: MovableWrap::new(
		Box::new(SubScreen::new(ctx, menu_rect,
					0, ggraphics::Color::from_rgba_u32(0xffffffff))),
		None,
		t),
	    menu_rect: menu_rect,
	    title: title,
	    book_info: Vec::new(),
	}
    }

    pub fn update_contents(&mut self, ctx: &mut ggez::Context, game_data: &GameData, task_result: &TaskResult) {
	self.book_info.clear();

	let mut book_info_position = numeric::Point2f::new(self.canvas.get_drawing_size(ctx).x - 20.0, 60.0);
	let book_font_information = FontInformation::new(game_data.get_font(FontID::JpFude1),
							 numeric::Vector2f::new(24.0, 24.0),
							 ggraphics::Color::from_rgba_u32(0xff));
	
	self.book_info = task_result.not_shelved_books.iter()
	    .map(|book_info| {
		let vtext = VerticalText::new(
		    format!("{}\t\t{}", book_info.billing_number, &book_info.name),
		    book_info_position,
		    numeric::Vector2f::new(1.0, 1.0),
		    0.0,
		    0,
		    book_font_information.clone()
		);
		
		book_info_position.x -= 30.0;
		
		vtext
	    })
    	    .collect();
    }
    
    pub fn slide_appear(&mut self, slide_position: numeric::Point2f, t: Clock) {
	debug::debug_screen_push_text("fix appearing point @ shop_scene.rs ShelvingDetailContents::slide_appear");
	self.canvas.override_move_func(
	    move_fn::devide_distance(slide_position, 0.5),
	    t);
    }

    pub fn slide_hide(&mut self, t: Clock) {
	debug::debug_screen_push_text("fix appearing point @ shop_scene.rs ShelvingDetailContents::slide_appear");
	self.canvas.override_move_func(
	    move_fn::devide_distance(numeric::Point2f::new(-self.menu_rect.w, 0.0), 0.2),
	    t);
    }

}

impl Updatable for ShelvingDetailContents {
    fn update(&mut self, _: &ggez::Context, t: Clock) {
	self.canvas.move_with_func(t);
    }
}

impl DrawableComponent for ShelvingDetailContents {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
	if self.is_visible() {
	    sub_screen::stack_screen(ctx, self.canvas.ref_wrapped_object());

	    self.title.draw(ctx)?;
	    for vtext in &mut self.book_info {
		vtext.draw(ctx)?;
	    }

	    sub_screen::pop_screen(ctx);
	    self.canvas.draw(ctx)?;
	}
	Ok(())
    }

    #[inline(always)]
    fn hide(&mut self) {
	self.canvas.hide();
    }

    #[inline(always)]
    fn appear(&mut self) {
	self.canvas.appear();
    }

    #[inline(always)]
    fn is_visible(&self) -> bool {
	self.canvas.is_visible()
    }

    #[inline(always)]
    fn set_drawing_depth(&mut self, depth: i8) {
	self.canvas.set_drawing_depth(depth);
    }
    
    #[inline(always)]
    fn get_drawing_depth(&self) -> i8 {
	self.canvas.get_drawing_depth()
    }
}

///
/// メニューに表示するやつ
///
pub struct ShopMenuContents {
    day_text: VerticalText,
    copy_request: VerticalText,
    copy_request_num: VerticalText,
    wait_for_return: VerticalText,
    wait_for_return_num: VerticalText,
    not_shelved: VerticalText,
    not_shelved_num: VerticalText,
    kosuzu_level: VerticalText,
    kosuzu_level_num: VerticalText,
    drwob_essential: DrawableObjectEssential,
}

impl ShopMenuContents {
    pub fn new(game_data: &GameData) -> Self {
	let normal_scale_font = FontInformation::new(game_data.get_font(FontID::JpFude1),
						     numeric::Vector2f::new(26.0, 26.0),
						     ggraphics::Color::from_rgba_u32(0x000000ff));
	
	let large_scale_font = FontInformation::new(game_data.get_font(FontID::JpFude1),
						    numeric::Vector2f::new(30.0, 30.0),
						    ggraphics::Color::from_rgba_u32(0x000000ff));

	ShopMenuContents {
	    day_text: VerticalText::new(
		format!("日付　{}月 {}日", number_to_jk(12), number_to_jk(12)),
		numeric::Point2f::new(370.0, 50.0),
		numeric::Vector2f::new(1.0, 1.0),
		0.0,
		0,
		large_scale_font),
	    copy_request: VerticalText::new(
		format!("写本受注数"),
		numeric::Point2f::new(300.0, 50.0),
		numeric::Vector2f::new(1.0, 1.0),
		0.0,
		0,
		normal_scale_font),
	    copy_request_num: VerticalText::new(
		format!("{}件", number_to_jk(0)),
		numeric::Point2f::new(260.0, 150.0),
		numeric::Vector2f::new(1.0, 1.0),
		0.0,
		0,
		large_scale_font),
	    wait_for_return: VerticalText::new(
		format!("返却待冊数"),
		numeric::Point2f::new(200.0, 50.0),
		numeric::Vector2f::new(1.0, 1.0),
		0.0,
		0,
		normal_scale_font),
	    wait_for_return_num: VerticalText::new(
		format!("{}冊", number_to_jk(0)),
		numeric::Point2f::new(160.0, 150.0),
		numeric::Vector2f::new(1.0, 1.0),
		0.0,
		0,
		large_scale_font),
	    not_shelved: VerticalText::new(
		format!("未配架冊数"),
		numeric::Point2f::new(100.0, 50.0),
		numeric::Vector2f::new(1.0, 1.0),
		0.0,
		0,
		normal_scale_font),
	    not_shelved_num: VerticalText::new(
		format!("{}冊", number_to_jk(0)),
		numeric::Point2f::new(60.0, 150.0),
		numeric::Vector2f::new(1.0, 1.0),
		0.0,
		0,
		large_scale_font),
	    kosuzu_level: VerticalText::new(
		format!("小鈴 習熟度"),
		numeric::Point2f::new(300.0, 350.0),
		numeric::Vector2f::new(1.0, 1.0),
		0.0,
		0,
		normal_scale_font),
	    kosuzu_level_num: VerticalText::new(
		format!("{}", number_to_jk(0)),
		numeric::Point2f::new(260.0, 450.0),
		numeric::Vector2f::new(1.0, 1.0),
		0.0,
		0,
		large_scale_font),
	    drwob_essential: DrawableObjectEssential::new(true, 0),
	}
    }

    pub fn update_contents(&mut self, game_data: &GameData, task_result: &TaskResult) {
	let _normal_scale_font = FontInformation::new(game_data.get_font(FontID::JpFude1),
						     numeric::Vector2f::new(26.0, 26.0),
						     ggraphics::Color::from_rgba_u32(0x000000ff));
	
	let large_scale_font = FontInformation::new(game_data.get_font(FontID::JpFude1),
						    numeric::Vector2f::new(30.0, 30.0),
						    ggraphics::Color::from_rgba_u32(0x000000ff));
	
	self.day_text = VerticalText::new(
	    format!("日付　{}月 {}日", number_to_jk(12), number_to_jk(12)),
	    numeric::Point2f::new(370.0, 50.0),
	    numeric::Vector2f::new(1.0, 1.0),
	    0.0,
	    0,
	    large_scale_font);
	
	self.copy_request_num = VerticalText::new(
	    format!("{}件", number_to_jk(task_result.remain_copy_request.len() as u64)),
	    numeric::Point2f::new(260.0, 150.0),
	    numeric::Vector2f::new(1.0, 1.0),
	    0.0,
	    0,
	    large_scale_font);
	
	self.wait_for_return_num = VerticalText::new(
	    format!("{}冊", number_to_jk(task_result.borrowing_books.len() as u64)),
	    numeric::Point2f::new(160.0, 150.0),
	    numeric::Vector2f::new(1.0, 1.0),
	    0.0,
	    0,
	    large_scale_font);
	
	self.not_shelved_num = VerticalText::new(
	    format!("{}冊", number_to_jk(task_result.not_shelved_books.len() as u64)),
	    numeric::Point2f::new(60.0, 150.0),
	    numeric::Vector2f::new(1.0, 1.0),
	    0.0,
	    0,
	    large_scale_font);
	
	self.kosuzu_level_num = VerticalText::new(
	    format!("{}", number_to_jk((task_result.done_works / 3) as u64)),
	    numeric::Point2f::new(260.0, 450.0),
	    numeric::Vector2f::new(1.0, 1.0),
	    0.0,
	    0,
	    large_scale_font);
    }
}

impl DrawableComponent for ShopMenuContents {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
	if self.is_visible() {
	    self.day_text.draw(ctx).unwrap();
	    
	    self.copy_request.draw(ctx).unwrap();
	    self.copy_request_num.draw(ctx).unwrap();

	    self.wait_for_return.draw(ctx).unwrap();
	    self.wait_for_return_num.draw(ctx).unwrap();

	    self.not_shelved.draw(ctx).unwrap();
	    self.not_shelved_num.draw(ctx).unwrap();

	    self.kosuzu_level.draw(ctx).unwrap();
	    self.kosuzu_level_num.draw(ctx).unwrap();
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

pub struct ShopMenu {
    canvas: MovableWrap<SubScreen>,
    menu_contents: ShopMenuContents,
    menu_canvas_size: numeric::Vector2f,
    now_appear: bool,
}

impl ShopMenu {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData, size: numeric::Vector2f, t: Clock) -> Self {
	ShopMenu {
	    canvas: MovableWrap::new(
		Box::new(SubScreen::new(ctx, numeric::Rect::new(-size.x, 0.0, size.x, size.y),
					0, ggraphics::Color::from_rgba_u32(0xffffffff))),
		None,
		t),
	    menu_contents: ShopMenuContents::new(game_data),
	    menu_canvas_size: size,
	    now_appear: false,
	}
    }

    pub fn slide_toggle(&mut self, t: Clock) {
	if self.now_appear {
	    self.canvas.override_move_func(
		move_fn::devide_distance(numeric::Point2f::new(-self.menu_canvas_size.x, 0.0), 0.5),
		t);
	    self.now_appear = false;
	} else {
	    self.canvas.override_move_func(
		move_fn::devide_distance(numeric::Point2f::new(0.0, 0.0), 0.5),
		t);
	    self.now_appear = true;
	}
    }

    pub fn appearing_now(&self) -> bool {
	self.now_appear
    }

    pub fn update_menu_contents(&mut self, game_data: &GameData, task_result: &TaskResult) {
	self.menu_contents.update_contents(game_data, task_result);
    }
}

impl Updatable for ShopMenu {
    fn update(&mut self, _: &ggez::Context, t: Clock) {
	self.canvas.move_with_func(t);
    }
}

impl DrawableComponent for ShopMenu {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
	sub_screen::stack_screen(ctx, self.canvas.ref_wrapped_object());

	self.menu_contents.draw(ctx).unwrap();
	
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

#[derive(PartialEq, Clone, Copy)]
pub enum ShopDetailMenuSymbol {
    ShelvingBooks = 0,
    SuzunaMap,
    None,
}

pub struct ShopDetailMenuContents {
    shelving_info: ShelvingDetailContents,
    drwob_essential: DrawableObjectEssential,
    contents_switch: ShopDetailMenuSymbol,
    appear_position: numeric::Point2f,
    now_appear: bool,
}

impl ShopDetailMenuContents {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData, appear_position: numeric::Point2f,
	       shelving_rect: numeric::Rect, t: Clock) -> Self {
	ShopDetailMenuContents {
	    shelving_info: ShelvingDetailContents::new(ctx, game_data, shelving_rect, t),
	    drwob_essential: DrawableObjectEssential::new(true, 0),
	    contents_switch: ShopDetailMenuSymbol::None,
	    appear_position: appear_position,
	    now_appear: false,
	}
    }

    pub fn update_contents(&mut self) {
	debug::debug_screen_push_text("No Implementation (STUB)");
    }

    pub fn detail_menu_is_open(&self) -> bool {
	self.now_appear
    }

    
    pub fn slide_toggle(&mut self, t: Clock) {
	match self.contents_switch {
	    ShopDetailMenuSymbol::ShelvingBooks => {
		if self.now_appear {
		    self.now_appear = false;
		    self.shelving_info.slide_hide(t);
		} else {
		    self.now_appear = true;
		    self.shelving_info.slide_appear(self.appear_position, t);
		}
	    },
	    ShopDetailMenuSymbol::SuzunaMap => {
		// まだ
	    },
	    ShopDetailMenuSymbol::None => (),
	}
    }

    pub fn set_slide_contents(&mut self, contents_switch: ShopDetailMenuSymbol) {
	self.contents_switch = contents_switch;
    }
}

impl Updatable for ShopDetailMenuContents {
    fn update(&mut self, ctx: &ggez::Context, t: Clock) {
	self.shelving_info.update(ctx, t);
    }
}

impl DrawableComponent for ShopDetailMenuContents {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
	if self.is_visible() {
	    self.shelving_info.draw(ctx)?;
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

pub struct ShopMenuMaster {
    first_menu: ShopMenu,
    detail_menu: ShopDetailMenuContents,
    canvas: SubScreen,
}

impl ShopMenuMaster {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData,
	       first_menu_size: numeric::Vector2f, t: Clock) -> Self {
	
	ShopMenuMaster {
	    first_menu: ShopMenu::new(ctx, game_data, first_menu_size, t),
	    detail_menu: ShopDetailMenuContents::new(ctx, game_data, numeric::Point2f::new(first_menu_size.x, 0.0),
						     numeric::Rect::new(-450.0, 0.0, 450.0, 768.0), t),
	    canvas: SubScreen::new(ctx, numeric::Rect::new(0.0, 0.0, 1366.0, 768.0), 0, ggraphics::Color::from_rgba_u32(0)),
	}
    }

    pub fn update_contents(&mut self, game_data: &GameData, task_result: &TaskResult) {
	self.first_menu.update_menu_contents(game_data, task_result);
	self.detail_menu.update_contents();
    }

    pub fn toggle_first_menu(&mut self, t: Clock) {
	self.first_menu.slide_toggle(t);
    }

    pub fn first_menu_is_open(&self) -> bool {
	self.first_menu.appearing_now()
    }

    pub fn menu_key_action(&mut self, _: &mut ggez::Context, _: &GameData, vkey: VirtualKey, t: Clock) {
	match vkey {
	    VirtualKey::Action3 => {
		if self.first_menu_is_open() {
		    debug::debug_screen_push_text("slide detail menu");
		    self.detail_menu.set_slide_contents(ShopDetailMenuSymbol::ShelvingBooks);
		    self.detail_menu.slide_toggle(t);
		}
	    }
	    _ => (),
	}
    }
}

impl Updatable for ShopMenuMaster {
    fn update(&mut self, ctx: &ggez::Context, t: Clock) {
	self.first_menu.update(ctx, t);
	self.detail_menu.update(ctx, t);
    }
}

impl DrawableComponent for ShopMenuMaster {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
	if self.is_visible() {
	    sub_screen::stack_screen(ctx, &self.canvas);

	    self.detail_menu.draw(ctx)?;
	    self.first_menu.draw(ctx)?;
	    
	    sub_screen::pop_screen(ctx);
            self.canvas.draw(ctx)?;
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
/// # 夢の中のステージ
///
/// ## フィールド
/// ### player
/// プレイキャラ
///
/// ### key_listener
/// キー監視用
///
/// ### map_event_lsit
/// マップ上のイベント
///
/// ### clock
/// 基準クロック
///
/// ### tile_map
/// マップ情報
///
/// ### camera
/// マップを覗くカメラ
///
pub struct ShopScene {
    player: PlayableCharacter,
    character_group: CharacterGroup,
    drawable_component_list: Vec<Box<dyn DrawableComponent>>,
    key_listener: tdev::KeyboardListener,
    clock: Clock,
    map: MapData,
    shop_menu: ShopMenuMaster,
    camera: Rc<RefCell<numeric::Rect>>,
    dark_effect_panel: DarkEffectPanel,
    transition_status: SceneTransition,
    transition_scene: SceneID,
}

impl ShopScene {
    
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData, map_id: u32) -> ShopScene {

        let key_listener = tdev::KeyboardListener::new_masked(vec![tdev::KeyInputDevice::GenericKeyboard],
                                                                  vec![]);

        let camera = Rc::new(RefCell::new(numeric::Rect::new(0.0, 0.0, 1366.0, 768.0)));

        let map_position = numeric::Point2f::new(800.0, 190.0);
        
        let player = PlayableCharacter::new(
            character_factory::create_character(character_factory::CharacterFactoryOrder::PlayableDoremy1,
                                                game_data,
                                                &camera.borrow(),
                                                map_position),
            PlayerStatus { hp: 10, mp: 10.0 });
        
        let character_group = CharacterGroup::new();

        ShopScene {
            player: player,
            character_group: character_group,
	    drawable_component_list: Vec::new(),
            key_listener: key_listener,
            clock: 0,
	    map: MapData::new(ctx, game_data, map_id, camera.clone()),
	    shop_menu: ShopMenuMaster::new(ctx, game_data, numeric::Vector2f::new(450.0, 768.0), 0),
	    dark_effect_panel: DarkEffectPanel::new(ctx, numeric::Rect::new(0.0, 0.0, 1366.0, 768.0), 0),
            camera: camera,
	    transition_scene: SceneID::SuzunaShop,
	    transition_status: SceneTransition::Keep,
        }
    }

    ///
    /// キー入力のイベントハンドラ
    ///
    fn check_key_event(&mut self, ctx: &ggez::Context) {
	if self.map.scenario_box.is_none() {
	    self.player.reset_speed();
            if self.key_listener.current_key_status(ctx, &VirtualKey::Right) == tdev::KeyStatus::Pressed {
		self.right_key_handler(ctx);
            }

            if self.key_listener.current_key_status(ctx, &VirtualKey::Left) == tdev::KeyStatus::Pressed {
		self.left_key_handler(ctx);
            }
	    
            if self.key_listener.current_key_status(ctx, &VirtualKey::Up) == tdev::KeyStatus::Pressed {
		self.up_key_handler(ctx);
            }
	    
	    if self.key_listener.current_key_status(ctx, &VirtualKey::Down) == tdev::KeyStatus::Pressed {
		self.down_key_handler(ctx);
            }
	}
    }

    ///
    /// カメラを動かすメソッド
    ///
    pub fn move_camera(&mut self, offset: numeric::Vector2f) {
        self.camera.borrow_mut().x += offset.x;
        self.camera.borrow_mut().y += offset.y;
    }

    pub fn set_camera_x(&mut self, offset: f32) {
        self.camera.borrow_mut().x = offset;
    }

    pub fn set_camera_y(&mut self, offset: f32) {
        self.camera.borrow_mut().y = offset;
    }

    pub fn set_camera(&mut self, offset: numeric::Vector2f) {
        self.camera.borrow_mut().x = offset.x;
        self.camera.borrow_mut().y = offset.y;
    }
    
    fn right_key_handler(&mut self, _ctx: &ggez::Context) {
	self.player.set_speed_x(4.0);
    }

    fn left_key_handler(&mut self, _ctx: &ggez::Context) {
	self.player.set_speed_x(-4.0);
    }

    fn up_key_handler(&mut self, _ctx: &ggez::Context) {
	self.player.set_speed_y(-4.0);
    }

    fn down_key_handler(&mut self, _ctx: &ggez::Context) {
	self.player.set_speed_y(4.0);
    }

    fn fix_camera_position(&self) -> numeric::Point2f {
        numeric::Point2f::new(
	    if self.player.get_map_position().x >= 800.0 {
		self.player.get_character_object().obj().get_position().x
	    } else if self.player.get_map_position().x >= 650.0 {
		650.0
	    } else {
		self.player.get_map_position().x
	    },
	    if self.player.get_map_position().y >= 1130.0 {
		self.player.get_character_object().obj().get_position().y
	    } else if self.player.get_map_position().y >= 400.0 {
		400.0
	    } else {
		self.player.get_map_position().y
	    })
    }

    ///
    /// マップオブジェクトとの衝突を調べるためのメソッド
    ///
    fn check_collision_horizon(ctx: &mut ggez::Context, character: &mut MapObject, tile_map: &mp::StageObjectMap, t: Clock) {
        let collision_info = tile_map.check_character_collision(ctx, character);

        // 衝突していたか？
        if  collision_info.collision {
            // 修正動作
            let diff = character.fix_collision_horizon(ctx, &collision_info, t);
            character.move_map(numeric::Vector2f::new(diff, 0.0));
        }
    }

    ///
    /// マップオブジェクトとの衝突を調べるためのメソッド
    ///
    fn check_collision_vertical(ctx: &mut ggez::Context, character: &mut MapObject, tile_map: &mp::StageObjectMap, t: Clock) {
        let collision_info = tile_map.check_character_collision(ctx, character);

        // 衝突していたか？
        if  collision_info.collision  {
            // 修正動作
            let diff = character.fix_collision_vertical(ctx, &collision_info, t);
            character.move_map(numeric::Vector2f::new(0.0, diff));
        }
    }

    ///
    /// PlayerCharacterのマップチップとのX方向の衝突を修正する
    ///
    fn playable_check_collision_horizon(&mut self, ctx: &mut ggez::Context) {
        
        let t = self.get_current_clock();

	// プレイヤーのマップチップに対する衝突を修正
        Self::check_collision_horizon(ctx, self.player.get_mut_character_object(), &self.map.tile_map, t);

	// プレイヤーのマップ座標を更新
        self.player.get_mut_character_object().update_display_position(&self.camera.borrow());

	// カメラをプレイヤーにフォーカス
	self.focus_camera_playable_character_x();
        
    }

    ///
    /// PlayerCharacterのマップチップとのY方向の衝突を修正する
    ///
    fn playable_check_collision_vertical(&mut self, ctx: &mut ggez::Context) {
        let t = self.get_current_clock();
        
	// プレイヤーのマップチップに対する衝突を修正
        Self::check_collision_vertical(ctx, self.player.get_mut_character_object(), &self.map.tile_map, t);

	// プレイヤーのマップ座標を更新
        self.player.get_mut_character_object().update_display_position(&self.camera.borrow());

	// カメラをプレイヤーにフォーカス
	self.focus_camera_playable_character_y();
    }

    fn focus_camera_playable_character_x(&mut self) {
	// カメラとプレイヤーキャラクターの差分を計算し、プレイヤーが中心に来るようにカメラを移動
        let player_diff = self.player.get_mut_character_object().obj().get_position() - self.fix_camera_position();
        self.move_camera(numeric::Vector2f::new(player_diff.x, 0.0));
    }

    fn focus_camera_playable_character_y(&mut self) {
	// カメラとプレイヤーキャラクターの差分を計算し、プレイヤーが中心に来るようにカメラを移動
        let player_diff = self.player.get_mut_character_object().obj().get_position() - self.fix_camera_position();
        self.move_camera(numeric::Vector2f::new(0.0, player_diff.y));
    }

    ///
    /// PlayerCharacterの他キャラクターとのX方向の衝突を修正する
    ///
    fn check_character_collision_x(&mut self, ctx: &mut ggez::Context, t: Clock) {
	// マップ座標を更新
        self.player.get_mut_character_object().update_display_position(&self.camera.borrow());

	// 他キャラクターすべてとの衝突判定を行う
        for e in self.character_group.iter_mut() {
	    // 他キャラクターのマップ座標を更新
            e.get_mut_character_object().update_display_position(&self.camera.borrow());

	    // 衝突情報を取得
            let collision_info = self.player.get_character_object().check_collision_with_character(ctx, e.get_character_object());

	    // collisionフィールドがtrueなら、衝突している
            if collision_info.collision {
		// プレイヤーと他キャラクターの衝突状況から、プレイヤーがどれだけ、突き放されればいいのか計算
                let diff = self.player.get_mut_character_object().fix_collision_horizon(ctx, &collision_info, t);
		
		// プレイヤーの突き放し距離分動かす
                self.player.get_mut_character_object().move_map(numeric::Vector2f::new(-diff, 0.0));

		// プレイヤーのマップ座標を更新
		self.player.get_mut_character_object().update_display_position(&self.camera.borrow());
            }
        }

	// カメラをプレイヤーに合わせる
        self.focus_camera_playable_character_x();
    }

    ///
    /// PlayerCharacterの他キャラクターとのY方向の衝突を修正する
    ///
    fn check_character_collision_y(&mut self, ctx: &mut ggez::Context, t: Clock) {
	// マップ座標を更新
        self.player.get_mut_character_object().update_display_position(&self.camera.borrow());

	// 他キャラクターすべてとの衝突判定を行う
        for e in self.character_group.iter_mut() {

	    // 他キャラクターのマップ座標を更新
            e.get_mut_character_object().update_display_position(&self.camera.borrow());

	    // 衝突情報を取得
            let collision_info = self.player.get_character_object().check_collision_with_character(ctx, e.get_character_object());

	    // collisionフィールドがtrueなら、衝突している
            if collision_info.collision {
		// プレイヤーと他キャラクターの衝突状況から、プレイヤーがどれだけ、突き放されればいいのか計算
                let diff = self.player.get_mut_character_object().fix_collision_vertical(ctx, &collision_info, t);

		// プレイヤーの突き放し距離分動かす
                self.player.get_mut_character_object().move_map(numeric::Vector2f::new(0.0, -diff));

		// プレイヤーのマップ座標を更新
		self.player.get_mut_character_object().update_display_position(&self.camera.borrow());
            }
        }

	// カメラをプレイヤーに合わせる
        self.focus_camera_playable_character_y();
    }

    fn move_playable_character_x(&mut self, ctx: &mut ggez::Context, t: Clock) {
	// プレイヤーのX方向の移動
        self.player.move_map_current_speed_x(1500.0);
	
	// マップ座標を更新, これで、衝突判定を行えるようになる
        self.player.get_mut_character_object().update_display_position(&self.camera.borrow());

        // マップチップとの衝突判定（横）
        self.playable_check_collision_horizon(ctx);

	// 他キャラクターとの当たり判定
        self.check_character_collision_x(ctx, t);
    }

    fn move_playable_character_y(&mut self, ctx: &mut ggez::Context, t: Clock) {
        // プレイヤーのY方向の移動
        self.player.move_map_current_speed_y(1500.0);
	// マップ座標を更新, これで、衝突判定を行えるようになる
        self.player.get_mut_character_object().update_display_position(&self.camera.borrow());

	// マップチップとの衝突判定（縦）
        self.playable_check_collision_vertical(ctx);

	// 他キャラクターとの当たり判定
        self.check_character_collision_y(ctx, t);
    }

    fn move_playable_character(&mut self, ctx: &mut ggez::Context, t: Clock) {
        // キーのチェック
        //self.check_key_event(ctx);
        
        self.player.get_mut_character_object().update_texture(t);

        self.move_playable_character_x(ctx, t);
        self.move_playable_character_y(ctx, t);
    }

    fn check_event_panel_onmap(&mut self,
			       ctx: &mut ggez::Context,
			       game_data: &GameData) {
	let t = self.get_current_clock();
	let target_event = self.map.check_event_panel(EventTrigger::Action,
						      self.player.get_map_position(), self.get_current_clock());
	
	if let Some(event_element) = target_event {
	    match event_element {
		MapEventElement::TextEvent(text) => {
		    println!("{}", text.get_text());
		    
		    let mut scenario_box = ScenarioBox::new(ctx, game_data,
							    numeric::Rect::new(33.0, 470.0, 1300.0, 270.0), t);
		    scenario_box.text_box.set_fixed_text(text.get_text(),
							 FontInformation::new(game_data.get_font(FontID::JpFude1),
									      numeric::Vector2f::new(32.0, 32.0),
									      ggraphics::Color::from_rgba_u32(0x000000ff)));
                    self.map.scenario_box = Some(scenario_box);
                },
		MapEventElement::SwitchScene(switch_scene) => {
		    self.transition_status = SceneTransition::StackingTransition;
		    self.transition_scene = switch_scene.get_switch_scene_id();
		},
		MapEventElement::BookStoreEvent(book_store_event) => {
		    debug::debug_screen_push_text(&format!("book store event: {:?}", book_store_event.get_book_shelf_info()));
		},
	    }
	}
    }

    pub fn start_mouse_move(&mut self,
                            ctx: &mut ggez::Context,
                            point: numeric::Point2f) {
	let current = self.player.get_character_object().obj().get_center(ctx);
	let offset = numeric::Point2f::new(point.x - current.x, point.y - current.y);
	let rad = (offset.y / offset.x).atan();
	let mut speed = numeric::Vector2f::new(rad.cos() * 4.0, rad.sin() * 4.0);
	
	if offset.x < 0.0 {
	    speed.y = -speed.y;
	    speed.x = -speed.x;
	}
	
	self.player.set_speed(speed);
    }

    pub fn switched_and_restart(&mut self) {
	self.transition_scene = SceneID::SuzunaShop;
    }

    pub fn update_task_result(&mut self, game_data: &GameData, task_result: &TaskResult) {
	self.shop_menu.update_contents(game_data, task_result);
    }
}

impl SceneManager for ShopScene {
    
    fn key_down_event(&mut self,
                      ctx: &mut ggez::Context,
                      game_data: &GameData,
                      vkey: tdev::VirtualKey) {
	match vkey {
            tdev::VirtualKey::Action1 => {
		if let Some(scenario_box) = self.map.scenario_box.as_mut() {
		    if scenario_box.get_text_box_status() == TextBoxStatus::FixedText {
			self.map.scenario_box = None;
		    }
		} else {
		    debug::debug_screen_push_text("OK");
		    self.check_event_panel_onmap(ctx, game_data);
		}	
	    },
	    tdev::VirtualKey::Action2 => {
		self.shop_menu.toggle_first_menu(self.get_current_clock());
		if self.shop_menu.first_menu_is_open() {
		    self.dark_effect_panel.new_effect(8, self.get_current_clock(), 0, 200);
		} else {
		    self.dark_effect_panel.new_effect(8, self.get_current_clock(), 200, 0);
		}
	    },
	    tdev::VirtualKey::Action3 => {
		self.drawable_component_list.push(
		    Box::new(
			menu::VerticalMenu::new(ctx, numeric::Point2f::new(100.0, 100.0), 10.0,
						vec!["項目 一".to_string(), "項目 二".to_string(), "項目 三".to_string()],
						FontInformation::new(game_data.get_font(FontID::JpFude1),
								     numeric::Vector2f::new(20.0, 20.0),
								     ggraphics::Color::from_rgba_u32(0xffffffff)))
		    )
		);
	    },
            _ => (),
	}

	for component in &mut self.drawable_component_list {
	    component.virtual_key_event(ctx, KeyboardEvent::FirstPressed, vkey);
	}

	self.shop_menu.menu_key_action(ctx, game_data, vkey, self.get_current_clock());
    }
    
    fn key_up_event(&mut self,
                    ctx: &mut ggez::Context,
                    _game_data: &GameData,
                    vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => println!("Action1 up!"),
            _ => (),
        }

	for component in &mut self.drawable_component_list {
	    component.virtual_key_event(ctx, KeyboardEvent::Typed, vkey);
	}
    }

    fn mouse_motion_event(&mut self,
                          _ctx: &mut ggez::Context,
                          _game_data: &GameData,
                          _point: numeric::Point2f,
                          _offset: numeric::Vector2f) {

    }

    fn mouse_button_down_event(&mut self,
                               ctx: &mut ggez::Context,
                               _game_data: &GameData,
                               button: MouseButton,
                               point: numeric::Point2f) {
	match button {
	    MouseButton::Left => {
		self.start_mouse_move(ctx, point);
	    },
	    MouseButton::Right => {
		self.player.reset_speed();
	    },
	    _ => (),
	}
    }
    
    fn mouse_button_up_event(&mut self,
                             _ctx: &mut ggez::Context,
                             _game_data: &GameData,
                             _button: MouseButton,
                             _point: numeric::Point2f) {
    }

    fn pre_process(&mut self, ctx: &mut ggez::Context, _: &GameData) {
        let t = self.get_current_clock();

	if !self.shop_menu.first_menu_is_open() {
            self.move_playable_character(ctx, t);
	    self.map.check_event_panel(EventTrigger::Touch,
				       self.player.get_map_position(), self.get_current_clock());
            
            self.character_group.move_and_collision_check(ctx, &self.camera.borrow(), &self.map.tile_map, t);
            
            // マップ描画の準備
            self.map.tile_map.update(ctx, t);
	}

	// 暗転の描画
	self.dark_effect_panel.run_effect(ctx, t);
	
	// メニューの更新
	self.shop_menu.update(ctx, t);
    }
    
    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.map.tile_map.draw(ctx).unwrap();

        self.player
            .get_mut_character_object()
            .obj_mut()
            .draw(ctx).unwrap();
        self.character_group.draw(ctx).unwrap();

	if let Some(scenario_box) = self.map.scenario_box.as_mut() {
	    scenario_box.draw(ctx).unwrap();
	}

	for component in &mut self.drawable_component_list {
	    component.draw(ctx).unwrap();
	}

	self.dark_effect_panel.draw(ctx).unwrap();
	
	self.shop_menu.draw(ctx).unwrap();
    }
    
    fn post_process(&mut self, _ctx: &mut ggez::Context, _: &GameData) -> SceneTransition {
        self.update_current_clock();
	self.transition_status
    }

    fn transition(&self) -> SceneID {
	self.transition_scene
    }

    fn get_current_clock(&self) -> Clock {
        self.clock
    }
    
    fn update_current_clock(&mut self) {
        self.clock += 1;
    }
}
