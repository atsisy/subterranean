use std::cell::RefCell;
use std::rc::Rc;

use torifune::device as tdev;
use torifune::graphics::object::*;

use ggez::graphics as ggraphics;
use ggez::input as ginput;
use tdev::{KeyboardEvent, VirtualKey};
use torifune::core::Clock;
use torifune::core::Updatable;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::graphics::object::VerticalText;
use torifune::graphics::*;

use ginput::mouse::MouseButton;
use torifune::debug;
use torifune::numeric;

use super::*;
use crate::core::map_parser as mp;
use crate::core::{BookInformation, FontID, GameData, BookShelfInformation};
use crate::object::map_object::EventTrigger;
use crate::object::map_object::*;
use crate::object::move_fn;
use crate::object::scenario::*;
use crate::object::shop_object::*;
use crate::object::*;
use crate::scene::suzuna_scene::TaskResult;

use number_to_jk::number_to_jk;

struct CharacterGroup {
    group: Vec<CustomerCharacter>,
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
    pub fn add(&mut self, character: CustomerCharacter) {
        self.group.push(character);
    }

    #[inline(always)]
    pub fn remove_if<F>(&mut self, f: F)
    where
        F: Fn(&CustomerCharacter) -> bool,
    {
        self.group.retain(|e| !f(e));
    }

    pub fn len(&self) -> usize {
        self.group.len()
    }

    pub fn move_and_collision_check(
        &mut self,
        ctx: &mut ggez::Context,
        camera: &numeric::Rect,
        tile_map: &mp::StageObjectMap,
        t: Clock,
    ) {
        self.group.iter_mut().for_each(|character| {
            character.move_map_current_speed_y();

            // 当たり判定の前に描画位置を決定しないとバグる。この仕様も直すべき
            character
                .get_mut_character_object()
                .update_display_position(camera);

            ShopScene::check_collision_vertical(
                ctx,
                character.get_mut_character_object(),
                tile_map,
                t,
            );
            character
                .get_mut_character_object()
                .update_display_position(camera);

            character.move_map_current_speed_x();
            character
                .get_mut_character_object()
                .update_display_position(camera);
            ShopScene::check_collision_horizon(
                ctx,
                character.get_mut_character_object(),
                tile_map,
                t,
            );
            character
                .get_mut_character_object()
                .update_display_position(camera);
        });
    }

    pub fn iter(&self) -> std::slice::Iter<CustomerCharacter> {
        self.group.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<CustomerCharacter> {
        self.group.iter_mut()
    }
}

impl DrawableComponent for CharacterGroup {
    #[inline(always)]
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        self.group.iter_mut().for_each(|e| {
            e.get_mut_character_object().obj_mut().draw(ctx).unwrap();
        });
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
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        map_id: u32,
        camera: Rc<RefCell<numeric::Rect>>,
    ) -> Self {
        let map_constract_data = game_data.get_map_data(map_id).unwrap();

        MapData {
            tile_map: mp::StageObjectMap::new(
                ctx,
                &map_constract_data.map_file_path,
                camera.clone(),
                numeric::Rect::new(0.0, 0.0, 1366.0, 768.0),
                numeric::Vector2f::new(6.0, 6.0),
            ),
            event_map: MapEventList::from_file(&map_constract_data.event_map_file_path),
            scenario_box: None,
        }
    }

    pub fn check_event_panel(
        &self,
        trigger: EventTrigger,
        point: numeric::Point2f,
        _t: Clock,
    ) -> Option<&MapEventElement> {
        let tile_size = self.tile_map.get_tile_drawing_size();
        self.event_map.check_event(
            trigger,
            numeric::Point2i::new(
                (point.x as f32 / tile_size.x) as i32,
                (point.y as f32 / tile_size.y) as i32,
            ),
        )
    }
}

///
/// メニューに表示するやつ
///
pub struct ShelvingDetailContents {
    canvas: MovableWrap<SubScreen>,
    menu_rect: numeric::Rect,
    title: VerticalText,
    cell_desc: VerticalText,
    book_info: Vec<VerticalText>,
}

impl ShelvingDetailContents {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        menu_rect: numeric::Rect,
        t: Clock,
    ) -> Self {
        let title = VerticalText::new(
            "配架中".to_string(),
            numeric::Point2f::new(menu_rect.w - 60.0, 70.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                game_data.get_font(FontID::JpFude1),
                numeric::Vector2f::new(40.0, 40.0),
                ggraphics::Color::from_rgba_u32(0xff),
            ),
        );

        let cell_desc = VerticalText::new(
            "請求番号\t\t表題".to_string(),
            numeric::Point2f::new(menu_rect.w - 120.0, 150.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                game_data.get_font(FontID::JpFude1),
                numeric::Vector2f::new(34.0, 34.0),
                ggraphics::Color::from_rgba_u32(0xff),
            ),
        );

        ShelvingDetailContents {
            canvas: MovableWrap::new(
                Box::new(SubScreen::new(
                    ctx,
                    menu_rect,
                    0,
                    ggraphics::Color::from_rgba_u32(0xffffffff),
                )),
                None,
                t,
            ),
            menu_rect: menu_rect,
            title: title,
            cell_desc: cell_desc,
            book_info: Vec::new(),
        }
    }

    pub fn update_contents(
        &mut self,
        game_data: &GameData,
        player_shelving: &Vec<BookInformation>,
    ) {
        self.book_info.clear();

        let mut book_info_position = numeric::Point2f::new(self.menu_rect.w - 180.0, 150.0);
        let book_font_information = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(30.0, 30.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        self.book_info = player_shelving
            .iter()
            .map(|book_info| {
                let vtext = VerticalText::new(
                    format!(
                        "{0:<6}{1}",
                        number_to_jk(book_info.billing_number as u64),
                        &book_info.name
                    ),
                    book_info_position,
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    book_font_information.clone(),
                );

                book_info_position.x -= 35.0;

                vtext
            })
            .collect();
    }

    pub fn slide_appear(&mut self, slide_position: numeric::Point2f, t: Clock) {
        self.canvas
            .override_move_func(move_fn::devide_distance(slide_position, 0.5), t);
    }

    pub fn slide_hide(&mut self, t: Clock) {
        self.canvas.override_move_func(
            move_fn::devide_distance(numeric::Point2f::new(-self.menu_rect.w, 0.0), 0.2),
            t,
        );
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
            self.cell_desc.draw(ctx)?;
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
        let normal_scale_font = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(26.0, 26.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );

        let large_scale_font = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(30.0, 30.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );

        ShopMenuContents {
            day_text: VerticalText::new(
                format!("日付　{}月 {}日", number_to_jk(12), number_to_jk(12)),
                numeric::Point2f::new(370.0, 50.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                large_scale_font,
            ),
            copy_request: VerticalText::new(
                format!("写本受注数"),
                numeric::Point2f::new(300.0, 50.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                normal_scale_font,
            ),
            copy_request_num: VerticalText::new(
                format!("{}件", number_to_jk(0)),
                numeric::Point2f::new(260.0, 150.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                large_scale_font,
            ),
            wait_for_return: VerticalText::new(
                format!("返却待冊数"),
                numeric::Point2f::new(200.0, 50.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                normal_scale_font,
            ),
            wait_for_return_num: VerticalText::new(
                format!("{}冊", number_to_jk(0)),
                numeric::Point2f::new(160.0, 150.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                large_scale_font,
            ),
            not_shelved: VerticalText::new(
                format!("未配架冊数"),
                numeric::Point2f::new(100.0, 50.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                normal_scale_font,
            ),
            not_shelved_num: VerticalText::new(
                format!("{}冊", number_to_jk(0)),
                numeric::Point2f::new(60.0, 150.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                large_scale_font,
            ),
            kosuzu_level: VerticalText::new(
                format!("小鈴 習熟度"),
                numeric::Point2f::new(300.0, 350.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                normal_scale_font,
            ),
            kosuzu_level_num: VerticalText::new(
                format!("{}", number_to_jk(0)),
                numeric::Point2f::new(260.0, 450.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                large_scale_font,
            ),
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }

    pub fn update_contents(&mut self, game_data: &GameData, task_result: &TaskResult) {
        let _normal_scale_font = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(26.0, 26.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );

        let large_scale_font = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(30.0, 30.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );

        self.day_text = VerticalText::new(
            format!("日付　{}月 {}日", number_to_jk(12), number_to_jk(12)),
            numeric::Point2f::new(370.0, 50.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            large_scale_font,
        );

        self.copy_request_num = VerticalText::new(
            format!(
                "{}件",
                number_to_jk(task_result.remain_copy_request.len() as u64)
            ),
            numeric::Point2f::new(260.0, 150.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            large_scale_font,
        );

        self.wait_for_return_num = VerticalText::new(
            format!(
                "{}冊",
                number_to_jk(task_result.borrowing_books.len() as u64)
            ),
            numeric::Point2f::new(160.0, 150.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            large_scale_font,
        );

        self.not_shelved_num = VerticalText::new(
            format!(
                "{}冊",
                number_to_jk(task_result.not_shelved_books.len() as u64)
            ),
            numeric::Point2f::new(60.0, 150.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            large_scale_font,
        );

        self.kosuzu_level_num = VerticalText::new(
            format!("{}", number_to_jk((task_result.done_works / 3) as u64)),
            numeric::Point2f::new(260.0, 450.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            large_scale_font,
        );
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
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        size: numeric::Vector2f,
        t: Clock,
    ) -> Self {
        ShopMenu {
            canvas: MovableWrap::new(
                Box::new(SubScreen::new(
                    ctx,
                    numeric::Rect::new(-size.x, 0.0, size.x, size.y),
                    0,
                    ggraphics::Color::from_rgba_u32(0xffffffff),
                )),
                None,
                t,
            ),
            menu_contents: ShopMenuContents::new(game_data),
            menu_canvas_size: size,
            now_appear: false,
        }
    }

    pub fn slide_toggle(&mut self, t: Clock) {
        if self.now_appear {
            self.canvas.override_move_func(
                move_fn::devide_distance(numeric::Point2f::new(-self.menu_canvas_size.x, 0.0), 0.5),
                t,
            );
            self.now_appear = false;
        } else {
            self.canvas.override_move_func(
                move_fn::devide_distance(numeric::Point2f::new(0.0, 0.0), 0.5),
                t,
            );
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
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        appear_position: numeric::Point2f,
        shelving_rect: numeric::Rect,
        t: Clock,
    ) -> Self {
        ShopDetailMenuContents {
            shelving_info: ShelvingDetailContents::new(ctx, game_data, shelving_rect, t),
            drwob_essential: DrawableObjectEssential::new(true, 0),
            contents_switch: ShopDetailMenuSymbol::None,
            appear_position: appear_position,
            now_appear: false,
        }
    }

    pub fn update_contents(
        &mut self,
        game_data: &GameData,
        player_shelving: &Vec<BookInformation>,
    ) {
        self.shelving_info
            .update_contents(game_data, player_shelving);
    }

    pub fn detail_menu_is_open(&self) -> bool {
        self.now_appear
    }
    
    pub fn hide_toggle(&mut self, t: Clock) {
	self.now_appear = false;
        self.shelving_info.slide_hide(t);
    }

    pub fn appear_toggle(&mut self, t: Clock) {
	self.now_appear = true;
        self.shelving_info.slide_appear(self.appear_position, t);
    }

    pub fn slide_toggle(&mut self, t: Clock) {
        match self.contents_switch {
            ShopDetailMenuSymbol::ShelvingBooks => {
                if self.now_appear {
		    self.hide_toggle(t);
                } else {
		    self.appear_toggle(t);
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
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        first_menu_size: numeric::Vector2f,
        t: Clock,
    ) -> Self {
        ShopMenuMaster {
            first_menu: ShopMenu::new(ctx, game_data, first_menu_size, t),
            detail_menu: ShopDetailMenuContents::new(
                ctx,
                game_data,
                numeric::Point2f::new(first_menu_size.x, 0.0),
                numeric::Rect::new(-450.0, 0.0, 450.0, 768.0),
                t,
            ),
            canvas: SubScreen::new(
                ctx,
                numeric::Rect::new(0.0, 0.0, 1366.0, 768.0),
                0,
                ggraphics::Color::from_rgba_u32(0),
            ),
        }
    }

    pub fn update_contents(
        &mut self,
        game_data: &GameData,
        task_result: &TaskResult,
        player_shelving: &Vec<BookInformation>,
    ) {
        self.first_menu.update_menu_contents(game_data, task_result);
        self.detail_menu
            .update_contents(game_data, player_shelving);
    }

    pub fn toggle_first_menu(&mut self, t: Clock) {
        self.first_menu.slide_toggle(t);
	if !self.first_menu_is_open() {
	    self.detail_menu.hide_toggle(t);
	}
    }

    pub fn first_menu_is_open(&self) -> bool {
        self.first_menu.appearing_now()
    }

    pub fn menu_key_action(
        &mut self,
        _: &mut ggez::Context,
        _: &GameData,
        vkey: VirtualKey,
        t: Clock,
    ) {
        match vkey {
            VirtualKey::Action3 => {
                if self.first_menu_is_open() {
                    debug::debug_screen_push_text("slide detail menu");
                    self.detail_menu
                        .set_slide_contents(ShopDetailMenuSymbol::ShelvingBooks);
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

struct ShopSpecialObject {
    event_list: DelayEventList<Self>,
    shelving_select_ui: Option<MovableWrap<SelectShelvingBookUI>>,
    storing_select_ui: Option<MovableWrap<SelectStoreBookUI>>,
    drwob_essential: DrawableObjectEssential,
}

impl ShopSpecialObject {
    pub fn new() -> Self {
        ShopSpecialObject {
            event_list: DelayEventList::new(),
            shelving_select_ui: None,
	    storing_select_ui: None,
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }

    pub fn show_shelving_select_ui(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        task_result: &TaskResult,
        player_shelving_books: Vec<BookInformation>,
        t: Clock,
    ) {
        if self.shelving_select_ui.is_none() {
            self.shelving_select_ui = Some(MovableWrap::new(
                Box::new(SelectShelvingBookUI::new(
                    ctx,
                    game_data,
                    numeric::Rect::new(0.0, -768.0, 1366.0, 768.0),
                    task_result.not_shelved_books.clone(),
                    player_shelving_books,
                )),
                move_fn::devide_distance(numeric::Point2f::new(0.0, 0.0), 0.4),
                t,
            ));
        }
    }

    pub fn show_storing_select_ui(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
	book_shelf_info: BookShelfInformation,
        player_shelving_books: Vec<BookInformation>,
        t: Clock,
    ) {
        if self.storing_select_ui.is_none() {
            self.storing_select_ui = Some(MovableWrap::new(
                Box::new(SelectStoreBookUI::new(
                    ctx,
                    game_data,
                    numeric::Rect::new(0.0, -768.0, 1366.0, 768.0),
		    book_shelf_info,
                    player_shelving_books,
                )),
                move_fn::devide_distance(numeric::Point2f::new(0.0, 0.0), 0.4),
                t,
            ));
        }
    }

    pub fn hide_shelving_select_ui(
        &mut self,
        t: Clock,
    ) -> Option<(Vec<BookInformation>, Vec<BookInformation>)> {
        if let Some(ui) = self.shelving_select_ui.as_mut() {
            ui.override_move_func(
                move_fn::devide_distance(numeric::Point2f::new(0.0, -768.0), 0.4),
                t,
            );
            self.event_list.add_event(
                Box::new(|shop_special_object, _, _| {
                    shop_special_object.shelving_select_ui = None;
                }),
                t + 7,
            );

            Some(ui.ref_wrapped_object().get_select_result())
        } else {
            None
        }
    }

    pub fn hide_storing_select_ui(
        &mut self,
        t: Clock,
    ) -> Option<(Vec<BookInformation>, Vec<BookInformation>)> {
        if let Some(ui) = self.storing_select_ui.as_mut() {
            ui.override_move_func(
                move_fn::devide_distance(numeric::Point2f::new(0.0, -768.0), 0.4),
                t,
            );
            self.event_list.add_event(
                Box::new(|shop_special_object, _, _| {
                    shop_special_object.storing_select_ui = None;
                }),
                t + 7,
            );

            Some(ui.ref_wrapped_object().get_storing_result())
        } else {
            None
        }
    }
    
    pub fn is_enable_now(&self) -> bool {
        self.shelving_select_ui.is_some() || self.storing_select_ui.is_some()
    }

    pub fn mouse_down_action(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        button: MouseButton,
        point: numeric::Point2f,
        t: Clock,
    ) {
        if let Some(ui) = self.shelving_select_ui.as_mut() {
            ui.ref_wrapped_object_mut()
                .on_click(ctx, game_data, t, button, point);
        }

	if let Some(ui) = self.storing_select_ui.as_mut() {
            ui.ref_wrapped_object_mut()
                .on_click(ctx, game_data, t, button, point);
        }
    }

    pub fn run_delay_event(&mut self, ctx: &mut ggez::Context, game_data: &GameData, t: Clock) {
        // 最後の要素の所有権を移動
        while let Some(event) = self.event_list.move_top() {
            // 時間が来ていない場合は、取り出した要素をリストに戻して処理ループを抜ける
            if event.run_time > t {
                self.event_list.add(event);
                break;
            }

            // 所有権を移動しているため、selfを渡してもエラーにならない
            (event.func)(self, ctx, game_data);
        }
    }
}

impl Updatable for ShopSpecialObject {
    fn update(&mut self, _: &ggez::Context, t: Clock) {
        if let Some(ui) = self.shelving_select_ui.as_mut() {
            ui.move_with_func(t);
        }

	if let Some(storing_ui) = self.storing_select_ui.as_mut() {
            storing_ui.move_with_func(t);
        }
    }
}

impl DrawableComponent for ShopSpecialObject {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            if let Some(select_ui) = self.shelving_select_ui.as_mut() {
                select_ui.draw(ctx)?;
            }

	    if let Some(store_ui) = self.storing_select_ui.as_mut() {
                store_ui.draw(ctx)?;
            }
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
    shop_object_list: Vec<Box<dyn Clickable>>,
    shop_special_object: ShopSpecialObject,
    key_listener: tdev::KeyboardListener,
    task_result: TaskResult,
    clock: Clock,
    map: MapData,
    shop_menu: ShopMenuMaster,
    wait_customer_flag: bool,
    camera: Rc<RefCell<numeric::Rect>>,
    dark_effect_panel: DarkEffectPanel,
    transition_status: SceneTransition,
    transition_scene: SceneID,
}

impl ShopScene {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData, map_id: u32) -> ShopScene {
        let key_listener =
            tdev::KeyboardListener::new_masked(vec![tdev::KeyInputDevice::GenericKeyboard], vec![]);

        let camera = Rc::new(RefCell::new(numeric::Rect::new(0.0, 0.0, 1366.0, 768.0)));

        let map_position = numeric::Point2f::new(800.0, 830.0);

        let player = PlayableCharacter::new(
            character_factory::create_character(
                character_factory::CharacterFactoryOrder::PlayableDoremy1,
                game_data,
                &camera.borrow(),
                map_position,
            ),
            PlayerStatus { hp: 10, mp: 10.0 },
        );

        let mut character_group = CharacterGroup::new();
	character_group.add(CustomerCharacter::new(
	    character_factory::create_character(
		character_factory::CharacterFactoryOrder::CustomerSample,
		game_data, &camera.borrow(), numeric::Point2f::new(1170.0, 870.0)),
	    CustomerDestPoint::new(
		vec![
		    numeric::Vector2u::new(10, 3),
		    numeric::Vector2u::new(6, 3),
		    numeric::Vector2u::new(4, 10)]
	    )));
	    
	let mut map = MapData::new(ctx, game_data, map_id, camera.clone());
	map.tile_map.build_collision_map();

        ShopScene {
            player: player,
            character_group: character_group,
            shop_object_list: Vec::new(),
            shop_special_object: ShopSpecialObject::new(),
            key_listener: key_listener,
            task_result: TaskResult::new(),
            clock: 0,
            map: map,
            shop_menu: ShopMenuMaster::new(ctx, game_data, numeric::Vector2f::new(450.0, 768.0), 0),
	    wait_customer_flag: false,
            dark_effect_panel: DarkEffectPanel::new(
                ctx,
                numeric::Rect::new(0.0, 0.0, 1366.0, 768.0),
                0,
            ),
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
            //self.player.reset_speed();
            if self
                .key_listener
                .current_key_status(ctx, &VirtualKey::RightSub)
                == tdev::KeyStatus::Pressed
            {
                self.right_key_handler();
            }

            if self
                .key_listener
                .current_key_status(ctx, &VirtualKey::LeftSub)
                == tdev::KeyStatus::Pressed
            {
                self.left_key_handler();
            }

            if self
                .key_listener
                .current_key_status(ctx, &VirtualKey::UpSub)
                == tdev::KeyStatus::Pressed
            {
                self.up_key_handler();
            }

            if self
                .key_listener
                .current_key_status(ctx, &VirtualKey::DownSub)
                == tdev::KeyStatus::Pressed
            {
                self.down_key_handler();
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

    fn right_key_handler(&mut self) {
        self.player
            .get_mut_character_object()
            .change_animation_mode(2);
        self.player.set_speed_x(4.0);
    }

    fn left_key_handler(&mut self) {
        self.player
            .get_mut_character_object()
            .change_animation_mode(3);
        self.player.set_speed_x(-4.0);
    }

    fn up_key_handler(&mut self) {
        self.player
            .get_mut_character_object()
            .change_animation_mode(1);
        self.player.set_speed_y(-4.0);
    }

    fn down_key_handler(&mut self) {
        self.player
            .get_mut_character_object()
            .change_animation_mode(0);
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
            },
        )
    }

    ///
    /// マップオブジェクトとの衝突を調べるためのメソッド
    ///
    fn check_collision_horizon(
        ctx: &mut ggez::Context,
        character: &mut MapObject,
        tile_map: &mp::StageObjectMap,
        t: Clock,
    ) {
        let collision_info = tile_map.check_character_collision(ctx, character);

        // 衝突していたか？
        if collision_info.collision {
            // 修正動作
            let diff = character.fix_collision_horizon(ctx, &collision_info, t);
            character.move_map(numeric::Vector2f::new(diff, 0.0));
        }
    }

    ///
    /// マップオブジェクトとの衝突を調べるためのメソッド
    ///
    fn check_collision_vertical(
        ctx: &mut ggez::Context,
        character: &mut MapObject,
        tile_map: &mp::StageObjectMap,
        t: Clock,
    ) {
        let collision_info = tile_map.check_character_collision(ctx, character);

        // 衝突していたか？
        if collision_info.collision {
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
        Self::check_collision_horizon(
            ctx,
            self.player.get_mut_character_object(),
            &self.map.tile_map,
            t,
        );

        // プレイヤーのマップ座標を更新
        self.player
            .get_mut_character_object()
            .update_display_position(&self.camera.borrow());

        // カメラをプレイヤーにフォーカス
        self.focus_camera_playable_character_x();
    }

    ///
    /// PlayerCharacterのマップチップとのY方向の衝突を修正する
    ///
    fn playable_check_collision_vertical(&mut self, ctx: &mut ggez::Context) {
        let t = self.get_current_clock();

        // プレイヤーのマップチップに対する衝突を修正
        Self::check_collision_vertical(
            ctx,
            self.player.get_mut_character_object(),
            &self.map.tile_map,
            t,
        );

        // プレイヤーのマップ座標を更新
        self.player
            .get_mut_character_object()
            .update_display_position(&self.camera.borrow());

        // カメラをプレイヤーにフォーカス
        self.focus_camera_playable_character_y();
    }

    fn focus_camera_playable_character_x(&mut self) {
        // カメラとプレイヤーキャラクターの差分を計算し、プレイヤーが中心に来るようにカメラを移動
        let player_diff = self.player.get_mut_character_object().obj().get_position()
            - self.fix_camera_position();
        self.move_camera(numeric::Vector2f::new(player_diff.x, 0.0));
    }

    fn focus_camera_playable_character_y(&mut self) {
        // カメラとプレイヤーキャラクターの差分を計算し、プレイヤーが中心に来るようにカメラを移動
        let player_diff = self.player.get_mut_character_object().obj().get_position()
            - self.fix_camera_position();
        self.move_camera(numeric::Vector2f::new(0.0, player_diff.y));
    }

    ///
    /// PlayerCharacterの他キャラクターとのX方向の衝突を修正する
    ///
    fn check_character_collision_x(&mut self, ctx: &mut ggez::Context, t: Clock) {
        // マップ座標を更新
        self.player
            .get_mut_character_object()
            .update_display_position(&self.camera.borrow());

        // 他キャラクターすべてとの衝突判定を行う
        for e in self.character_group.iter_mut() {
            // 他キャラクターのマップ座標を更新
            e.get_mut_character_object()
                .update_display_position(&self.camera.borrow());

            // 衝突情報を取得
            let collision_info = self
                .player
                .get_character_object()
                .check_collision_with_character(ctx, e.get_character_object());

            // collisionフィールドがtrueなら、衝突している
            if collision_info.collision {
                // プレイヤーと他キャラクターの衝突状況から、プレイヤーがどれだけ、突き放されればいいのか計算
                let diff = self
                    .player
                    .get_mut_character_object()
                    .fix_collision_horizon(ctx, &collision_info, t);

                // プレイヤーの突き放し距離分動かす
                self.player
                    .get_mut_character_object()
                    .move_map(numeric::Vector2f::new(-diff, 0.0));

                // プレイヤーのマップ座標を更新
                self.player
                    .get_mut_character_object()
                    .update_display_position(&self.camera.borrow());
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
        self.player
            .get_mut_character_object()
            .update_display_position(&self.camera.borrow());

        // 他キャラクターすべてとの衝突判定を行う
        for e in self.character_group.iter_mut() {
            // 他キャラクターのマップ座標を更新
            e.get_mut_character_object()
                .update_display_position(&self.camera.borrow());

            // 衝突情報を取得
            let collision_info = self
                .player
                .get_character_object()
                .check_collision_with_character(ctx, e.get_character_object());

            // collisionフィールドがtrueなら、衝突している
            if collision_info.collision {
                // プレイヤーと他キャラクターの衝突状況から、プレイヤーがどれだけ、突き放されればいいのか計算
                let diff = self
                    .player
                    .get_mut_character_object()
                    .fix_collision_vertical(ctx, &collision_info, t);

                // プレイヤーの突き放し距離分動かす
                self.player
                    .get_mut_character_object()
                    .move_map(numeric::Vector2f::new(0.0, -diff));

                // プレイヤーのマップ座標を更新
                self.player
                    .get_mut_character_object()
                    .update_display_position(&self.camera.borrow());
            }
        }

        // カメラをプレイヤーに合わせる
        self.focus_camera_playable_character_y();
    }

    fn move_playable_character_x(&mut self, ctx: &mut ggez::Context, t: Clock) {
        // プレイヤーのX方向の移動
        self.player.move_map_current_speed_x(1500.0);

        // マップ座標を更新, これで、衝突判定を行えるようになる
        self.player
            .get_mut_character_object()
            .update_display_position(&self.camera.borrow());

        // マップチップとの衝突判定（横）
        self.playable_check_collision_horizon(ctx);

        // 他キャラクターとの当たり判定
        self.check_character_collision_x(ctx, t);
    }

    fn move_playable_character_y(&mut self, ctx: &mut ggez::Context, t: Clock) {
        // プレイヤーのY方向の移動
        self.player.move_map_current_speed_y(1500.0);
        // マップ座標を更新, これで、衝突判定を行えるようになる
        self.player
            .get_mut_character_object()
            .update_display_position(&self.camera.borrow());

        // マップチップとの衝突判定（縦）
        self.playable_check_collision_vertical(ctx);

        // 他キャラクターとの当たり判定
        self.check_character_collision_y(ctx, t);
    }

    fn move_playable_character(&mut self, ctx: &mut ggez::Context, t: Clock) {
        // キーのチェック
        self.check_key_event(ctx);

        self.player.get_mut_character_object().update_texture(t);

        self.move_playable_character_x(ctx, t);
        self.move_playable_character_y(ctx, t);
    }

    pub fn run_builtin_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        builtin_event: BuiltinEvent,
    ) {
        match builtin_event.get_event_symbol() {
            BuiltinEventSymbol::SelectShelvingBook => {
                self.dark_effect_panel
                    .new_effect(8, self.get_current_clock(), 0, 200);
                self.shop_special_object.show_shelving_select_ui(
                    ctx,
                    game_data,
                    &self.task_result,
                    self.player.get_shelving_book().clone(),
                    self.get_current_clock(),
                );
            }
        }
    }

    fn check_event_panel_onmap(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        trigger: EventTrigger,
    ) {
        let t = self.get_current_clock();
        let target_event = self.map.check_event_panel(
            trigger,
            self.player.get_center_map_position(ctx),
            self.get_current_clock(),
        );

        if let Some(event_element) = target_event {
            match event_element {
                MapEventElement::TextEvent(text) => {
                    println!("{}", text.get_text());

                    let mut scenario_box = ScenarioBox::new(
                        ctx,
                        game_data,
                        numeric::Rect::new(33.0, 470.0, 1300.0, 270.0),
                        t,
                    );
                    scenario_box.text_box.set_fixed_text(
                        text.get_text(),
                        FontInformation::new(
                            game_data.get_font(FontID::JpFude1),
                            numeric::Vector2f::new(32.0, 32.0),
                            ggraphics::Color::from_rgba_u32(0x000000ff),
                        ),
                    );
                    self.map.scenario_box = Some(scenario_box);
                }
                MapEventElement::SwitchScene(switch_scene) => {
		    if self.wait_customer_flag {
			self.transition_status = SceneTransition::StackingTransition;
			self.transition_scene = switch_scene.get_switch_scene_id();
		    }
                }
                MapEventElement::BookStoreEvent(book_store_event) => {
                    debug::debug_screen_push_text(&format!(
                        "book store event: {:?}",
                        book_store_event.get_book_shelf_info()
                    ));
		    self.dark_effect_panel
			.new_effect(8, self.get_current_clock(), 0, 200);
		    self.shop_special_object.show_storing_select_ui(ctx, game_data,
								    book_store_event.get_book_shelf_info().clone(),
								    self.player.get_shelving_book().clone(),
								    t);
                }
                MapEventElement::BuiltinEvent(builtin_event) => {
                    let builtin_event = builtin_event.clone();
                    self.run_builtin_event(ctx, game_data, builtin_event);
                }
            }
        }
    }

    fn update_playable_character_texture(&mut self, rad: f32) {
	
	if rad >= 45.0_f32.to_radians() && rad < 135.0_f32.to_radians() {
	    // 上向き
            self.player
                .get_mut_character_object()
                .change_animation_mode(0);
	}

	if rad >= 135.0_f32.to_radians() && rad < 225.0_f32.to_radians() {
	    // 左向き
            self.player
                .get_mut_character_object()
                .change_animation_mode(3);
	}

	if rad >= 225.0_f32.to_radians() && rad < 315.0_f32.to_radians() {
	    // 下向き
            self.player
                .get_mut_character_object()
                .change_animation_mode(1);
	}

	if (rad >= 315.0_f32.to_radians() && rad <= 360.0_f32.to_radians()) ||
	    (rad >= 0.0_f32.to_radians() && rad < 45.0_f32.to_radians()){
		// 右向き
                self.player
                    .get_mut_character_object()
                    .change_animation_mode(2);
	}

    }

    pub fn start_mouse_move(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        let current = self.player.get_character_object().obj().get_center(ctx);
        let offset = numeric::Point2f::new(point.x - current.x, point.y - current.y);
 	let rad =
	    if offset.x >= 0.0 {
		if offset.y >= 0.0 {
		    (offset.y / offset.x).atan()	
		} else {
		    (offset.y / offset.x).atan() + 360.0_f32.to_radians()
		}
	    } else {
		(offset.y / offset.x).atan() + 180.0_f32.to_radians()
	    };
	let speed = numeric::Vector2f::new(rad.cos() * 4.0, rad.sin() * 4.0);
	
	self.player.set_speed(speed);
        self.update_playable_character_texture(rad);
    }

    pub fn switched_and_restart(&mut self) {
        self.transition_scene = SceneID::SuzunaShop;
    }

    pub fn update_task_result(&mut self, game_data: &GameData, task_result: &TaskResult) {
        self.shop_menu
            .update_contents(game_data, task_result, self.player.get_shelving_book());
        self.task_result = task_result.clone();
    }

    fn try_hide_shelving_select_ui(&mut self, game_data: &GameData) {
	let select_result = self
            .shop_special_object
            .hide_shelving_select_ui(self.get_current_clock());
        if let Some((boxed, shelving)) = select_result {
            self.task_result.not_shelved_books = boxed;
            self.player.update_shelving_book(shelving);
            self.shop_menu.update_contents(
                game_data,
                &self.task_result,
                self.player.get_shelving_book(),
                    );
            self.dark_effect_panel
                .new_effect(8, self.get_current_clock(), 200, 0);
        }
    }

    fn try_hide_storing_select_ui(&mut self, game_data: &GameData) {
	let store_result = self
            .shop_special_object
            .hide_storing_select_ui(self.get_current_clock());
        if let Some((_stored, shelving)) = store_result {
            self.player.update_shelving_book(shelving);
            self.shop_menu.update_contents(
                game_data,
                &self.task_result,
                self.player.get_shelving_book());
            self.dark_effect_panel
                .new_effect(8, self.get_current_clock(), 200, 0);
        }
    }
}

impl SceneManager for ShopScene {
    fn key_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        vkey: tdev::VirtualKey,
    ) {
        match vkey {
            tdev::VirtualKey::Action1 => {
                if let Some(scenario_box) = self.map.scenario_box.as_mut() {
                    if scenario_box.get_text_box_status() == TextBoxStatus::FixedText {
                        self.map.scenario_box = None;
                    }
                } else {
                    debug::debug_screen_push_text("OK");
                    self.check_event_panel_onmap(ctx, game_data, EventTrigger::Action);
                }
            },
            tdev::VirtualKey::Action2 => {
                self.shop_menu.toggle_first_menu(self.get_current_clock());
                if self.shop_menu.first_menu_is_open() {
                    self.dark_effect_panel
                        .new_effect(8, self.get_current_clock(), 0, 200);
                } else {
                    self.dark_effect_panel
                        .new_effect(8, self.get_current_clock(), 200, 0);
                }
            },
            tdev::VirtualKey::Action3 => {
		self.try_hide_shelving_select_ui(game_data);
		self.try_hide_storing_select_ui(game_data);
            }
            _ => (),
        }

        for component in &mut self.shop_object_list {
            component.virtual_key_event(ctx, KeyboardEvent::FirstPressed, vkey);
        }

        self.shop_menu
            .menu_key_action(ctx, game_data, vkey, self.get_current_clock());
    }

    fn key_up_event(
        &mut self,
        ctx: &mut ggez::Context,
        _game_data: &GameData,
        vkey: tdev::VirtualKey,
    ) {
        match vkey {
            tdev::VirtualKey::Action1 => println!("Action1 up!"),
            _ => (),
        }

        for component in &mut self.shop_object_list {
            component.virtual_key_event(ctx, KeyboardEvent::Typed, vkey);
        }
    }

    fn mouse_motion_event(
        &mut self,
        ctx: &mut ggez::Context,
        _game_data: &GameData,
        point: numeric::Point2f,
        _offset: numeric::Vector2f,
    ) {
        if ggez::input::mouse::button_pressed(ctx, MouseButton::Left) {
            self.start_mouse_move(ctx, point);
        }
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        match button {
            MouseButton::Left => {
                let t = self.get_current_clock();
                for shop_object in &mut self.shop_object_list {
                    shop_object.on_click(ctx, game_data, t, button, point);
                }
		self.start_mouse_move(ctx, point);
            }
            MouseButton::Right => {
                self.player.reset_speed();
            }
            _ => (),
        }

        self.shop_special_object.mouse_down_action(
            ctx,
            game_data,
            button,
            point,
            self.get_current_clock(),
        );
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _game_data: &GameData,
        button: MouseButton,
        _point: numeric::Point2f,
    ) {
        match button {
            MouseButton::Left => {
                self.player.reset_speed();
            }
            _ => (),
        }
    }
    
    fn pre_process(&mut self, ctx: &mut ggez::Context, game_data: &GameData) {
        let t = self.get_current_clock();

        if !self.shop_menu.first_menu_is_open() && !self.shop_special_object.is_enable_now() {
            self.move_playable_character(ctx, t);
            self.check_event_panel_onmap(ctx, game_data, EventTrigger::Touch);

            self.character_group.move_and_collision_check(
                ctx,
                &self.camera.borrow(),
                &self.map.tile_map,
                t,
            );

	    for customer in self.character_group.iter_mut() {
		if let Some(request) = customer.check_rise_hand(game_data) {
		    self.wait_customer_flag = true;
		}
		
		customer.try_update_move_effect(ctx, game_data, &self.map.tile_map, numeric::Vector2u::new(4, 10), t);
		customer.get_mut_character_object().update_texture(t);
	    }

            // マップ描画の準備
            self.map.tile_map.update(ctx, t);
        }

        // 暗転の描画
        self.dark_effect_panel.run_effect(ctx, t);

        // select_uiなどの更新
        self.shop_special_object.run_delay_event(ctx, game_data, t);
        self.shop_special_object.update(ctx, t);

        // メニューの更新
        self.shop_menu.update(ctx, t);
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.map.tile_map.draw(ctx).unwrap();

	self.player.draw(ctx).unwrap();
        self.character_group.draw(ctx).unwrap();

        if let Some(scenario_box) = self.map.scenario_box.as_mut() {
            scenario_box.draw(ctx).unwrap();
        }

        for component in &mut self.shop_object_list {
            component.draw(ctx).unwrap();
        }

        self.dark_effect_panel.draw(ctx).unwrap();

        self.shop_special_object.draw(ctx).unwrap();
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
