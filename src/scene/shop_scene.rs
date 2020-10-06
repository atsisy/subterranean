use std::cell::RefCell;
use std::cmp::PartialOrd;
use std::collections::VecDeque;
use std::rc::Rc;

use torifune::device as tdev;
use torifune::graphics::object::*;
use torifune::manhattan_distance;

use ggez::graphics as ggraphics;
use torifune::core::Clock;
use torifune::core::Updatable;
use torifune::graphics::drawable::*;

use ggez::input::mouse::MouseButton;
use torifune::numeric;

use super::*;
use crate::core::map_parser as mp;
use crate::core::util;
use crate::core::{
    BookInformation, FontID, ResultReport, SavableData, SuzuContext, TileBatchTextureID,
};
use crate::flush_delay_event;
use crate::flush_delay_event_and_redraw_check;
use crate::object::effect_object;
use crate::object::map_object::*;
use crate::object::notify;
use crate::object::scenario::*;
use crate::object::shop_object::*;
use crate::object::task_object::tt_main_component::CustomerRequest;
use crate::object::task_object::tt_sub_component::BookConditionEvalReport;
use crate::object::util_object::*;
use crate::object::*;
use effect_object::{SceneTransitionEffectType, TilingEffectType};
use notify::*;
use crate::perf_measure;

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
    pub fn drain_remove_if<F>(&mut self, f: F) -> Vec<CustomerCharacter>
    where
        F: Fn(&CustomerCharacter) -> bool,
    {
        let mut removed = Vec::new();

        for index in (0..self.group.len()).rev() {
            if f(self.group.get(index).as_ref().unwrap()) {
                let removed_character = self.group.swap_remove(index);
                removed.push(removed_character);
            }
        }

        removed
    }

    #[inline(always)]
    pub fn remove_if<F>(&mut self, f: F)
    where
        F: Fn(&CustomerCharacter) -> bool,
    {
        self.group.retain(|c| !f(c));
    }

    pub fn sort_by_y_position(&mut self) {
        self.group.sort_by(|a, b| {
            a.get_map_position()
                .y
                .partial_cmp(&b.get_map_position().y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    pub fn move_and_collision_check(
        &mut self,
        ctx: &mut ggez::Context,
        camera: &numeric::Rect,
        tile_map: &mp::StageObjectMap,
        t: Clock,
    ) {
        self.group.iter_mut().for_each(|customer| {
            ShopScene::customer_move_and_collision_check(ctx, customer, camera, tile_map, t)
        });

	// for target_index in 0..self.group.len() {
	//     for checked_index in 0..self.group.len() {
	// 	if target_index == checked_index {
	// 	    continue;
	// 	}

	// 	let checked = self.group.get_mut(checked_index).unwrap().get_mut_character_object();
	// 	checked.update_display_position(camera);

	// 	let target = self.group.get_mut(target_index).unwrap().get_mut_character_object();
	// 	target.update_display_position(camera);
		
	// 	let checked = self.group.get(checked_index).unwrap().get_character_object();
	// 	let target = self.group.get(target_index).unwrap().get_character_object();

	// 	// 衝突情報を取得
	// 	let collision_info = target
	// 	    .check_collision_with_character(ctx, checked);

	// 	let target = self.group.get_mut(target_index).unwrap().get_mut_character_object();

	// 	// collisionフィールドがtrueなら、衝突している
	// 	if collision_info.collision {
	// 	    // プレイヤーと他キャラクターの衝突状況から、プレイヤーがどれだけ、突き放されればいいのか計算
	// 	    let diff =
	// 		target
	// 		.fix_collision_horizon(ctx, &collision_info, t);
		    
	// 	    // プレイヤーの突き放し距離分動かす
	// 	    target
	// 		.move_map(numeric::Vector2f::new(-diff, 0.0));
		    
	// 	    // プレイヤーのマップ座標を更新
	// 	    target
	// 		.update_display_position(camera);
	// 	}
	//     }
	// }
	
    }

    pub fn iter(&self) -> std::slice::Iter<CustomerCharacter> {
        self.group.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<CustomerCharacter> {
        self.group.iter_mut()
    }

    pub fn pickup_goto_check_customer(&mut self) -> Option<CustomerCharacter> {
	let len = self.group.len();

	for index in 0..len {
	    if self.group.get(index).unwrap().ready_to_check() {
		return Some(self.group.remove(index));
	    }
	}

	None
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
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        map_id: u32,
        camera: Rc<RefCell<numeric::Rect>>,
    ) -> Self {
        let map_constract_data = ctx.resource.get_map_data(map_id).unwrap();

        MapData {
            tile_map: mp::StageObjectMap::new(
                ctx.context,
                &map_constract_data.map_file_path,
                camera.clone(),
                numeric::Rect::new(0.0, 0.0, 1366.0, 768.0),
                numeric::Vector2f::new(3.0, 3.0),
            ),
            event_map: MapEventList::from_file(&map_constract_data.event_map_file_path),
            scenario_box: None,
        }
    }

    pub fn get_tile_size(&self) -> numeric::Vector2f {
        self.tile_map.get_tile_drawing_size()
    }

    pub fn check_event_panel(
        &self,
        trigger: EventTrigger,
        point: numeric::Point2f,
        _t: Clock,
    ) -> Option<&MapEventElement> {
        let tile_size = self.get_tile_size();
        self.event_map.check_event(
            trigger,
            numeric::Point2i::new(
                (point.x as f32 / tile_size.x) as i32,
                (point.y as f32 / tile_size.y) as i32,
            ),
        )
    }
}

struct MapObjectDrawer<'a> {
    ref_list: Vec<Box<&'a mut dyn OnMap>>,
}

impl<'a> MapObjectDrawer<'a> {
    pub fn new() -> MapObjectDrawer<'a> {
        MapObjectDrawer {
            ref_list: Vec::new(),
        }
    }

    pub fn add(&mut self, onmap: &'a mut dyn OnMap) {
        self.ref_list.push(Box::new(onmap));
    }

    pub fn sort(&mut self, ctx: &mut ggez::Context) {
        self.ref_list.sort_by(|a, b| {
            a.get_map_position_bottom_right(ctx)
                .y
                .partial_cmp(&b.get_map_position_bottom_right(ctx).y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    pub fn draw(&mut self, ctx: &mut ggez::Context) {
        for obj in &mut self.ref_list {
            obj.draw(ctx).unwrap();
        }

        self.ref_list.clear();
    }
}

pub struct CustomerQueue {
    customer_queue: VecDeque<(CustomerCharacter, Clock)>,
    drwob_essential: DrawableObjectEssential,
}

impl CustomerQueue {
    pub fn new(depth: i8) -> Self {
	CustomerQueue {
	    customer_queue: VecDeque::new(),
	    drwob_essential: DrawableObjectEssential::new(true, depth),
	}
    }

    pub fn push_back(&mut self, customer: CustomerCharacter, t: Clock) {
	self.customer_queue.push_back((customer, t));
    }

    pub fn pop_head_customer(&mut self) -> Option<(CustomerCharacter, Clock)> {
	self.customer_queue.pop_front()
    }

    pub fn iter_mut(&mut self) -> std::collections::vec_deque::IterMut<(CustomerCharacter, Clock)> {
	self.customer_queue.iter_mut()
    }

    pub fn is_empty(&self) -> bool {
	self.customer_queue.is_empty()
    }

    pub fn len(&self) -> usize {
	self.customer_queue.len()
    }

    pub fn drain_giveup_customers(&mut self, now: Clock) -> Vec<CustomerCharacter> {
	let mut giveup_customers = Vec::new();

        for index in (0..self.customer_queue.len()).rev() {
            let (_, t) = self.customer_queue.get(index).unwrap();

            if (now - t) > 12000 {
                let (giveup, _) = self.customer_queue.remove(index).unwrap();
                giveup_customers.push(giveup);
            }
        }

	giveup_customers
    }

    pub fn tail_map_position(&self) -> numeric::Vector2u {
	numeric::Vector2u::new(5 + self.len() as u32, 14)
    }
}

impl DrawableComponent for CustomerQueue {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
	if self.is_visible() {
	    for (c, _) in self.customer_queue.iter_mut() {
		c.draw(ctx)?;
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

pub struct GoToCheckCustomers {
    customers: Vec<CustomerCharacter>,
    drwob_essential: DrawableObjectEssential,
    current_check_queue_tail: numeric::Vector2u,
}

impl GoToCheckCustomers {
    pub fn new(check_position: numeric::Vector2u, depth: i8) -> Self {
	GoToCheckCustomers {
	    customers: Vec::new(),
	    drwob_essential: DrawableObjectEssential::new(true, depth),
	    current_check_queue_tail: check_position,
	}
    }

    fn sort_customers(&mut self, map_data: &mp::StageObjectMap) {
	let mut sorted = Vec::new();
	let tile_size = map_data.get_tile_drawing_size();
	let mut tail = numeric::Vector2f::new(
	    self.current_check_queue_tail.x as f32 * tile_size.x,
	    self.current_check_queue_tail.y as f32 * tile_size.y,
	);

	for _ in 0..self.customers.len() {
	    self.customers.sort_unstable_by(|a, b| {
		let a_pos = a.get_map_position();
		let b_pos = b.get_map_position();
		let a_man = manhattan_distance!(a_pos, tail);
		let b_man = manhattan_distance!(b_pos, tail);
		
		a_man.partial_cmp(&b_man).unwrap()
	    });
	    sorted.push(self.customers.pop().unwrap());
	    tail.x += tile_size.x;
	}

	self.customers = sorted;
    }

    pub fn insert_new_customer<'a>(
	&mut self,
	ctx: &mut SuzuContext<'a>,
	customer: CustomerCharacter,
	map_data: &mp::StageObjectMap,
	current_tail: numeric::Vector2u,
    ) {
	self.customers.push(customer);
	self.reset_each_customers_goal(ctx, map_data, current_tail);
    }

    pub fn reset_each_customers_goal<'a>(
	&mut self,
	ctx: &mut SuzuContext<'a>,
	map_data: &mp::StageObjectMap,
	mut current_tail: numeric::Vector2u
    ) {
	self.sort_customers(map_data);

	for customer in self.customers.iter_mut() {
	    customer.goto_check(ctx.context, map_data, current_tail);
	    current_tail.x += 1;
	}
    }

    pub fn move_and_collision_check(
        &mut self,
        ctx: &mut ggez::Context,
        camera: &numeric::Rect,
        tile_map: &mp::StageObjectMap,
        t: Clock,
    ) {
        self.customers.iter_mut().for_each(|customer| {
            ShopScene::customer_move_and_collision_check(ctx, customer, camera, tile_map, t)
        });
    }

    pub fn go_moving<'a>(&mut self, ctx: &mut SuzuContext<'a>, camera: &numeric::Rect, map_data: &mp::StageObjectMap, t: Clock) {
	self.move_and_collision_check(ctx.context, camera, map_data, t);
	
	for customer in self.customers.iter_mut() {
	    customer.try_update_move_effect(
                ctx,
                map_data,
                numeric::Vector2u::new(5, 14),
                numeric::Vector2u::new(15, 14),
                t,
            );
            customer.get_mut_character_object().update_texture(t);
	}
    }

    pub fn drain_remove_if<F>(&mut self, f: F) -> Vec<CustomerCharacter>
    where
        F: Fn(&CustomerCharacter) -> bool,
    {
        let mut removed = Vec::new();

        for index in (0..self.customers.len()).rev() {
            if f(self.customers.get(index).as_ref().unwrap()) {
                let removed_character = self.customers.swap_remove(index);
                removed.push(removed_character);
            }
        }

        removed
    }

    pub fn debug_print(&self, map_data: &mp::StageObjectMap) {
	let tile_size = map_data.get_tile_drawing_size();
	for customer in self.customers.iter() {
	    let map_pos = customer.get_map_position();
	    println!("goto customer => {}, {}", map_pos.x / tile_size.x, map_pos.y / tile_size.y);
	}
    }
}

impl DrawableComponent for GoToCheckCustomers {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
	if self.is_visible() {
	    for c in self.customers.iter_mut() {
		c.draw(ctx)?;
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
    shop_special_object: ShopSpecialObject,
    key_listener: tdev::KeyboardListener,
    clock: Clock,
    shop_clock: ShopClock,
    map: MapData,
    event_list: DelayEventList<Self>,
    result_report: ResultReport,
    shop_menu: ShopMenuMaster,
    shop_map: MovableWrap<ShopMapViewer>,
    shop_map_is_staged: bool,
    customer_request_queue: VecDeque<CustomerRequest>,
    customer_queue: CustomerQueue,
    goto_check_customers: GoToCheckCustomers,
    camera: Rc<RefCell<numeric::Rect>>,
    dark_effect_panel: DarkEffectPanel,
    pause_screen_set: Option<PauseScreenSet>,
    transition_status: SceneTransition,
    transition_scene: SceneID,
    scene_transition_effect: Option<effect_object::ScreenTileEffect>,
    notification_area: NotificationArea,
    begining_save_data: SavableData,
    drawable_shop_clock: DrawableShopClock,
}

impl ShopScene {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        map_id: u32,
        new_books: Vec<BookInformation>,
    ) -> ShopScene {
        let key_listener =
            tdev::KeyboardListener::new_masked(vec![tdev::KeyInputDevice::GenericKeyboard], vec![]);

        let camera = Rc::new(RefCell::new(numeric::Rect::new(0.0, 0.0, 1366.0, 768.0)));

        let map_position = numeric::Point2f::new(800.0, 830.0);

        let player = PlayableCharacter::new(character_factory::create_character(
            character_factory::CharacterFactoryOrder::PlayableDoremy1,
            ctx,
            &camera.borrow(),
            map_position,
        ));

        let character = character_factory::create_character(
            character_factory::CharacterFactoryOrder::CustomerSample,
            ctx,
            &camera.borrow(),
            numeric::Point2f::new(1824.0, 1344.0),
        );
        let mut character_group = CharacterGroup::new();
        character_group.add(CustomerCharacter::new(
            ctx.resource,
            character,
            CustomerDestPoint::new(vec![
                numeric::Vector2u::new(10, 4),
                numeric::Vector2u::new(6, 4),
                //numeric::Vector2u::new(5, 14),
            ]),
        ));

        let mut map = MapData::new(ctx, map_id, camera.clone());
        map.tile_map.build_collision_map();

        let shop_time = ShopClock::new(8, 0);
        let drawble_shop_clock = DrawableShopClock::from_toml(
            ctx,
            "resources/other_config/shop_clock.toml",
            shop_time.clone(),
        );

        let mut result_report = ResultReport::new();
        for new_book in new_books.iter() {
            ctx.savable_data
                .task_result
                .not_shelved_books
                .push(new_book.clone());
            result_report.add_new_book_id(new_book.get_unique_id());
        }

        let mut delay_event_list = DelayEventList::new();
        delay_event_list.add_event(
            Box::new(move |slf: &mut ShopScene, ctx, t| {
                slf.shop_special_object
                    .show_new_books_viewer(ctx, new_books, t);
            }),
            31,
        );

        ShopScene {
            player: player,
            character_group: character_group,
            shop_special_object: ShopSpecialObject::new(),
            key_listener: key_listener,
            clock: 0,
            shop_clock: shop_time,
            map: map,
            event_list: delay_event_list,
            result_report: result_report,
            shop_menu: ShopMenuMaster::new(ctx, numeric::Vector2f::new(450.0, 768.0), 0),
            shop_map: MovableWrap::new(
                Box::new(ShopMapViewer::new(
                    ctx,
                    numeric::Rect::new(1366.0, 70.0, 1000.0, 628.0),
                    0,
                )),
                None,
                0,
            ),
            shop_map_is_staged: false,
            customer_request_queue: VecDeque::new(),
            customer_queue: CustomerQueue::new(0),
	    goto_check_customers: GoToCheckCustomers::new(numeric::Vector2u::new(15, 4), 0),
            dark_effect_panel: DarkEffectPanel::new(
                ctx.context,
                numeric::Rect::new(0.0, 0.0, 1366.0, 768.0),
                0,
            ),
            pause_screen_set: None,
            camera: camera,
            transition_scene: SceneID::SuzunaShop,
            transition_status: SceneTransition::Keep,
            scene_transition_effect: None,
            notification_area: NotificationArea::new(
                ctx,
                numeric::Point2f::new((crate::core::WINDOW_SIZE_X - 20) as f32, 20.0),
                0,
            ),
            begining_save_data: ctx.savable_data.clone(),
            drawable_shop_clock: drawble_shop_clock,
        }
    }

    ///
    /// カメラを動かすメソッド
    ///
    pub fn move_camera(&mut self, offset: numeric::Vector2f) {
        self.camera.borrow_mut().x += offset.x;
        self.camera.borrow_mut().y += offset.y;

        if offset.x != 0.0 || offset.y != 0.0 {
            self.map.tile_map.request_redraw();
            self.map.tile_map.request_updating_tile_batch();
        }
    }

    pub fn set_camera_x(&mut self, offset: f32) {
        self.camera.borrow_mut().x = offset;

        if offset != 0.0 {
            self.map.tile_map.request_redraw();
            self.map.tile_map.request_updating_tile_batch();
        }
    }

    pub fn set_camera_y(&mut self, offset: f32) {
        self.camera.borrow_mut().y = offset;

        if offset != 0.0 {
            self.map.tile_map.request_redraw();
            self.map.tile_map.request_updating_tile_batch();
        }
    }

    pub fn set_camera(&mut self, offset: numeric::Vector2f) {
        self.camera.borrow_mut().x = offset.x;
        self.camera.borrow_mut().y = offset.y;

        if offset.x != 0.0 || offset.y != 0.0 {
            self.map.tile_map.request_redraw();
            self.map.tile_map.request_updating_tile_batch();
        }
    }

    pub fn customer_move_and_collision_check(
        ctx: &mut ggez::Context,
        customer: &mut CustomerCharacter,
        camera: &numeric::Rect,
        tile_map: &mp::StageObjectMap,
        t: Clock,
    ) {
        customer.move_map_current_speed_y();

        // 当たり判定の前に描画位置を決定しないとバグる。この仕様も直すべき
        customer
            .get_mut_character_object()
            .update_display_position(camera);

        ShopScene::check_collision_vertical(ctx, customer.get_mut_character_object(), tile_map, t);
        customer
            .get_mut_character_object()
            .update_display_position(camera);

        customer.move_map_current_speed_x();
        customer
            .get_mut_character_object()
            .update_display_position(camera);
        ShopScene::check_collision_horizon(ctx, customer.get_mut_character_object(), tile_map, t);
        customer
            .get_mut_character_object()
            .update_display_position(camera);
    }

    fn camera_focus_character_x(&mut self) {
        let chara_pos = self.player.get_map_position();

        let x = if chara_pos.x <= 683.0 {
            0.0
        } else if chara_pos.x >= 853.0 {
            170.0
        } else {
            chara_pos.x - 683.0
        };

        self.set_camera_x(x);
    }

    fn camera_focus_character_y(&mut self) {
        let chara_pos = self.player.get_map_position();

        let y = if chara_pos.y <= 384.0 {
            0.0
        } else if chara_pos.y >= 1344.0 {
            960.0
        } else {
            chara_pos.y - 384.0
        };

        self.set_camera_y(y);
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

        self.camera_focus_character_x();
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
        self.camera_focus_character_y();
    }

    fn check_character_collision_x_sub(
        ctx: &mut ggez::Context,
        player: &mut MapObject,
        character: &mut MapObject,
        camera: &numeric::Rect,
        t: Clock,
    ) {
        // 他キャラクターのマップ座標を更新
        character
            .update_display_position(camera);

        // 衝突情報を取得
        let collision_info = player
            .check_collision_with_character(ctx, character);

        // collisionフィールドがtrueなら、衝突している
        if collision_info.collision {
            // プレイヤーと他キャラクターの衝突状況から、プレイヤーがどれだけ、突き放されればいいのか計算
            let diff =
                player
                    .fix_collision_horizon(ctx, &collision_info, t);

            // プレイヤーの突き放し距離分動かす
            player
                .move_map(numeric::Vector2f::new(-diff, 0.0));

            // プレイヤーのマップ座標を更新
            player
                .update_display_position(camera);
        }
    }

    fn check_character_collision_y_sub(
        ctx: &mut ggez::Context,
        player: &mut PlayableCharacter,
        character: &mut CustomerCharacter,
        camera: &numeric::Rect,
        t: Clock,
    ) {
        // 他キャラクターのマップ座標を更新
        character
            .get_mut_character_object()
            .update_display_position(camera);

        // 衝突情報を取得
        let collision_info = player
            .get_character_object()
            .check_collision_with_character(ctx, character.get_character_object());

        // collisionフィールドがtrueなら、衝突している
        if collision_info.collision {
            // プレイヤーと他キャラクターの衝突状況から、プレイヤーがどれだけ、突き放されればいいのか計算
            let diff =
                player
                    .get_mut_character_object()
                    .fix_collision_vertical(ctx, &collision_info, t);

            // プレイヤーの突き放し距離分動かす
            player
                .get_mut_character_object()
                .move_map(numeric::Vector2f::new(0.0, -diff));

            // プレイヤーのマップ座標を更新
            player
                .get_mut_character_object()
                .update_display_position(&camera);
        }
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
            Self::check_character_collision_x_sub(
                ctx,
                self.player.get_mut_character_object(),
                e.get_mut_character_object(),
                &self.camera.borrow(),
                t,
            );
        }

        for (e, _) in self.customer_queue.iter_mut() {
            Self::check_character_collision_x_sub(
                ctx,
                self.player.get_mut_character_object(),
                e.get_mut_character_object(),
                &self.camera.borrow(),
                t,
            );
        }

        // カメラをプレイヤーに合わせる
        self.camera_focus_character_x();
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
            Self::check_character_collision_y_sub(
                ctx,
                &mut self.player,
                e,
                &self.camera.borrow(),
                t,
            );
        }

        // 他キャラクターすべてとの衝突判定を行う
        for (e, _) in self.customer_queue.iter_mut() {
            Self::check_character_collision_y_sub(
                ctx,
                &mut self.player,
                e,
                &self.camera.borrow(),
                t,
            );
        }

        // カメラをプレイヤーに合わせる
        self.camera_focus_character_y();
    }

    fn move_playable_character_x(&mut self, ctx: &mut ggez::Context, t: Clock) {
        // プレイヤーのX方向の移動
        self.player.move_map_current_speed_x(
            ctx,
            numeric::Vector2f::new(0.0, self.map.tile_map.get_map_size().x),
        );

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
        self.player.move_map_current_speed_y(
            ctx,
            numeric::Vector2f::new(0.0, self.map.tile_map.get_map_size().y),
        );
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
        self.player.get_mut_character_object().update_texture(t);

        self.move_playable_character_x(ctx, t);
        self.move_playable_character_y(ctx, t);
    }

    pub fn run_builtin_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        builtin_event: BuiltinEvent,
    ) {
        match builtin_event.get_event_symbol() {
            BuiltinEventSymbol::SelectShelvingBook => {
                self.dark_effect_panel
                    .new_effect(8, self.get_current_clock(), 0, 200);
                self.shop_special_object.show_shelving_select_ui(
                    ctx,
                    self.player.get_shelving_book().clone(),
                    self.get_current_clock(),
                );
            }
        }
    }

    fn scene_transition_close_effect<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        self.scene_transition_effect = Some(effect_object::ScreenTileEffect::new(
            ctx,
            TileBatchTextureID::Shoji,
            numeric::Rect::new(
                0.0,
                0.0,
                crate::core::WINDOW_SIZE_X as f32,
                crate::core::WINDOW_SIZE_Y as f32,
            ),
            60,
            SceneTransitionEffectType::Close,
            TilingEffectType::WholeTile,
            -128,
            t,
        ));
    }

    fn run_event_panel_onmap_at<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        trigger: EventTrigger,
        map_position: numeric::Point2f,
    ) -> Option<EventTrigger> {
        let t = self.get_current_clock();
        let target_event =
            self.map
                .check_event_panel(trigger, map_position, self.get_current_clock());

        if let Some(event_element) = target_event {
            match event_element {
                MapEventElement::TextEvent(text) => {
                    println!("{}", text.get_text());

                    let mut scenario_box =
                        ScenarioBox::new(ctx, numeric::Rect::new(33.0, 470.0, 1300.0, 270.0), t);
                    scenario_box.text_box.set_fixed_text(
                        text.get_text(),
                        FontInformation::new(
                            ctx.resource.get_font(FontID::JpFude1),
                            numeric::Vector2f::new(32.0, 32.0),
                            ggraphics::Color::from_rgba_u32(0x000000ff),
                        ),
                    );
                    self.map.scenario_box = Some(scenario_box);
                }
                MapEventElement::SwitchScene(switch_scene) => {
                    if !self.customer_request_queue.is_empty() && !self.customer_queue.is_empty() {
                        let switch_scene_id = switch_scene.get_switch_scene_id();

                        self.event_list.add_event(
                            Box::new(move |slf: &mut ShopScene, ctx, _| {
                                slf.transition_status = SceneTransition::StackingTransition;
                                slf.transition_scene = switch_scene_id;

                                if slf.transition_scene == SceneID::MainDesk {
                                    slf.shop_clock.add_minute(10);
                                    slf.drawable_shop_clock.update_time(&slf.shop_clock);
                                }

                                let (mut customer, _) = slf.customer_queue.pop_head_customer().unwrap();

				customer.get_out_shop(
				    ctx.context,
				    &slf.map.tile_map,
				    numeric::Vector2u::new(15, 14)
				);
				
                                slf.character_group.add(customer);
                            }),
                            t + 31,
                        );

                        self.scene_transition_close_effect(ctx, t);
                    }
                }
                MapEventElement::BookStoreEvent(book_store_event) => {
                    self.dark_effect_panel
                        .new_effect(8, self.get_current_clock(), 0, 200);
                    self.shop_special_object.show_storing_select_ui(
                        ctx,
                        book_store_event.get_book_shelf_info().clone(),
                        self.player.get_shelving_book().clone(),
                        t,
                    );
                }
                MapEventElement::BuiltinEvent(builtin_event) => {
                    let builtin_event = builtin_event.clone();
                    self.run_builtin_event(ctx, builtin_event);
                }
            }

            return Some(trigger);
        }

        return None;
    }

    fn check_event_panel_onmap<'a>(&mut self, ctx: &mut SuzuContext<'a>, trigger: EventTrigger) {
        let map_position = self.player.get_center_map_position(ctx.context);
        let result = self.run_event_panel_onmap_at(ctx, trigger, map_position);

        if result.is_none() {
            let mut sub_map_position = map_position;
            let tile_size = self.map.get_tile_size();

            match self.player.get_character_object().current_direction() {
                ObjectDirection::Down => sub_map_position.y += tile_size.y,
                ObjectDirection::Up => sub_map_position.y -= tile_size.y,
                ObjectDirection::Right => sub_map_position.x += tile_size.x,
                ObjectDirection::Left => sub_map_position.x -= tile_size.x,
            }

            let _ = self.run_event_panel_onmap_at(ctx, trigger, sub_map_position);
        }
    }

    fn update_playable_character_texture(&mut self, rad: f32) {
        if rad >= 45.0_f32.to_radians() && rad < 135.0_f32.to_radians() {
            // 上向き
            self.player
                .get_mut_character_object()
                .change_animation_mode(ObjectDirection::Down);
        }

        if rad >= 135.0_f32.to_radians() && rad < 225.0_f32.to_radians() {
            // 左向き
            self.player
                .get_mut_character_object()
                .change_animation_mode(ObjectDirection::Left);
        }

        if rad >= 225.0_f32.to_radians() && rad < 315.0_f32.to_radians() {
            // 下向き
            self.player
                .get_mut_character_object()
                .change_animation_mode(ObjectDirection::Up);
        }

        if (rad >= 315.0_f32.to_radians() && rad <= 360.0_f32.to_radians())
            || (rad >= 0.0_f32.to_radians() && rad < 45.0_f32.to_radians())
        {
            // 右向き
            self.player
                .get_mut_character_object()
                .change_animation_mode(ObjectDirection::Right);
        }
    }

    pub fn start_mouse_move(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        let current = self.player.get_character_object().obj().get_center(ctx);

        let offset = numeric::Point2f::new(point.x - current.x, point.y - current.y);

        if offset.x == 0.0 && offset.y == 0.0 {
            return;
        }

        let d = (offset.x.powf(2.0) + offset.y.powf(2.0)).sqrt();
        let speed_k = if d > 300.0 { 300.0 } else { d } / 200.0;

        let rad = if offset.x >= 0.0 {
            if offset.y >= 0.0 {
                (offset.y / offset.x).atan()
            } else {
                (offset.y / offset.x).atan() + 360.0_f32.to_radians()
            }
        } else {
            (offset.y / offset.x).atan() + 180.0_f32.to_radians()
        };
        let speed = numeric::Vector2f::new(rad.cos() * 4.0 * speed_k, rad.sin() * 4.0 * speed_k);

        self.player.set_speed(speed);
        self.update_playable_character_texture(rad);
    }

    pub fn switched_and_restart<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        elapsed_clock: Clock,
        condition_eval_report: Option<BookConditionEvalReport>,
    ) {
        let t = self.get_current_clock();
        let animation_time = 30;

        self.transition_scene = SceneID::SuzunaShop;

        self.scene_transition_effect = Some(effect_object::ScreenTileEffect::new(
            ctx,
            TileBatchTextureID::Shoji,
            numeric::Rect::new(
                0.0,
                0.0,
                crate::core::WINDOW_SIZE_X as f32,
                crate::core::WINDOW_SIZE_Y as f32,
            ),
            animation_time,
            SceneTransitionEffectType::Open,
            TilingEffectType::WholeTile,
            -128,
            t,
        ));

        self.shop_clock.add_minute((elapsed_clock / 1000) as u8);
        if let Some(report) = condition_eval_report {
            self.result_report
                .add_condition_eval_mistakes(report.count_mistake());
        }

        // self.event_list.add_event(
        //     Box::new(move |slf: &mut ShopScene, _, _| { slf.scene_transition_effect = None; }),
        //     animation_time + 1
        // );
    }

    pub fn update_task_result<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        self.shop_menu
            .update_contents(ctx, self.player.get_shelving_book());
    }

    fn try_hide_shelving_select_ui<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        let select_result = self
            .shop_special_object
            .hide_shelving_select_ui(self.get_current_clock());
        if let Some((boxed, shelving)) = select_result {
            ctx.savable_data.task_result.not_shelved_books = boxed;
            self.player.update_shelving_book(shelving);
            self.shop_menu
                .update_contents(ctx, self.player.get_shelving_book());
            self.dark_effect_panel
                .new_effect(8, self.get_current_clock(), 200, 0);
        }
    }

    fn try_hide_storing_select_ui<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        let store_result = self
            .shop_special_object
            .hide_storing_select_ui(self.get_current_clock());
        if let Some((_stored, shelving)) = store_result {
            self.player.update_shelving_book(shelving);
            self.shop_menu
                .update_contents(ctx, self.player.get_shelving_book());
            self.dark_effect_panel
                .new_effect(8, self.get_current_clock(), 200, 0);
        }
    }

    pub fn pop_customer_request(&mut self) -> Option<CustomerRequest> {
        self.customer_request_queue.pop_front()
    }

    ///
    /// # 再描画要求有り
    ///
    pub fn update_shop_clock_regular<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        if self.get_current_clock() % 20 == 0 {
            self.shop_clock.add_minute(1);
            self.drawable_shop_clock.update_time(&self.shop_clock);

            if self.shop_clock.equals(12, 0) {
                self.notification_area.insert_new_contents_generic(
                    ctx,
                    NotificationContentsData::new(
                        "おしらせ".to_string(),
                        "十二時ヲ過ギマシタ".to_string(),
                        NotificationType::Time,
                    ),
                    t,
                );
            }

            ctx.process_utility.redraw();
        }
    }

    pub fn clone_begning_save_data(&self) -> SavableData {
        self.begining_save_data.clone()
    }

    pub fn clone_result_report(&self) -> ResultReport {
        self.result_report.clone()
    }

    ///
    /// # 再描画要求有り
    ///
    pub fn check_shop_clock_regular<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        if self.shop_clock.equals(18, 0) {
            self.event_list.add_event(
                Box::new(move |slf: &mut Self, ctx, _| {
                    // reportに未配架の本のIDをメモする
                    for book_info in ctx.savable_data.task_result.not_shelved_books.iter() {
                        slf.result_report
                            .add_yet_shelved_book_id(book_info.get_unique_id());
                    }

                    slf.transition_status = SceneTransition::SwapTransition;
                    slf.transition_scene = SceneID::DayResult;
                }),
                t + 120,
            );

            self.scene_transition_close_effect(ctx, t);
            ctx.process_utility.redraw();
        }
    }

    fn random_add_customer<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        if rand::random::<usize>() % 500 == 0 {
            let character = character_factory::create_character(
                character_factory::CharacterFactoryOrder::CustomerSample,
                ctx,
                &self.camera.borrow(),
                numeric::Point2f::new(1536.0, 1248.0),
            );
            self.character_group.add(CustomerCharacter::new(
                ctx.resource,
                character,
                CustomerDestPoint::new(vec![
                    numeric::Vector2u::new(10, 4),
                    numeric::Vector2u::new(6, 4),
                    //numeric::Vector2u::new(5, 14),
                ]),
            ));
        }
    }

    fn notify_customer_calling<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        let notification = Box::new(notify::GeneralNotificationContents::new(
            ctx,
            NotificationContentsData::new(
                "セラ知オ".to_string(),
                "御客ガ呼ンデイマス".to_string(),
                NotificationType::CustomerCalling,
            ),
            0,
        ));
        self.notification_area
            .insert_new_contents(ctx, notification, t);
    }

    fn transition_to_title_scene<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        self.event_list.add_event(
            Box::new(|slf: &mut Self, _, _| {
                slf.transition_status = SceneTransition::SwapTransition;
                slf.transition_scene = SceneID::Title;
            }),
            t + 60,
        );
        self.scene_transition_close_effect(ctx, t);
    }

    fn exit_pause_screen(&mut self, t: Clock) {
        self.dark_effect_panel.new_effect(8, t, 220, 0);
        self.pause_screen_set = None;
    }

    fn enter_pause_screen<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        self.dark_effect_panel.new_effect(8, t, 0, 220);
        self.player.reset_speed();
        self.pause_screen_set = Some(PauseScreenSet::new(ctx, 0));
    }

    fn now_paused(&self) -> bool {
        self.pause_screen_set.is_some()
    }

    fn pause_screen_click_handler<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        t: Clock,
    ) {
        let pause_screen_set = match self.pause_screen_set.as_ref() {
            Some(it) => it,
            _ => return,
        };

        if let Some(pause_result) = pause_screen_set.mouse_click_handler(ctx, point) {
            match pause_result {
                PauseResult::GoToTitle => self.transition_to_title_scene(ctx, t),
                PauseResult::ReleasePause => self.exit_pause_screen(t),
            }
        }
    }

    fn check_waiting_customer_giveup<'a>(&mut self, ctx: &mut SuzuContext<'a>, now: Clock) {
        let giveup_customers = self.customer_queue.drain_giveup_customers(now);

	if giveup_customers.len() > 0 {
	    self.goto_check_customers.reset_each_customers_goal(ctx, &self.map.tile_map, self.customer_queue.tail_map_position());
	}
	
        for mut customer in giveup_customers {
            customer.get_out_shop(
                ctx.context,
                &self.map.tile_map,
                numeric::Vector2u::new(15, 14),
            );
            self.character_group.add(customer);
        }
    }

    fn non_paused_key_down_event<'a>(&mut self, ctx: &mut SuzuContext<'a>, vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => {
                if let Some(scenario_box) = self.map.scenario_box.as_mut() {
                    if scenario_box.get_text_box_status() == TextBoxStatus::FixedText {
                        self.map.scenario_box = None;
                    }
                } else {
                    self.check_event_panel_onmap(ctx, EventTrigger::Action);
                }
            }
            tdev::VirtualKey::Action2 => {
                self.shop_menu.toggle_first_menu(self.get_current_clock());
                if self.shop_menu.first_menu_is_open() {
                    self.dark_effect_panel
                        .new_effect(8, self.get_current_clock(), 0, 200);
                } else {
                    self.dark_effect_panel
                        .new_effect(8, self.get_current_clock(), 200, 0);
                }
            }
            tdev::VirtualKey::Action3 => {
                self.try_hide_shelving_select_ui(ctx);
                self.try_hide_storing_select_ui(ctx);
            }
            tdev::VirtualKey::Action4 => {
                let t = self.get_current_clock();
                self.enter_pause_screen(ctx, t);
            }
            tdev::VirtualKey::Action5 => {
                // self.transition_status = SceneTransition::StackingTransition;
                // self.transition_scene = SceneID::MainDesk;
		if self.goto_check_customers.is_visible() {
		    self.goto_check_customers.hide();
		} else {
		    self.goto_check_customers.appear();
		}

		self.goto_check_customers.debug_print(&self.map.tile_map);
            }
            _ => (),
        }

        self.shop_menu
            .menu_key_action(vkey, self.get_current_clock());
    }

    fn try_add_goto_check_customers<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
	if t % 900 != 0 {
	    return;
	}
	
	if let Some(customer) = self.character_group.pickup_goto_check_customer() {
	    println!("picked up!!");
	    self.goto_check_customers.insert_new_customer(
		ctx,
		customer,
		&self.map.tile_map,
		self.customer_queue.tail_map_position()
	    );
	}
	
    }
}

impl SceneManager for ShopScene {
    fn key_down_event<'a>(&mut self, ctx: &mut SuzuContext<'a>, vkey: tdev::VirtualKey) {
        if self.now_paused() {
            match vkey {
                tdev::VirtualKey::Action4 => {
                    let t = self.get_current_clock();
                    self.exit_pause_screen(t);
                }
                _ => (),
            }
        } else {
            self.non_paused_key_down_event(ctx, vkey);
        }
    }

    fn mouse_motion_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        _offset: numeric::Vector2f,
    ) {
        if self.now_paused() {
            if let Some(pause_screen_set) = self.pause_screen_set.as_mut() {
                pause_screen_set.mouse_motion_handler(ctx, point);
            }
        } else {
            if ggez::input::mouse::button_pressed(ctx.context, MouseButton::Left) {
                self.start_mouse_move(ctx.context, point);
            }
        }
    }

    fn mouse_button_down_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        if self.now_paused() {
        } else {
            match button {
                MouseButton::Left => {
                    self.start_mouse_move(ctx.context, point);
                }
                MouseButton::Right => {
                    self.player.reset_speed();
                }
                _ => (),
            }

            self.shop_special_object.mouse_down_action(
                ctx,
                button,
                point,
                self.get_current_clock(),
            );
        }
    }

    fn mouse_button_up_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        if self.now_paused() {
            match button {
                MouseButton::Left => {
                    let t = self.get_current_clock();
                    self.pause_screen_click_handler(ctx, point, t);
                }
                _ => (),
            }

            return;
        } else {
            match button {
                MouseButton::Left => {
                    self.player.reset_speed();
                }
                MouseButton::Right => {
                    if self.shop_map_is_staged {
                        self.shop_map.override_move_func(
                            move_fn::devide_distance(numeric::Point2f::new(1366.0, 70.0), 0.25),
                            self.get_current_clock(),
                        );
                    } else {
                        self.shop_map.override_move_func(
                            move_fn::devide_distance(numeric::Point2f::new(158.0, 70.0), 0.25),
                            self.get_current_clock(),
                        );
                    }

                    self.shop_map_is_staged = !self.shop_map_is_staged;
                }
                _ => (),
            }
        }
    }

    fn mouse_wheel_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        x: f32,
        y: f32,
    ) {
        if self.now_paused() {
        } else {
            self.shop_special_object
                .mouse_wheel_scroll_action(ctx, point, x, y);
        }
    }

    fn pre_process<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        let t = self.get_current_clock();
	
        //println!("{}", perf_measure!({

	flush_delay_event_and_redraw_check!(self, self.event_list, ctx, t);

        if !self.shop_menu.first_menu_is_open()
            && !self.shop_special_object.is_enable_now()
            && !self.now_paused()
        {
            self.random_add_customer(ctx);
            self.move_playable_character(ctx.context, t);
            self.check_event_panel_onmap(ctx, EventTrigger::Touch);

            self.character_group.move_and_collision_check(
                ctx.context,
                &self.camera.borrow(),
                &self.map.tile_map,
                t,
            );

            let mut rising_customers = self
                .goto_check_customers
                .drain_remove_if(|customer: &CustomerCharacter| customer.is_wait_on_clerk());
	    if rising_customers.len() > 0 {
		self.goto_check_customers.reset_each_customers_goal(
		    ctx,
		    &self.map.tile_map,
		    self.customer_queue.tail_map_position()
		);
	    }

            // 新しく客が列に並んだら、通知をする
            if !rising_customers.is_empty() {
                self.notify_customer_calling(ctx, t);
            }

            for customer in &mut rising_customers {
                if let Some(request) = customer.check_rise_hand(ctx) {
                    self.customer_request_queue.push_back(request);
                }
            }

            for customer in rising_customers {
                self.customer_queue.push_back(customer, t);
            }

            self.check_waiting_customer_giveup(ctx, t);

            for customer in self.character_group.iter_mut() {
                customer.try_update_move_effect(
                    ctx,
                    &self.map.tile_map,
                    numeric::Vector2u::new(5, 14),
                    numeric::Vector2u::new(15, 14),
                    t,
                );
                customer.get_mut_character_object().update_texture(t);
            }

	    self.goto_check_customers.go_moving(ctx, &self.camera.borrow(), &self.map.tile_map, t);

            self.result_report
                .add_customers_waiting_time(self.customer_queue.len() as Clock);
            for (customer, _) in self.customer_queue.iter_mut() {
                Self::customer_move_and_collision_check(
                    ctx.context,
                    customer,
                    &self.camera.borrow(),
                    &self.map.tile_map,
                    t,
                );
                customer.try_update_move_effect(
                    ctx,
                    &self.map.tile_map,
                    numeric::Vector2u::new(5, 14),
                    numeric::Vector2u::new(15, 14),
                    t,
                );
                customer.get_mut_character_object().update_texture(t);
            }

            self.character_group.remove_if(|c| c.is_got_out());

            self.character_group.sort_by_y_position();

	    self.try_add_goto_check_customers(ctx, t);
	    

            // マップ描画の準備
            self.map.tile_map.update(ctx.context, t);

            self.shop_map.move_with_func(t);

            // 時刻の更新
            self.update_shop_clock_regular(ctx, t);
            self.check_shop_clock_regular(ctx, t);

            ctx.process_utility.redraw();
        }

        // 通知の更新
        self.notification_area.update(ctx, t);

        // 暗転の描画
        self.dark_effect_panel.run_effect(ctx, t);

        if let Some(transition_effect) = self.scene_transition_effect.as_mut() {
            transition_effect.effect(ctx.context, t);
            ctx.process_utility.redraw();
        }

        self.shop_special_object.update(ctx, t);

        // メニューの更新
        self.shop_menu.update(ctx, t);

	//}));
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.map.tile_map.draw(ctx).unwrap();

        let mut map_obj_drawer = MapObjectDrawer::new();

        map_obj_drawer.add(&mut self.player);

        for customer in self.character_group.iter_mut() {
            if customer
                .get_character_object()
                .obj()
                .get_drawing_area(ctx)
                .overlaps(&numeric::Rect::new(0.0, 0.0, 1366.0, 768.0))
            {
                map_obj_drawer.add(customer);
            }
        }

        for (customer, _) in self.customer_queue.iter_mut() {
            if customer
                .get_character_object()
                .obj()
                .get_drawing_area(ctx)
                .overlaps(&numeric::Rect::new(0.0, 0.0, 1366.0, 768.0))
            {
                map_obj_drawer.add(customer);
            }
        }

	self.goto_check_customers.draw(ctx).unwrap();

        map_obj_drawer.sort(ctx);
        map_obj_drawer.draw(ctx);

        if let Some(scenario_box) = self.map.scenario_box.as_mut() {
            scenario_box.draw(ctx).unwrap();
        }

        self.shop_map.draw(ctx).unwrap();

        self.drawable_shop_clock.draw(ctx).unwrap();

        self.dark_effect_panel.draw(ctx).unwrap();

        self.shop_special_object.draw(ctx).unwrap();
        self.shop_menu.draw(ctx).unwrap();

        self.notification_area.draw(ctx).unwrap();

        if let Some(pause_screen) = self.pause_screen_set.as_mut() {
            pause_screen.draw(ctx).unwrap();
        }

        if let Some(transition_effect) = self.scene_transition_effect.as_mut() {
            transition_effect.draw(ctx).unwrap();
        }
    }

    fn post_process<'a>(&mut self, _ctx: &mut SuzuContext<'a>) -> SceneTransition {
        self.update_current_clock();
        self.transition_status
    }

    fn unfocus_event<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
	let t = self.get_current_clock();
	//self.enter_pause_screen(ctx, t);
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
