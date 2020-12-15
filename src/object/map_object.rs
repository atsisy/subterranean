use std::collections::HashMap;
use std::collections::VecDeque;
use std::rc::Rc;
use std::str::FromStr;

use ggez::graphics as ggraphics;

use torifune::core::Clock;
use torifune::debug;
use torifune::distance;
use torifune::graphics::drawable::*;
use torifune::graphics::object::*;
use torifune::numeric;

use crate::core::map_parser as mp;
use crate::core::*;
use crate::flush_delay_event;
use crate::object::collision::*;
use crate::object::task_object::tt_main_component::CustomerRequest;
use crate::object::util_object::*;
use crate::scene::{DelayEventList, SceneID};

pub struct TwoStepPoint {
    pub previous: numeric::Point2f,
    pub current: numeric::Point2f,
}

impl TwoStepPoint {
    pub fn diff(&self) -> numeric::Vector2f {
        self.current - self.previous
    }

    pub fn update(&mut self, pos: numeric::Point2f) {
        self.previous = self.current;
        self.current = pos;
    }

    pub fn move_diff(&mut self, pos: &numeric::Vector2f) {
        self.previous = self.current;
        self.current += *pos;
    }
}

///
/// マップ上に描画されるオブジェクトが実装すべきトレイト
///
pub trait OnMap: DrawableComponent {
    // マップ上のテクスチャ描画開始地点を返す
    fn get_map_position(&self) -> numeric::Point2f;

    // マップ上のテクスチャ描画領域の右下の位置を返す
    fn get_map_position_bottom_right(&self, ctx: &mut ggez::Context) -> numeric::Point2f;

    // マップ上のテクスチャ描画開始地点を設定する
    fn set_map_position(&mut self, position: numeric::Point2f);
}

///
/// マップ上に描画するオブジェクト
/// 基本的に、マップ上に描画するオブジェクトはこの構造体を移譲して使う
///
pub struct MapObject {
    last_position: numeric::Point2f,
    object: TextureAnimation,
    speed_info: TextureSpeedInfo,
    map_position: TwoStepPoint,
    collision_crop: numeric::Rect,
}

impl MapObject {
    pub fn new(
        obj: SimpleObject,
        mode_order: Vec<ObjectDirection>,
        textures: Vec<Vec<Rc<ggraphics::Image>>>,
        mode: ObjectDirection,
        speed_info: TextureSpeedInfo,
        map_position: numeric::Point2f,
        collision_crop: numeric::Rect,
        frame_speed: Clock,
    ) -> MapObject {
        MapObject {
            last_position: obj.get_position(),
            map_position: TwoStepPoint {
                previous: map_position,
                current: map_position,
            },
            speed_info: speed_info,
            object: TextureAnimation::new(obj, mode_order, textures, mode, frame_speed),
            collision_crop: collision_crop,
        }
    }

    pub fn current_direction(&self) -> ObjectDirection {
        self.object.get_current_mode()
    }

    ///
    /// 当たり判定のある領域を返すメソッド
    ///
    pub fn get_collision_area(&self, ctx: &mut ggez::Context) -> numeric::Rect {
        let croppped_size = self.get_collision_size(ctx);
        let collision_top_offset = self.get_collision_top_offset(ctx);
        let position = self.obj().get_position();

        numeric::Rect::new(
            position.x + collision_top_offset.x,
            position.y + collision_top_offset.y,
            croppped_size.x,
            croppped_size.y,
        )
    }

    ///
    /// 当たり判定のある領域のサイズを返すメソッド
    ///
    pub fn get_collision_size(&self, ctx: &mut ggez::Context) -> numeric::Vector2f {
        let drawing_size = self.obj().get_drawing_size(ctx);

        numeric::Vector2f::new(
            drawing_size.x * (self.collision_crop.w - self.collision_crop.x),
            drawing_size.y * (self.collision_crop.h - self.collision_crop.y),
        )
    }

    ///
    /// 当たり判定のある領域のテクスチャ内のオフセット
    ///
    fn get_collision_top_offset(&self, ctx: &mut ggez::Context) -> numeric::Vector2f {
        let drawing_size = self.obj().get_drawing_size(ctx);

        numeric::Vector2f::new(
            drawing_size.x * self.collision_crop.x,
            drawing_size.y * self.collision_crop.y,
        )
    }

    pub fn speed_info(&self) -> &TextureSpeedInfo {
        &self.speed_info
    }

    pub fn speed_info_mut(&mut self) -> &mut TextureSpeedInfo {
        &mut self.speed_info
    }

    ///
    /// アニメーションモードを変更するメソッド
    ///
    pub fn change_animation_mode(&mut self, mode: ObjectDirection) {
        self.object.change_mode(mode, AnimationType::Loop, mode);
    }

    pub fn obj(&self) -> &SimpleObject {
        self.object.get_object()
    }

    pub fn obj_mut(&mut self) -> &mut SimpleObject {
        self.object.get_mut_object()
    }

    pub fn get_last_position(&self) -> numeric::Point2f {
        self.last_position
    }

    pub fn undo_move(&mut self) {
        let last = self.get_last_position();
        self.object.get_mut_object().set_position(last);
    }

    fn get_last_move_distance(&self) -> numeric::Vector2f {
        let current = self.object.get_object().get_position();
        numeric::Vector2f::new(
            current.x - self.last_position.x,
            current.y - self.last_position.y,
        )
    }

    pub fn get_last_map_move_distance(&self) -> numeric::Vector2f {
        self.map_position.diff()
    }

    ///
    /// 当たり判定領域を基準としてマップ座標を更新する
    ///
    pub fn set_map_position_with_collision_top_offset(
        &mut self,
        ctx: &mut ggez::Context,
        position: numeric::Point2f,
    ) {
        let offset = self.get_collision_top_offset(ctx);
        self.map_position.update(position - offset);
    }

    ///
    /// 当たり判定領域を基準としたマップ座標を返す
    ///
    pub fn get_map_position_with_collision_top_offset(
        &self,
        ctx: &mut ggez::Context,
    ) -> numeric::Point2f {
        let offset = self.get_collision_top_offset(ctx);
        self.map_position.current + offset
    }

    ///
    /// キャラクタテクスチャの上側が衝突した場合
    /// どれだけ、テクスチャを移動させれば良いのかを返す
    ///
    fn fix_collision_above(
        &mut self,
        _ctx: &mut ggez::Context,
        info: &CollisionInformation,
        _: Clock,
    ) -> f32 {
        (info.object1_position.unwrap().y + info.object1_position.unwrap().h + 0.01)
            - info.object2_position.unwrap().y
    }

    ///
    /// キャラクタテクスチャの下側が衝突した場合
    /// どれだけ、テクスチャを移動させれば良いのかを返す
    ///
    fn fix_collision_bottom(
        &mut self,
        ctx: &mut ggez::Context,
        info: &CollisionInformation,
        _: Clock,
    ) -> f32 {
        let area = self.get_collision_size(ctx);
        info.object1_position.unwrap().y - (info.object2_position.unwrap().y + area.y) - 0.01
    }

    ///
    /// キャラクタテクスチャの右側が衝突した場合
    /// どれだけ、テクスチャを移動させれば良いのかを返す
    ///
    fn fix_collision_right(
        &mut self,
        ctx: &mut ggez::Context,
        info: &CollisionInformation,
        _: Clock,
    ) -> f32 {
        let area = self.get_collision_size(ctx);
        (info.object1_position.unwrap().x - 0.01) - (info.object2_position.unwrap().x + area.x)
    }

    ///
    /// キャラクタテクスチャの左側が衝突した場合
    /// どれだけ、テクスチャを移動させれば良いのかを返す
    ///
    fn fix_collision_left(
        &mut self,
        _ctx: &mut ggez::Context,
        info: &CollisionInformation,
        _t: Clock,
    ) -> f32 {
        (info.object1_position.unwrap().x + info.object1_position.unwrap().w + 0.01)
            - info.object2_position.unwrap().x
    }

    ///
    /// 垂直方向の衝突（めり込み）を修正するメソッド
    ///
    pub fn fix_collision_vertical(
        &mut self,
        ctx: &mut ggez::Context,
        info: &CollisionInformation,
        t: Clock,
    ) -> f32 {
        if info.center_diff.unwrap().y < 0.0 {
            return self.fix_collision_bottom(ctx, &info, t);
        } else if info.center_diff.unwrap().y > 0.0 {
            return self.fix_collision_above(ctx, &info, t);
        }

        0.0
    }

    ///
    /// 水平方向の衝突（めり込み）を修正するメソッド
    ///
    pub fn fix_collision_horizon(
        &mut self,
        ctx: &mut ggez::Context,
        info: &CollisionInformation,
        t: Clock,
    ) -> f32 {
        if info.center_diff.unwrap().x < 0.0 {
            return self.fix_collision_right(ctx, &info, t);
        } else if info.center_diff.unwrap().x > 0.0 {
            return self.fix_collision_left(ctx, &info, t);
        }

        0.0
    }

    ///
    /// テクスチャを更新する。（アニメーション）
    ///
    pub fn update_texture(&mut self, t: Clock) {
        self.object.try_next_frame(t);
    }

    ///
    /// マップ上の座標を動かす
    ///
    pub fn move_map(&mut self, offset: numeric::Vector2f) {
        self.map_position.move_diff(&offset);
    }

    ///
    /// マップ上の座標から、ディスプレイの描画位置を算出し、更新する
    ///
    pub fn update_display_position(&mut self, camera: &numeric::Rect) {
        let dp = mp::map_to_display(&self.map_position.current, camera);
        self.object.get_mut_object().set_position(dp);
    }

    ///
    /// キャラクター同士の衝突情報を計算する
    ///
    pub fn check_collision_with_character(
        &self,
        ctx: &mut ggez::Context,
        chara: &MapObject,
    ) -> CollisionInformation {
        let a1 = self.get_collision_area(ctx);
        let a2 = chara.get_collision_area(ctx);

        if a1.overlaps(&a2) {
            CollisionInformation::new_collision(
                a1,
                a2,
                numeric::Vector2f::new(a2.x - a1.x, a2.y - a1.y),
            )
        } else {
            CollisionInformation::new_not_collision()
        }
    }
}

impl DrawableComponent for MapObject {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.obj_mut().draw(ctx).unwrap();
        }
        Ok(())
    }

    fn hide(&mut self) {
        self.obj_mut().hide()
    }

    fn appear(&mut self) {
        self.obj_mut().appear()
    }

    fn is_visible(&self) -> bool {
        self.obj().is_visible()
    }

    fn set_drawing_depth(&mut self, depth: i8) {
        self.obj_mut().set_drawing_depth(depth)
    }

    fn get_drawing_depth(&self) -> i8 {
        self.obj().get_drawing_depth()
    }
}

impl OnMap for MapObject {
    // マップ上のテクスチャ描画開始地点を返す
    fn get_map_position(&self) -> numeric::Point2f {
        self.map_position.current
    }

    // マップ上のテクスチャ描画領域の右下の位置を返す
    fn get_map_position_bottom_right(&self, ctx: &mut ggez::Context) -> numeric::Point2f {
        self.get_map_position() + self.obj().get_drawing_size(ctx)
    }

    // マップ上のテクスチャ描画開始地点を設定する
    fn set_map_position(&mut self, position: numeric::Point2f) {
        self.map_position.update(position);
    }
}

///
/// 操作キャラクターの情報を保持
///
pub struct PlayableCharacter {
    character: MapObject,
    shelving_book: Vec<BookInformation>,
}

impl PlayableCharacter {
    pub fn new(character: MapObject) -> Self {
        PlayableCharacter {
            character: character,
            shelving_book: Vec::new(),
        }
    }

    pub fn get_center_map_position(&self, ctx: &mut ggez::Context) -> numeric::Point2f {
        let drawing_size = self.character.obj().get_drawing_size(ctx);
        self.get_map_position() + numeric::Vector2f::new(drawing_size.x / 2.0, drawing_size.y / 2.0)
    }

    pub fn get_character_object(&self) -> &MapObject {
        &self.character
    }

    pub fn get_mut_character_object(&mut self) -> &mut MapObject {
        &mut self.character
    }

    pub fn get_shelving_book(&self) -> &Vec<BookInformation> {
        &self.shelving_book
    }

    pub fn get_shelving_book_mut(&mut self) -> &mut Vec<BookInformation> {
        &mut self.shelving_book
    }

    pub fn update_shelving_book(&mut self, shelving_book: Vec<BookInformation>) {
        self.shelving_book = shelving_book;
    }

    pub fn fix_collision_horizon(
        &mut self,
        ctx: &mut ggez::Context,
        info: &CollisionInformation,
        t: Clock,
    ) -> f32 {
        self.character.fix_collision_horizon(ctx, info, t)
    }

    pub fn fix_collision_vertical(
        &mut self,
        ctx: &mut ggez::Context,
        info: &CollisionInformation,
        t: Clock,
    ) -> f32 {
        self.character.fix_collision_vertical(ctx, info, t)
    }

    pub fn move_map(&mut self, offset: numeric::Vector2f) {
        self.character.move_map(offset);
    }

    pub fn move_map_current_speed_x(&mut self, ctx: &mut ggez::Context, border: numeric::Vector2f) {
        let x_speed = self.get_character_object().speed_info().get_speed().x;
        let current_position = self.get_map_position();
        let next_position = self.get_map_position().x + x_speed;
        let drawing_size = self.character.obj().get_drawing_size(ctx);
        let right_border = border.y - drawing_size.x;

        if next_position < border.x {
            self.character
                .set_map_position(numeric::Point2f::new(border.x, current_position.y));
        } else if next_position > right_border {
            self.character
                .set_map_position(numeric::Point2f::new(right_border, current_position.y));
        } else {
            self.move_map(numeric::Vector2f::new(x_speed, 0.0))
        }
    }

    pub fn move_map_current_speed_y(&mut self, ctx: &mut ggez::Context, border: numeric::Vector2f) {
        let y_speed = self.get_character_object().speed_info().get_speed().y;
        let current_position = self.get_map_position();
        let next_position = self.get_map_position().y + y_speed;
        let drawing_size = self.character.obj().get_drawing_size(ctx);
        let bottom_border = border.y - drawing_size.y;

        if next_position < border.x {
            self.character
                .set_map_position(numeric::Point2f::new(current_position.x, border.x));
        } else if next_position > bottom_border {
            self.character
                .set_map_position(numeric::Point2f::new(current_position.x, bottom_border));
        } else {
            self.move_map(numeric::Vector2f::new(0.0, y_speed))
        }
    }

    pub fn get_speed(&self) -> numeric::Vector2f {
        self.character.speed_info().get_speed()
    }

    pub fn set_speed(&mut self, speed: numeric::Vector2f) {
        self.character.speed_info_mut().set_speed(speed);
    }

    pub fn set_speed_x(&mut self, speed: f32) {
        self.character.speed_info_mut().set_speed_x(speed);
    }

    pub fn set_speed_y(&mut self, speed: f32) {
        self.character.speed_info_mut().set_speed_y(speed);
    }

    pub fn reset_speed(&mut self) {
        self.character
            .speed_info_mut()
            .set_speed(numeric::Vector2f::new(0.0, 0.0));
    }
}

impl DrawableComponent for PlayableCharacter {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.get_mut_character_object().obj_mut().draw(ctx).unwrap();
        }
        Ok(())
    }

    fn hide(&mut self) {
        self.get_mut_character_object().hide()
    }

    fn appear(&mut self) {
        self.get_mut_character_object().appear()
    }

    fn is_visible(&self) -> bool {
        self.get_character_object().is_visible()
    }

    fn set_drawing_depth(&mut self, depth: i8) {
        self.get_mut_character_object().set_drawing_depth(depth)
    }

    fn get_drawing_depth(&self) -> i8 {
        self.get_character_object().get_drawing_depth()
    }
}

impl OnMap for PlayableCharacter {
    // マップ上のテクスチャ描画開始地点を返す
    fn get_map_position(&self) -> numeric::Point2f {
        self.character.get_map_position()
    }

    // マップ上のテクスチャ描画領域の右下の位置を返す
    fn get_map_position_bottom_right(&self, ctx: &mut ggez::Context) -> numeric::Point2f {
        self.character.get_map_position_bottom_right(ctx)
    }

    // マップ上のテクスチャ描画開始地点を設定する
    fn set_map_position(&mut self, position: numeric::Point2f) {
        self.character.set_map_position(position);
    }
}

pub struct CustomerDestPoint {
    candidates: Vec<numeric::Vector2u>,
}

impl CustomerDestPoint {
    pub fn new(candidates: Vec<numeric::Vector2u>) -> Self {
        CustomerDestPoint {
            candidates: candidates,
        }
    }

    pub fn random_select(&self) -> numeric::Vector2u {
        let random_index = rand::random::<usize>() % self.candidates.len();
        *self.candidates.get(random_index).unwrap()
    }
}

#[derive(Debug)]
pub struct CustomerMoveQueue {
    queue: VecDeque<numeric::Point2f>,
}

impl CustomerMoveQueue {
    pub fn new() -> Self {
        CustomerMoveQueue {
            queue: VecDeque::new(),
        }
    }

    pub fn enqueue(&mut self, points: Vec<numeric::Point2f>) {
        self.queue.extend(points);
    }

    pub fn dequeue(&mut self) -> Option<numeric::Point2f> {
        self.queue.pop_front()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CustomerCharacterStatus {
    Ready = 0,
    Moving,
    GoToCheck,
    WaitOnClerk,
    WaitOnBookShelf,
    GotOut,
    GettingOut,
}

pub struct CustomerInformation {
    pub name: String,
}

impl CustomerInformation {
    pub fn new(name: &str) -> Self {
        CustomerInformation {
            name: name.to_string(),
        }
    }
}

///
/// マップ上に表示するキャラクターの情報
///
pub struct CustomerCharacter {
    event_list: DelayEventList<Self>,
    character: MapObject,
    move_data: CustomerDestPoint,
    move_queue: CustomerMoveQueue,
    customer_status: CustomerCharacterStatus,
    shopping_is_done: bool,
    current_goal: numeric::Point2f,
    customer_info: CustomerInformation,
}

impl CustomerCharacter {
    pub fn new(
        game_data: &GameResource,
        character: MapObject,
        move_data: CustomerDestPoint,
    ) -> Self {
        CustomerCharacter {
            event_list: DelayEventList::new(),
            character: character,
            move_data: move_data,
            move_queue: CustomerMoveQueue::new(),
            customer_status: CustomerCharacterStatus::Ready,
            shopping_is_done: false,
            current_goal: numeric::Point2f::new(0.0, 0.0),
            customer_info: CustomerInformation::new(game_data.customer_random_select()),
        }
    }

    ///
    /// 現在のマップ位置から、指定された目的地までのルートを計算するメソッド
    ///
    pub fn find_route(
        &mut self,
        ctx: &mut ggez::Context,
        map_data: &mp::StageObjectMap,
        dest: numeric::Vector2u,
    ) -> Option<Vec<numeric::Point2f>> {
        // すでに過去の目的地に到達している場合のみ新たなルートを計算する
        if self.move_queue.empty() {
            // 現在のマップ位置がどこのタイルなのかを計算する
            let maybe_start = map_data.map_position_to_tile_position(
                self.character
                    .get_map_position_with_collision_top_offset(ctx),
            );

            // マップ位置が不正であった場合は、検索を行わない
            if let Some(map_start_pos) = maybe_start {
                // ルートを計算し、返す
                let maybe_route = map_data.find_shortest_route(map_start_pos, dest);
                if let Some(route) = maybe_route {
                    return Some(
                        route
                            .iter()
                            .map(|tp| map_data.tile_position_to_map_position(*tp))
                            .collect(),
                    );
                }
            }
        }

        None
    }

    ///
    /// 進行方向に応じて、アニメーションを変更する
    ///
    fn update_animation_mode_with_rad(&mut self, rad: f32) {
        if rad >= 45.0_f32.to_radians() && rad < 135.0_f32.to_radians() {
            // 上向き
            self.get_mut_character_object()
                .change_animation_mode(ObjectDirection::Down);
        } else if rad >= 135.0_f32.to_radians() && rad < 225.0_f32.to_radians() {
            // 左向き
            self.get_mut_character_object()
                .change_animation_mode(ObjectDirection::Left);
        } else if rad >= 225.0_f32.to_radians() && rad < 315.0_f32.to_radians() {
            // 下向き
            self.get_mut_character_object()
                .change_animation_mode(ObjectDirection::Up);
        } else {
            // 右向き
            self.get_mut_character_object()
                .change_animation_mode(ObjectDirection::Right);
        }
    }

    ///
    /// ゴールへの適切な速度を計算・更新する
    ///
    fn override_move_effect(&mut self, ctx: &mut ggez::Context, goal_point: numeric::Point2f) {
        // 現在のマップ位置
        let current = self
            .get_character_object()
            .get_map_position_with_collision_top_offset(ctx);

        // ゴールとのオフセット
        let offset = numeric::Point2f::new(goal_point.x - current.x, goal_point.y - current.y);

        // ゴールへのオフセットから、速度を算出
        let speed = if offset.x == 0.0 && offset.y == 0.0 {
            numeric::Vector2f::new(0.0, 0.0)
        } else {
            // 角度を計算
            let rad = if offset.x >= 0.0 {
                if offset.y >= 0.0 {
                    (offset.y / offset.x).atan()
                } else {
                    (offset.y / offset.x).atan() + 360.0_f32.to_radians()
                }
            } else {
                (offset.y / offset.x).atan() + 180.0_f32.to_radians()
            };

            // 基本的な速さは一致するようにしたいため、次のように計算する
            let speed = numeric::Vector2f::new(rad.cos() * 1.4, rad.sin() * 1.4);

            debug::debug_screen_push_text(&format!("rad: {}", rad.to_degrees()));

            // 向きによってアニメーションを更新
            self.update_animation_mode_with_rad(rad);

            speed
        };

        debug::debug_screen_push_text(&format!(
            "goal: {}:{}, speed: {}:{}",
            goal_point.x, goal_point.y, speed.x, speed.y
        ));

        // スピードを更新
        self.character.speed_info_mut().set_speed(speed);
    }

    fn goto_other_book_shelf_now(
        &mut self,
        ctx: &mut ggez::Context,
        map_data: &mp::StageObjectMap,
        t: Clock,
    ) {
        let goal = self.move_data.random_select();
        self.determine_next_goal(ctx, map_data, goal, t)
    }

    fn determine_next_goal(
        &mut self,
        ctx: &mut ggez::Context,
        map_data: &mp::StageObjectMap,
        goal: numeric::Vector2u,
        t: Clock,
    ) {
        // ルート検索
        let maybe_next_route = self.find_route(ctx, map_data, goal);

        debug::debug_screen_push_text(&format!("{:?}", maybe_next_route));

        // 一定時間後にルートを設定し、状態をReadyに変更する。
        // 移動開始するまでは、ストップ
        self.event_list.add_event(
            Box::new(move |customer, _, _| {
                if let Some(next_route) = maybe_next_route {
                    customer.move_queue.enqueue(next_route);
                    customer.customer_status = CustomerCharacterStatus::Ready;
                }
            }),
            t + 100,
        );

        self.customer_status = CustomerCharacterStatus::WaitOnBookShelf;
    }

    ///
    /// 移動速度を更新する
    ///
    fn update_move_effect(
        &mut self,
        ctx: &mut ggez::Context,
        map_data: &mp::StageObjectMap,
        t: Clock,
    ) {
        // 移動情報キューが空（目的地に到達してる or 初めて目的地を設定する）
        if self.move_queue.empty() {
            self.goto_other_book_shelf_now(ctx, map_data, t);
            return ();
        }

        // キューが空ではない場合
        // 情報をキューから取り出し、速度を計算し直す
        let maybe_next_position = self.move_queue.dequeue();
        if let Some(next_position) = maybe_next_position {
            debug::debug_screen_push_text(&format!("next: {:?}", next_position));
            self.override_move_effect(ctx, next_position);
            self.current_goal = next_position;
            self.customer_status = CustomerCharacterStatus::Moving;
        }
    }

    ///
    /// 目的地を強制的に上書きし設定するメソッド
    ///
    pub fn set_destination_forced(
        &mut self,
        ctx: &mut ggez::Context,
        map_data: &mp::StageObjectMap,
        dest: numeric::Vector2u,
    ) -> Result<(), ()> {
        // 現在の移動キューをクリア
        self.move_queue.clear();
        // 新しくルートを検索
        let maybe_next_route = self.find_route(ctx, map_data, dest);

        debug::debug_screen_push_text(&format!("{:?}", maybe_next_route));

        // ルートが見つかれば、その情報をキューに追加
        if let Some(next_route) = maybe_next_route {
            self.move_queue.enqueue(next_route);
            self.customer_status = CustomerCharacterStatus::Ready;
            Ok(())
        } else {
            println!(
                "failed, collision_top: {:?}, start -> {:?}, point -> {:?}, dest -> {:?}",
                self.character
                    .get_map_position_with_collision_top_offset(ctx),
                map_data.map_position_to_tile_position(
                    self.character
                        .get_map_position_with_collision_top_offset(ctx),
                ),
                self.character
                    .get_map_position_with_collision_top_offset(ctx),
                dest
            );
            Err(())
        }
    }

    pub fn get_out_shop(
        &mut self,
        ctx: &mut ggez::Context,
        map_data: &mp::StageObjectMap,
        dest: numeric::Vector2u,
    ) {
        match self.set_destination_forced(ctx, map_data, dest) {
            Ok(_) => self.customer_status = CustomerCharacterStatus::GettingOut,
            Err(_) => panic!("Failed to find route"),
        }
    }

    pub fn goto_check(
        &mut self,
        ctx: &mut ggez::Context,
        map_data: &mp::StageObjectMap,
        dest: numeric::Vector2u,
    ) {
        match self.set_destination_forced(ctx, map_data, dest) {
            Ok(_) => {
                self.customer_status = CustomerCharacterStatus::GoToCheck;
            }
            Err(_) => panic!("Failed to find route"),
        }
    }

    pub fn reset_speed(&mut self) {
        self.character
            .speed_info_mut()
            .set_speed(numeric::Vector2f::new(0.0, 0.0));
    }

    fn is_goal_now(&mut self, ctx: &mut ggez::Context) -> bool {
        let current = self
            .get_character_object()
            .get_map_position_with_collision_top_offset(ctx);
        distance!(current, self.current_goal) < 1.5
    }

    fn generate_hold_request<'a>(&mut self, ctx: &mut SuzuContext<'a>) -> CustomerRequest {
        let random_select = rand::random::<usize>() % 2;
        let today = ctx.savable_data.date.clone();

        match random_select {
            0 => CustomerRequest::Borrowing(BorrowingInformation::new(
                vec![ctx.resource.book_random_select().clone()],
                &self.customer_info.name,
                today,
                RentalLimit::random(),
            )),
            _ => CustomerRequest::Returning(ReturnBookInformation::new_random(
                ctx.resource,
                today,
                GensoDate::new(128, 12, 20),
            )),
        }
    }

    fn check_been_counter(
        &mut self,
        map_data: &mp::StageObjectMap,
        current_pos: numeric::Point2f,
        counter: numeric::Vector2u,
    ) {
        // 目的地がカウンターに設定されていた場合は、待機状態へ移行
        if !self.shopping_is_done
            && map_data.map_position_to_tile_position(current_pos).unwrap() == counter
        {
            self.character.change_animation_mode(ObjectDirection::Left);
            self.customer_status = CustomerCharacterStatus::WaitOnClerk;
            self.shopping_is_done = true;
        }
    }

    fn check_get_out(
        &mut self,
        map_data: &mp::StageObjectMap,
        current_pos: numeric::Point2f,
        exit: numeric::Vector2u,
    ) {
        // 目的地が出口に設定されていた場合は、待機状態へ移行
        if self.shopping_is_done
            && map_data.map_position_to_tile_position(current_pos).unwrap() == exit
        {
            self.customer_status = CustomerCharacterStatus::GotOut;
        }
    }

    pub fn is_got_out(&self) -> bool {
        self.customer_status == CustomerCharacterStatus::GotOut
    }

    ///
    /// 移動速度の更新が必要であれば行うメソッド
    ///
    pub fn try_update_move_effect<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        map_data: &mp::StageObjectMap,
        counter: numeric::Vector2u,
        exit: numeric::Vector2u,
        t: Clock,
    ) {
        // 遅延イベントを実行
        flush_delay_event!(self, self.event_list, ctx, t);

        match self.customer_status {
            CustomerCharacterStatus::Ready => {
                // 移動可能状態であれば、移動を開始する
                self.update_move_effect(ctx.context, map_data, t);
            }
            CustomerCharacterStatus::Moving => {
                // 移動中, 目的地に到着したか？
                if self.is_goal_now(ctx.context) {
                    let goal = self.current_goal;

                    debug::debug_screen_push_text(&format!("goal: {:?}", goal));

                    // 目的地でマップ位置を上書き
                    self.get_mut_character_object()
                        .set_map_position_with_collision_top_offset(ctx.context, goal);

                    // 移動可能状態に変更
                    self.customer_status = CustomerCharacterStatus::Ready;

                    // 速度もリセット
                    self.reset_speed();

                    // カウンターに到達したかチェック
                    self.check_been_counter(map_data, goal, counter);

                    // カウンターに到達したかチェック
                    self.check_get_out(map_data, goal, exit);
                }
            }
            CustomerCharacterStatus::WaitOnClerk => {}
            CustomerCharacterStatus::GettingOut => {
                if !self.is_goal_now(ctx.context) {
                    return;
                }

                let goal = self.current_goal;

                // 目的地でマップ位置を上書き
                self.get_mut_character_object()
                    .set_map_position_with_collision_top_offset(ctx.context, goal);

                // 店の出入口に到達したかチェック
                self.check_get_out(map_data, goal, exit);

                // GotOutなら、終了し、あとで削除されるのを待つ
                if self.customer_status == CustomerCharacterStatus::GotOut {
                    return;
                }

                // キューが空ではない場合
                // 情報をキューから取り出し、速度を計算し直す
                if let Some(next_position) = self.move_queue.dequeue() {
                    self.override_move_effect(ctx.context, next_position);
                    self.current_goal = next_position;
                }
            }
            CustomerCharacterStatus::GoToCheck => {
                // まだゴールしていない
                if !self.is_goal_now(ctx.context) {
                    return;
                }

                //
                // 以下ゴール後
                //

                let goal = self.current_goal;

                // 目的地でマップ位置を上書き
                self.get_mut_character_object()
                    .set_map_position_with_collision_top_offset(ctx.context, goal);

                // キューが空ではない場合
                // 情報をキューから取り出し、速度を計算し直す
                if let Some(next_position) = self.move_queue.dequeue() {
                    self.override_move_effect(ctx.context, next_position);
                    self.current_goal = next_position;
                } else {
                    self.check_been_counter(map_data, goal, counter);
                    // 速度もリセット
                    self.reset_speed();
                    self.character.change_animation_mode(ObjectDirection::Left);
                }
            }

            CustomerCharacterStatus::WaitOnBookShelf => {}
            CustomerCharacterStatus::GotOut => {}
        }
    }

    pub fn is_wait_on_clerk(&self) -> bool {
        self.customer_status == CustomerCharacterStatus::WaitOnClerk
    }

    pub fn check_rise_hand<'a>(&mut self, ctx: &mut SuzuContext<'a>) -> Option<CustomerRequest> {
        if self.customer_status == CustomerCharacterStatus::WaitOnClerk {
            Some(self.generate_hold_request(ctx))
        } else {
            None
        }
    }

    pub fn get_map_position(&self) -> numeric::Point2f {
        self.character.get_map_position()
    }

    pub fn get_character_object(&self) -> &MapObject {
        &self.character
    }

    pub fn get_mut_character_object(&mut self) -> &mut MapObject {
        &mut self.character
    }

    pub fn fix_collision_horizon(
        &mut self,
        ctx: &mut ggez::Context,
        info: &CollisionInformation,
        t: Clock,
    ) -> f32 {
        self.character.fix_collision_horizon(ctx, info, t)
    }

    pub fn fix_collision_vertical(
        &mut self,
        ctx: &mut ggez::Context,
        info: &CollisionInformation,
        t: Clock,
    ) -> f32 {
        self.character.fix_collision_vertical(ctx, info, t)
    }

    pub fn move_map(&mut self, offset: numeric::Vector2f) {
        self.character.move_map(offset);
    }

    pub fn move_map_current_speed_x(&mut self) {
        self.move_map(numeric::Vector2f::new(
            self.get_character_object().speed_info().get_speed().x,
            0.0,
        ))
    }

    pub fn move_map_current_speed_y(&mut self) {
        self.move_map(numeric::Vector2f::new(
            0.0,
            self.get_character_object().speed_info().get_speed().y,
        ))
    }

    pub fn ready_to_check(&self) -> bool {
        match self.customer_status {
            CustomerCharacterStatus::Moving | CustomerCharacterStatus::Ready => true,
            _ => false,
        }
    }
}

impl DrawableComponent for CustomerCharacter {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.get_mut_character_object().draw(ctx).unwrap();
        }
        Ok(())
    }

    fn hide(&mut self) {
        self.get_mut_character_object().hide()
    }

    fn appear(&mut self) {
        self.get_mut_character_object().appear()
    }

    fn is_visible(&self) -> bool {
        self.get_character_object().is_visible()
    }

    fn set_drawing_depth(&mut self, depth: i8) {
        self.get_mut_character_object().set_drawing_depth(depth)
    }

    fn get_drawing_depth(&self) -> i8 {
        self.get_character_object().get_drawing_depth()
    }
}

impl OnMap for CustomerCharacter {
    // マップ上のテクスチャ描画開始地点を返す
    fn get_map_position(&self) -> numeric::Point2f {
        self.character.get_map_position()
    }

    // マップ上のテクスチャ描画領域の右下の位置を返す
    fn get_map_position_bottom_right(&self, ctx: &mut ggez::Context) -> numeric::Point2f {
        self.character.get_map_position_bottom_right(ctx)
    }

    // マップ上のテクスチャ描画開始地点を設定する
    fn set_map_position(&mut self, position: numeric::Point2f) {
        self.character.set_map_position(position);
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum EventTrigger {
    Action,
    Touch,
}

impl FromStr for EventTrigger {
    type Err = ();

    fn from_str(trigger_str: &str) -> Result<Self, Self::Err> {
        match trigger_str {
            "action" => Ok(Self::Action),
            "touch" => Ok(Self::Touch),
            _ => panic!("Error: EventTrigger::from_str"),
        }
    }
}

pub trait MapEvent {
    fn get_trigger_method(&self) -> EventTrigger;
}

pub struct MapTextEvent {
    trigger: EventTrigger,
    text: String,
}

impl MapTextEvent {
    pub fn from_toml_object(toml_script: &toml::value::Value) -> Self {
        MapTextEvent {
            trigger: EventTrigger::from_str(toml_script.get("trigger").unwrap().as_str().unwrap())
                .unwrap(),
            text: toml_script
                .get("text")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
        }
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }
}

impl MapEvent for MapTextEvent {
    fn get_trigger_method(&self) -> EventTrigger {
        self.trigger
    }
}

pub struct MapEventSceneSwitch {
    trigger: EventTrigger,
    switch_scene: SceneID,
}

impl MapEventSceneSwitch {
    pub fn new(trigger: EventTrigger, switch_scene: SceneID) -> Self {
        MapEventSceneSwitch {
            trigger: trigger,
            switch_scene: switch_scene,
        }
    }

    pub fn from_toml_object(toml_script: &toml::value::Value) -> Self {
        MapEventSceneSwitch {
            trigger: EventTrigger::from_str(toml_script.get("trigger").unwrap().as_str().unwrap())
                .unwrap(),
            switch_scene: SceneID::from_str(
                toml_script
                    .get("switch-scene-id")
                    .unwrap()
                    .as_str()
                    .unwrap(),
            )
            .unwrap(),
        }
    }

    pub fn get_switch_scene_id(&self) -> SceneID {
        self.switch_scene
    }
}

impl MapEvent for MapEventSceneSwitch {
    fn get_trigger_method(&self) -> EventTrigger {
        self.trigger
    }
}

pub struct BookStoreEvent {
    trigger: EventTrigger,
    book_shelf_info: BookShelfInformation,
}

impl BookStoreEvent {
    pub fn from_toml_object(toml_script: &toml::value::Value) -> Self {
        let shelf_info = toml_script.get("shelf-info").unwrap().as_table().unwrap();
        let book_shelf_info = BookShelfInformation::new(
            shelf_info
                .get("begin-number")
                .unwrap()
                .as_integer()
                .unwrap() as u16,
            shelf_info.get("end-number").unwrap().as_integer().unwrap() as u16,
        );

        BookStoreEvent {
            trigger: EventTrigger::from_str(toml_script.get("trigger").unwrap().as_str().unwrap())
                .unwrap(),
            book_shelf_info: book_shelf_info,
        }
    }

    pub fn get_book_shelf_info(&self) -> &BookShelfInformation {
        &self.book_shelf_info
    }
}

impl MapEvent for BookStoreEvent {
    fn get_trigger_method(&self) -> EventTrigger {
        self.trigger
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum BuiltinEventSymbol {
    SelectShelvingBook = 0,
}

impl FromStr for BuiltinEventSymbol {
    type Err = ();

    fn from_str(builtin_event_symbol: &str) -> Result<Self, Self::Err> {
        match builtin_event_symbol {
            "select-shelving-book" => Ok(Self::SelectShelvingBook),
            _ => panic!("Error: BuiltinEventSymbol::from_str"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct BuiltinEvent {
    trigger: EventTrigger,
    event_symbol: BuiltinEventSymbol,
}

impl BuiltinEvent {
    pub fn from_toml_object(toml_script: &toml::value::Value) -> Self {
        let builtin_event_info = toml_script
            .get("builtin-event-info")
            .unwrap()
            .as_table()
            .unwrap();
        BuiltinEvent {
            trigger: EventTrigger::from_str(toml_script.get("trigger").unwrap().as_str().unwrap())
                .unwrap(),
            event_symbol: BuiltinEventSymbol::from_str(
                builtin_event_info.get("symbol").unwrap().as_str().unwrap(),
            )
            .unwrap(),
        }
    }

    pub fn get_event_symbol(&self) -> BuiltinEventSymbol {
        self.event_symbol
    }
}

impl MapEvent for BuiltinEvent {
    fn get_trigger_method(&self) -> EventTrigger {
        self.trigger
    }
}

pub enum MapEventElement {
    TextEvent(MapTextEvent),
    SwitchScene(MapEventSceneSwitch),
    BookStoreEvent(BookStoreEvent),
    BuiltinEvent(BuiltinEvent),
}

impl MapEvent for MapEventElement {
    fn get_trigger_method(&self) -> EventTrigger {
        match self {
            Self::TextEvent(text) => text.get_trigger_method(),
            Self::SwitchScene(switch_scene) => switch_scene.get_trigger_method(),
            Self::BookStoreEvent(book_store_event) => book_store_event.get_trigger_method(),
            Self::BuiltinEvent(builtin_event) => builtin_event.get_trigger_method(),
        }
    }
}

pub struct MapEventList {
    event_table: HashMap<numeric::Point2i, MapEventElement>,
}

impl MapEventList {
    pub fn from_file(file_path: &str) -> Self {
        let mut table = HashMap::new();

        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => panic!("Failed to read: {}", file_path),
        };

        let root = content.parse::<toml::Value>().unwrap();
        let array = root["event-panel"].as_array().unwrap();

        for elem in array {
            let position_data = elem.get("position").unwrap().as_table().unwrap();
            let position = numeric::Point2i::new(
                position_data.get("x").unwrap().as_integer().unwrap() as i32,
                position_data.get("y").unwrap().as_integer().unwrap() as i32,
            );
            if let Some(type_info) = elem.get("type") {
                match type_info.as_str().unwrap() {
                    "text" => {
                        table.insert(
                            position,
                            MapEventElement::TextEvent(MapTextEvent::from_toml_object(elem)),
                        );
                    }
                    "switch-scene" => {
                        table.insert(
                            position,
                            MapEventElement::SwitchScene(MapEventSceneSwitch::from_toml_object(
                                elem,
                            )),
                        );
                    }
                    "book-shelf" => {
                        table.insert(
                            position,
                            MapEventElement::BookStoreEvent(BookStoreEvent::from_toml_object(elem)),
                        );
                    }
                    "builtin-event" => {
                        table.insert(
                            position,
                            MapEventElement::BuiltinEvent(BuiltinEvent::from_toml_object(elem)),
                        );
                    }
                    _ => eprintln!("Error"),
                }
            } else {
                eprintln!("Error");
            }
        }

        MapEventList { event_table: table }
    }

    pub fn register_event(&mut self, point: numeric::Point2i, event: MapEventElement) -> &mut Self {
        self.event_table.insert(point, event);
        self
    }

    pub fn check_event(
        &self,
        trigger: EventTrigger,
        point: numeric::Point2i,
    ) -> Option<&MapEventElement> {
        if let Some(event_element) = self.event_table.get(&point) {
            if event_element.get_trigger_method() == trigger {
                return Some(&event_element);
            }
        }

        None
    }
}
