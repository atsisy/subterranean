pub mod factory;
pub mod tt_main_component;
pub mod tt_sub_component;

use std::rc::Rc;

use ggez::graphics as ggraphics;
use ggez::input as ginput;
use ginput::mouse::MouseButton;
use ginput::mouse::MouseCursor;

use torifune::core::{Clock, Updatable};
use torifune::debug;
use torifune::device::{KeyboardEvent, VirtualKey};
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::graphics::object::*;
use torifune::graphics::*;
use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;
use torifune::numeric;

use crate::object::{effect, move_fn};
use tt_main_component::*;
use tt_sub_component::*;

use super::Clickable;
use crate::core::{GameData, TextureID};

pub struct TaskTable {
    canvas: SubScreen,
    sight: SuzuMiniSight,
    desk: DeskObjects,
    goods: Goods,
    customer_info_ui: CustomerInformationUI,
    staging_object: Option<TaskTableStagingObject>,
    shelving_box: ShelvingBookBox,
    hold_data: HoldData,
}

impl TaskTable {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        pos: numeric::Rect,
        sight_rect: numeric::Rect,
        goods_rect: numeric::Rect,
        desk_rect: numeric::Rect,
        shelving_box_rect: numeric::Rect,
        t: Clock,
    ) -> Self {
        let sight = SuzuMiniSight::new(ctx, game_data, sight_rect, t);
        let mut desk = DeskObjects::new(ctx, game_data, desk_rect);

        desk.add_object(DeskObject::new(
            Box::new(OnDeskTexture::new(
                UniTexture::new(
                    game_data.ref_texture(TextureID::LotusPink),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(0.1, 0.1),
                    0.0,
                    -1,
                ),
                OnDeskType::Texture,
            )),
            Box::new(OnDeskTexture::new(
                UniTexture::new(
                    game_data.ref_texture(TextureID::LotusPink),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(0.1, 0.1),
                    0.0,
                    -1,
                ),
                OnDeskType::Texture,
            )),
            1,
            DeskObjectType::SuzunaObject,
            t,
        ));

        let mut record_book = DeskObject::new(
            Box::new(OnDeskTexture::new(
                UniTexture::new(
                    game_data.ref_texture(TextureID::Chobo1),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(0.2, 0.2),
                    0.0,
                    -1,
                ),
                OnDeskType::BorrowingRecordBook,
            )),
            Box::new(OnDeskTexture::new(
                UniTexture::new(
                    game_data.ref_texture(TextureID::Chobo1),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(0.5, 0.5),
                    0.0,
                    -1,
                ),
                OnDeskType::BorrowingRecordBook,
            )),
            0,
            DeskObjectType::BorrowRecordBook,
            t,
        );
        record_book.enable_large();
        desk.add_object(record_book);

        let shelving_box = ShelvingBookBox::new(ctx, game_data, shelving_box_rect);

        TaskTable {
            canvas: SubScreen::new(ctx, pos, 0, ggraphics::Color::from_rgba_u32(0x00000000)),
            sight: sight,
            desk: desk,
            goods: Goods::new(ctx, game_data, goods_rect),
            customer_info_ui: CustomerInformationUI::new(
                ctx,
                game_data,
                numeric::Rect::new(1300.0, 50.0, 600.0, 400.0),
                0,
            ),
            staging_object: None,
            shelving_box: shelving_box,
            hold_data: HoldData::None,
        }
    }

    fn select_dragging_object(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        let rpoint = self.canvas.relative_point(point);
        self.desk.select_dragging_object(ctx, rpoint);
    }

    fn check_staging_object_ready(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        t: Clock,
    ) {
        let maybe_staging_object = self.desk.check_staging_object(ctx, game_data, t);
        if maybe_staging_object.is_some() {
            self.staging_object = maybe_staging_object;
        }
    }

    pub fn double_click_handler(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
        game_data: &GameData,
        t: Clock,
    ) {
        let rpoint = self.canvas.relative_point(point);
        self.desk.double_click_handler(ctx, rpoint, game_data);
        self.check_staging_object_ready(ctx, game_data, t);
    }

    pub fn dragging_handler(&mut self, point: numeric::Point2f, last: numeric::Point2f) {
        let rpoint = self.canvas.relative_point(point);
        let rlast = self.canvas.relative_point(last);

        self.sight.dragging_handler(rpoint, rlast);
        self.desk.dragging_handler(rpoint, rlast);
        self.shelving_box.dragging_handler(rpoint, rlast);
    }

    pub fn unselect_dragging_object(&mut self, ctx: &mut ggez::Context, t: Clock) {
        self.sight.unselect_dragging_object(ctx, t);
        self.desk.unselect_dragging_object();
        self.shelving_box.unselect_dragging_object(t);
    }

    pub fn hand_over_check(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        let rpoint = self.canvas.relative_point(point);

        self.hand_over_check_d2s(ctx, rpoint);
        self.hand_over_check_s2d(ctx, rpoint);
        self.hand_over_check_desk2box(ctx, rpoint);
        self.hand_over_check_box2desk(ctx, rpoint);
    }

    fn apply_d2s_point_convertion(&mut self, ctx: &mut ggez::Context, obj: &mut DeskObject) {
        // オブジェクトの座標を取得
        let mut obj_p = obj.get_object().get_position();

        // obj_pのy座標をドラッグしているオブジェクトの中心y座標に書き換える
        // これは、受け渡す時にオブジェクトの移動に違和感が発生することを防ぐためである
        obj_p.x = obj.get_object().get_center(ctx).x;

        let p = self.desk_edge_to_sight_edge(ctx, obj_p);

        obj.enable_small();

        // Y座標が右のデスクにいた時の中心Y座標になっているので、
        // make_centerで中心座標に設定することで違和感が無くなる
        obj.get_object_mut().make_center(ctx, p);
    }

    fn apply_s2d_point_convertion(&mut self, ctx: &mut ggez::Context, obj: &mut DeskObject) {
        let mut obj_p = obj.get_object().get_position();
        obj_p.x = obj.get_object().get_center(ctx).x;
        let p = self.sight_edge_to_desk_edge(obj_p);
        obj.enable_large();
        obj.get_object_mut().make_center(ctx, p);
    }

    fn apply_desk2box_point_convertion(&mut self, ctx: &mut ggez::Context, obj: &mut DeskObject) {
        // オブジェクトの座標を取得
        let mut obj_p = obj.get_object().get_center(ctx);

        // Y座標は変更せず, X座標をCanvasの右端に来るように設定
        obj_p.x = 0.0;

        obj.enable_small();

        obj.get_object_mut().make_center(ctx, obj_p);

        // 新しい座標を設定
        obj.get_object_mut().make_center(ctx, obj_p);
    }

    fn apply_box2desk_point_convertion(&mut self, ctx: &mut ggez::Context, obj: &mut DeskObject) {
        // オブジェクトの座標を取得
        let mut obj_p = obj.get_object().get_center(ctx);

        // Y座標は変更せず, X座標をCanvasの左端に来るように設定
        obj_p.x = self.desk.canvas.get_drawing_size(ctx).x;
        debug::debug_screen_push_text(&format!("y: {}", obj_p.y));

        obj.enable_large();

        obj.get_object_mut().make_center(ctx, obj_p);

        // 新しい座標を設定
        obj.get_object_mut().make_center(ctx, obj_p);
    }

    ///
    /// DeskからMiniSightにオブジェクトを移動させるメソッド
    ///
    fn hand_over_check_d2s(&mut self, ctx: &mut ggez::Context, rpoint: numeric::Point2f) {
        let border = self.desk_border(ctx);

        if self.desk.has_dragging() && border > rpoint.y {
            if let Some(mut dragging) = self.desk.release_dragging() {
                self.apply_d2s_point_convertion(ctx, &mut dragging);
                self.sight.insert_dragging(dragging);
            }
        }
    }

    ///
    /// MiniSightからDeskにオブジェクトを移動させるメソッド
    ///
    fn hand_over_check_s2d(&mut self, ctx: &mut ggez::Context, rpoint: numeric::Point2f) {
        let border = self.desk_border(ctx);

        if self.sight.has_dragging() && border < rpoint.y {
            if let Some(mut dragging) = self.sight.release_dragging() {
                self.apply_s2d_point_convertion(ctx, &mut dragging);
                self.desk.insert_dragging(dragging);
            }
        }
    }

    fn hand_over_check_desk2box(&mut self, ctx: &mut ggez::Context, rpoint: numeric::Point2f) {
        let border = self.desk_border_x(ctx);

        if self.desk.has_dragging() && border < rpoint.x {
            debug::debug_screen_push_text("desk 2 box");
            if let Some(mut dragging) = self.desk.release_dragging() {
                self.apply_desk2box_point_convertion(ctx, &mut dragging);
                self.shelving_box.insert_dragging(dragging);
            }
        }
    }

    fn hand_over_check_box2desk(&mut self, ctx: &mut ggez::Context, rpoint: numeric::Point2f) {
        let border = self.desk_border_x(ctx);

        if self.shelving_box.has_dragging() && border >= rpoint.x {
            debug::debug_screen_push_text("box 2 desk");
            if let Some(mut dragging) = self.shelving_box.release_dragging() {
                self.apply_box2desk_point_convertion(ctx, &mut dragging);
                self.desk.insert_dragging(dragging);
            }
        }
    }

    fn desk_border(&mut self, ctx: &mut ggez::Context) -> f32 {
        let sight_edge =
            self.sight.canvas.get_position().y + self.sight.canvas.get_texture_size(ctx).y;
        let diff = (sight_edge - self.desk.canvas.get_position().y).abs();
        sight_edge + diff
    }

    fn desk_border_x(&mut self, ctx: &mut ggez::Context) -> f32 {
        let desk_edge =
            self.desk.canvas.get_position().x + self.desk.canvas.get_texture_size(ctx).x;
        let diff = (desk_edge - self.shelving_box.canvas.get_position().x).abs();
        desk_edge + diff
    }

    fn desk_edge_to_sight_edge(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> numeric::Point2f {
        numeric::Point2f::new(point.x, self.sight.canvas.get_texture_size(ctx).y)
    }

    fn sight_edge_to_desk_edge(&mut self, rpoint: numeric::Point2f) -> numeric::Point2f {
        numeric::Point2f::new(rpoint.x, 0.0)
    }

    fn check_sight_drop_to_desk(&mut self, ctx: &mut ggez::Context, t: Clock) {
        let converted = self.sight.check_drop_desk();
        if converted.len() == 0 {
            return ();
        }

        let min = self.desk.desk_objects.get_minimum_depth();
        let converted = converted
            .into_iter()
            .map(|mut obj| {
                self.apply_s2d_point_convertion(ctx, &mut obj);
                obj.get_object_mut().clear_effect();
                obj.get_object_mut()
                    .override_move_func(move_fn::gravity_move(1.0, 10.0, 400.0, 0.3), t);
                obj.get_object_mut().set_drawing_depth(min);
                obj.get_object_mut().add_effect(vec![Box::new(
                    |obj: &mut dyn MovableObject, _: &ggez::Context, t: Clock| {
                        if obj.get_position().y > 150.0 {
                            obj.override_move_func(None, t);
                            EffectFnStatus::EffectFinish
                        } else {
                            EffectFnStatus::EffectContinue
                        }
                    },
                )]);
                obj
            })
            .collect::<Vec<_>>();
        self.desk.desk_objects.change_depth_equally(1);
        self.desk.add_customer_object_vec(converted);
    }

    pub fn update(&mut self, ctx: &mut ggez::Context, game_data: &GameData, t: Clock) {
        self.sight.update(ctx, game_data, t);
        self.desk.update(ctx, t);
        self.shelving_box.update(ctx, t);
        self.check_sight_drop_to_desk(ctx, t);
        self.customer_info_ui.update(ctx, t);
    }

    pub fn finish_customer_event(&mut self, now: Clock) {
        self.sight.finish_customer_event(now);
    }

    pub fn get_remaining_customer_object_number(&self) -> usize {
        self.desk
            .count_object_by_type(DeskObjectType::CustomerObject)
    }

    fn start_borrowing_customer_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        info: BorrowingInformation,
        t: Clock,
    ) {
        // 客への返却処理有効化
        self.sight.unlock_object_handover();

        for _ in &info.borrowing {
            let mut obj =
                factory::create_dobj_book_random(ctx, game_data, DeskObjectType::CustomerObject, t);
            obj.enable_large();
            self.desk.add_customer_object(obj);
        }

        let mut new_silhouette = SimpleObject::new(
            MovableUniTexture::new(
                game_data.ref_texture(TextureID::JunkoTachieDefault),
                numeric::Point2f::new(100.0, 20.0),
                numeric::Vector2f::new(0.1, 0.1),
                0.0,
                0,
                None,
                t,
            ),
            vec![
                effect::appear_bale_down_from_top(50, t),
                effect::fade_in(50, t),
            ],
        );
        new_silhouette.set_alpha(0.0);
        self.sight.silhouette_new_customer_update(
            ctx,
            new_silhouette,
            info.borrower.to_string(),
            CustomerDialogue::new(
                vec!["こんにちは".to_string(), "この本貸してください".to_string()],
                vec![100, 100],
            ),
            t,
        );
    }

    fn start_returning_customer_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        info: ReturnBookInformation,
        t: Clock,
    ) {
        // 客への返却処理無効化
        self.sight.lock_object_handover();

        for _ in &info.returning {
            let mut obj =
                factory::create_dobj_book_random(ctx, game_data, DeskObjectType::CustomerObject, t);
            obj.enable_large();
            self.desk.add_customer_object(obj);
        }

        let mut new_silhouette = SimpleObject::new(
            MovableUniTexture::new(
                game_data.ref_texture(TextureID::JunkoTachieDefault),
                numeric::Point2f::new(100.0, 20.0),
                numeric::Vector2f::new(0.1, 0.1),
                0.0,
                0,
                None,
                t,
            ),
            vec![
                effect::appear_bale_down_from_top(50, t),
                effect::fade_in(50, t),
            ],
        );
        new_silhouette.set_alpha(0.0);
        self.sight.silhouette_new_customer_update(
            ctx,
            new_silhouette,
            info.borrower.to_string(),
            CustomerDialogue::new(
                vec!["こんにちは".to_string(), "本の返却お願いします".to_string()],
                vec![100, 100],
            ),
            t,
        );
    }

    fn start_copying_request_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        info: CopyingRequestInformation,
        t: Clock,
    ) {
        // 客への返却処理有効化
        self.sight.unlock_object_handover();

        let paper_info = CopyingRequestInformation::new_random(
            game_data,
            GensoDate::new(128, 12, 8),
            GensoDate::new(128, 12, 8),
        );
        let paper_obj = DeskObject::new(
            Box::new(OnDeskTexture::new(
                UniTexture::new(
                    game_data.ref_texture(TextureID::Paper1),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(0.1, 0.1),
                    0.0,
                    0,
                ),
                OnDeskType::Texture,
            )),
            Box::new(CopyingRequestPaper::new(
                ctx,
                ggraphics::Rect::new(0.0, 0.0, 420.0, 350.0),
                TextureID::Paper1,
                paper_info,
                game_data,
                t,
            )),
            1,
            DeskObjectType::CustomerObject,
            t,
        );

        let mut new_silhouette = SimpleObject::new(
            MovableUniTexture::new(
                game_data.ref_texture(TextureID::JunkoTachieDefault),
                numeric::Point2f::new(100.0, 20.0),
                numeric::Vector2f::new(0.1, 0.1),
                0.0,
                0,
                None,
                t,
            ),
            vec![effect::fade_in(50, t)],
        );
        self.desk.add_customer_object(paper_obj);

        new_silhouette.set_alpha(0.0);
        self.sight.silhouette_new_customer_update(
            ctx,
            new_silhouette,
            info.customer.to_string(),
            CustomerDialogue::new(
                vec![
                    "こんにちは".to_string(),
                    "この本の写本\nお願いできますか".to_string(),
                ],
                vec![100, 100],
            ),
            t,
        );
    }

    pub fn start_customer_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        info: CustomerRequest,
        t: Clock,
    ) {
        match info {
            CustomerRequest::Borrowing(info) => {
                self.start_borrowing_customer_event(ctx, game_data, info, t)
            }
            CustomerRequest::Returning(info) => {
                self.start_returning_customer_event(ctx, game_data, info, t)
            }
            CustomerRequest::Copying(info) => {
                self.start_copying_request_event(ctx, game_data, info, t)
            }
        }
    }

    pub fn clear_hold_data(&mut self) {
        self.hold_data = HoldData::None;
    }

    fn update_hold_data_if_some(&mut self, new_hold_data: HoldData) {
        if new_hold_data.is_some() {
            self.hold_data = new_hold_data;
        }
    }

    ///
    /// HoldDataが取得可能な場合、取得し、self.hold_dataに上書きする
    ///
    fn update_hold_data(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        if self.hold_data.is_none() {
            let clicked_data = self.desk.check_data_click(ctx, point);
            self.update_hold_data_if_some(clicked_data);

            let clicked_data = self.sight.check_data_click(ctx, point);
            self.update_hold_data_if_some(clicked_data);

            let clicked_data = self.goods.check_data_click(ctx, point);
            self.update_hold_data_if_some(clicked_data);

            let clicked_data = self.customer_info_ui.check_data_click(ctx, point);
            self.update_hold_data_if_some(clicked_data);
        } else {
            if self.desk.check_insert_data(ctx, point, &self.hold_data) {
                self.hold_data = HoldData::None;
            }
        }
    }

    pub fn get_shelving_box(&self) -> &ShelvingBookBox {
        &self.shelving_box
    }

    pub fn get_shelving_box_mut(&mut self) -> &mut ShelvingBookBox {
        &mut self.shelving_box
    }

    /// キーハンドラ
    pub fn key_event_handler(&mut self, ctx: &mut ggez::Context, vkey: VirtualKey, t: Clock) {
        match vkey {
            VirtualKey::Action2 => {
                self.customer_info_ui.slide_toggle(t);
            }
            _ => (),
        }
    }
}

impl DrawableComponent for TaskTable {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.sight.draw(ctx).unwrap();
            self.desk.draw(ctx).unwrap();
            self.goods.draw(ctx).unwrap();
            self.shelving_box.draw(ctx).unwrap();

            if let Some(staging_object) = self.staging_object.as_mut() {
                staging_object.draw(ctx)?;
            }

            self.customer_info_ui.draw(ctx)?;

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

    /// 描画順序を設定する
    fn set_drawing_depth(&mut self, depth: i8) {
        self.canvas.set_drawing_depth(depth);
    }

    /// 描画順序を返す
    fn get_drawing_depth(&self) -> i8 {
        self.canvas.get_drawing_depth()
    }
}

impl DrawableObject for TaskTable {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for TaskTable {
    impl_texture_object_for_wrapped! {canvas}
}

impl Clickable for TaskTable {
    fn button_down(
        &mut self,
        ctx: &mut ggez::Context,
        _: &GameData,
        _: Clock,
        _button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        self.select_dragging_object(ctx, point);
    }

    fn button_up(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        t: Clock,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        let rpoint = self.canvas.relative_point(point);
        self.desk
            .button_up_handler(ctx, game_data, t, button, rpoint);
        if self.hold_data.is_some() && self.customer_info_ui.contains(ctx, rpoint) {
            let is_inserted = self.customer_info_ui.try_insert_hold_data_with_click(
                ctx,
                game_data,
                point,
                self.hold_data.clone(),
            );
            if is_inserted {
                self.hold_data = HoldData::None;
            }
        }

        if let Some(staging_object) = self.staging_object.as_mut() {
            staging_object.insert_data(ctx, point, &self.hold_data);
        }
    }

    fn on_click(
        &mut self,
        ctx: &mut ggez::Context,
        _: &GameData,
        _: Clock,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        if button == MouseButton::Left {
            let rpoint = self.canvas.relative_point(point);
            self.update_hold_data(ctx, rpoint);
            match &self.hold_data {
                HoldData::BookName(title) => println!("{}", title),
                HoldData::CustomerName(name) => println!("{}", name),
                _ => (),
            }
        }
    }

    fn clickable_status(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> ggez::input::mouse::MouseCursor {
        let rpoint = self.canvas.relative_point(point);

        let mut cursor_status = self.desk.check_mouse_cursor_status(ctx, rpoint);

        if cursor_status != MouseCursor::Default {
            return cursor_status;
        }

        cursor_status = self.sight.check_mouse_cursor_status(ctx, rpoint);
        if cursor_status != MouseCursor::Default {
            return cursor_status;
        }

        cursor_status = self.goods.check_mouse_cursor_status(ctx, rpoint);
        if cursor_status != MouseCursor::Default {
            return cursor_status;
        }

        cursor_status
    }
}
