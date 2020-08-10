pub mod factory;
pub mod tt_main_component;
pub mod tt_menu_component;
pub mod tt_sub_component;

use std::rc::Rc;

use ggez::graphics as ggraphics;
use ggez::input as ginput;
use ginput::mouse::MouseCursor;

use torifune::core::Clock;
use torifune::debug;
use torifune::device::VirtualKey;
use torifune::graphics::drawable::*;
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::graphics::object::*;
use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;
use torifune::numeric;

use crate::flush_delay_event;
use crate::object::util_object::*;
use crate::object::{effect, move_fn};
use crate::scene::*;
use tt_main_component::*;
use tt_menu_component::*;
use tt_sub_component::*;

use super::Clickable;
use crate::core::{
    BorrowingInformation, GensoDate, RentalLimit, ReturnBookInformation, SuzuContext, TextureID,
    TileBatchTextureID,
};

use number_to_jk::number_to_jk;

pub struct TaskTable {
    canvas: SubScreen,
    info_panel: TaskInfoPanel,
    sight: SuzuMiniSight,
    desk: DeskObjects,
    staging_object: Option<TaskTableStagingObject>,
    kosuzu_memory: KosuzuMemory,
    shelving_box: ShelvingBookBox,
    event_list: DelayEventList<TaskTable>,
    borrowing_record_book: BorrowingRecordBook,
    record_book_is_staged: bool,
    customer_silhouette_menu: CustomerMenuGroup,
    on_desk_menu: OnDeskMenuGroup,
    record_book_menu: RecordBookMenuGroup,
    current_customer_request: Option<CustomerRequest>,
    kosuzu_phrase: KosuzuPhrase,
    today: GensoDate,
    task_is_done: bool,
    appearance_frame: TileBatchFrame,
}

impl TaskTable {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        pos: numeric::Rect,
        info_panel_rect: numeric::Rect,
        sight_rect: numeric::Rect,
        desk_rect: numeric::Rect,
        shelving_box_rect: numeric::Rect,
        record_book_data: Option<BorrowingRecordBookData>,
        customer_request: Option<CustomerRequest>,
        t: Clock,
    ) -> Self {
        let sight = SuzuMiniSight::new(ctx, sight_rect, t);

        let mut desk = DeskObjects::new(ctx, desk_rect);

	let texture = UniTexture::new(
            ctx.ref_texture(TextureID::Chobo1),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(0.2, 0.2),
            0.0,
            -1,
        );

	let chobo_texture = UniTexture::new(
            ctx.ref_texture(TextureID::Chobo1),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(0.5, 0.5),
            0.0,
            -1,
        );
        let mut record_book = TaskTexture::new(
            OnDeskTexture::new(
                ctx.context,
                texture,
                OnDeskType::BorrowingRecordBook,
            ),
            OnDeskTexture::new(
                ctx.context,
                chobo_texture,
                OnDeskType::BorrowingRecordBook,
            ),
            0,
            true,
            true,
            DeskObjectType::BorrowRecordBook,
            t,
        );
        record_book.enable_large();
        desk.add_object(TaskItem::Texture(record_book));

        let appr_frame = TileBatchFrame::new(
            ctx.resource,
            TileBatchTextureID::BlackFrame,
            numeric::Rect::new(300.0, 0.0, 1066.0, 768.0),
            numeric::Vector2f::new(1.0, 1.0),
            0,
        );

        let shelving_box = ShelvingBookBox::new(ctx, shelving_box_rect);

        let mut record_book = BorrowingRecordBook::new(
            ctx,
            ggraphics::Rect::new(320.0, -550.0, 1000.0, 550.0),
            0,
            record_book_data,
            t,
        );
        if record_book.pages_length() == 0 {
            record_book.add_empty_page(ctx, 0);
        }
        record_book.hide();

        TaskTable {
            canvas: SubScreen::new(
                ctx.context,
                pos,
                0,
                ggraphics::Color::from_rgba_u32(0x00000000),
            ),
            info_panel: TaskInfoPanel::new(ctx, info_panel_rect, customer_request),
            sight: sight,
            desk: desk,
            staging_object: None,
            kosuzu_memory: KosuzuMemory::new(),
            shelving_box: shelving_box,
            event_list: DelayEventList::new(),
            borrowing_record_book: record_book,
            record_book_is_staged: false,
            customer_silhouette_menu: CustomerMenuGroup::new(0),
            record_book_menu: RecordBookMenuGroup::new(0),
            on_desk_menu: OnDeskMenuGroup::new(0),
            current_customer_request: None,
            kosuzu_phrase: KosuzuPhrase::new(ctx, 0),
            today: ctx.savable_data.date,
            task_is_done: false,
            appearance_frame: appr_frame,
        }
    }

    pub fn get_kosuzu_memory(&self) -> &KosuzuMemory {
        &self.kosuzu_memory
    }

    fn select_dragging_object<'a>(&mut self, ctx: &mut SuzuContext<'a>, point: numeric::Point2f) {
        let rpoint = self.canvas.relative_point(point);

        // メニューがオブジェクトの上に表示されている場合、ドラッグする
        // オブジェクトの走査は行わない
        if self.record_book_is_staged {
            return ();
        }

        if self
            .record_book_menu
            .is_contains_any_menus(ctx.context, rpoint)
        {
            return ();
        }

        if self
            .customer_silhouette_menu
            .is_contains_any_menus(ctx.context, rpoint)
        {
            return ();
        }

        if self.on_desk_menu.is_contains_any_menus(ctx.context, rpoint) {
            return ();
        }

        self.desk.select_dragging_object(ctx, rpoint);
    }

    pub fn mouse_motion_handler<'a>(
        &mut self,
        _: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        _: numeric::Vector2f,
    ) {
        self.borrowing_record_book.mouse_motion_handler(point);
    }

    fn slide_appear_record_book(&mut self, t: Clock) {
        self.borrowing_record_book.appear();

        self.borrowing_record_book.override_move_func(
            move_fn::devide_distance(numeric::Point2f::new(320.0, 100.0), 0.1),
            t,
        );
        self.event_list.add_event(
            Box::new(|tt: &mut TaskTable, _, t| tt.borrowing_record_book.override_move_func(None, t)),
            t + 100,
        );
    }

    fn slide_hide_record_book(&mut self, t: Clock) {
        self.event_list.add_event(
            Box::new(|tt: &mut TaskTable, _, _| tt.borrowing_record_book.hide()),
            t + 30,
        );

        self.borrowing_record_book.override_move_func(
            move_fn::devide_distance(numeric::Point2f::new(320.0, -550.0), 0.1),
            t,
        );
        self.record_book_is_staged = false;

        self.record_book_menu.close_all(t);
    }

    pub fn double_click_handler<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        t: Clock,
    ) {
        let rpoint = self.canvas.relative_point(point);

        let clicked_object_type = self.desk.double_click_handler(ctx, rpoint);

        if clicked_object_type.is_some() {
            match clicked_object_type.unwrap() {
                OnDeskType::BorrowingRecordBook => {
                    debug::debug_screen_push_text("slide appear record book");
                    self.slide_appear_record_book(t);
                    self.record_book_is_staged = true;
                }
                _ => (),
            }
        }
    }

    pub fn dragging_handler<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        last: numeric::Point2f,
    ) {
        let rpoint = self.canvas.relative_point(point);
        let rlast = self.canvas.relative_point(last);

        self.sight.dragging_handler(rpoint, rlast);
        self.desk.dragging_handler(ctx, rpoint);
        self.shelving_box.dragging_handler(rpoint, rlast);
    }

    pub fn unselect_dragging_object<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        self.sight.unselect_dragging_object(ctx.context, t);
        self.desk.unselect_dragging_object(ctx);
        self.shelving_box.unselect_dragging_object(t);
    }

    pub fn hand_over_check(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        let rpoint = self.canvas.relative_point(point);

        self.hand_over_check_d2s(ctx, rpoint);
        self.hand_over_check_s2d(ctx, rpoint);
        self.hand_over_check_desk2box(ctx, rpoint);
        self.hand_over_check_box2desk(ctx, rpoint);
    }

    fn apply_d2s_point_convertion(&mut self, ctx: &mut ggez::Context, obj: &mut TaskItem) {
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

    fn apply_s2d_point_convertion(&mut self, ctx: &mut ggez::Context, obj: &mut TaskItem) {
        let mut obj_p = obj.get_object().get_position();
        obj_p.x = obj.get_object().get_center(ctx).x;
        let p = self.sight_edge_to_desk_edge(obj_p);
        obj.enable_large();
        obj.get_object_mut().make_center(ctx, p);
    }

    fn apply_desk2box_point_convertion(&mut self, ctx: &mut ggez::Context, obj: &mut TaskItem) {
        // オブジェクトの座標を取得
        let mut obj_p = obj.get_object().get_center(ctx);

        // Y座標は変更せず, X座標をCanvasの右端に来るように設定
        obj_p.x = 0.0;

        obj.enable_small();

        obj.get_object_mut().make_center(ctx, obj_p);

        // 新しい座標を設定
        obj.get_object_mut().make_center(ctx, obj_p);
    }

    fn apply_box2desk_point_convertion(&mut self, ctx: &mut ggez::Context, obj: &mut TaskItem) {
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
            if self
                .desk
                .ref_dragging()
                .unwrap()
                .is_shelving_box_handover_locked()
            {
                return;
            }

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

    fn check_sight_drop_to_desk<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        let converted = self.sight.check_drop_desk();
        if converted.len() == 0 {
            return ();
        }

        let min = self.desk.desk_objects.get_minimum_depth();
        let converted = converted
            .into_iter()
            .map(|mut obj| {
                self.apply_s2d_point_convertion(ctx.context, &mut obj);
                obj.as_effectable_object().clear_effect();
                obj.as_movable_object_mut()
                    .override_move_func(move_fn::gravity_move(1.0, 10.0, 400.0, 0.3), t);
                obj.get_object_mut().set_drawing_depth(min);
                obj.get_object_mut().finish_dragging(ctx);

                obj.as_effectable_object().add_effect(vec![Box::new(
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

    pub fn update<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        flush_delay_event!(self, self.event_list, ctx, t);

        self.sight.update(ctx, t);
        self.desk.update(ctx, t);
        self.shelving_box.update(ctx.context, t);
        self.check_sight_drop_to_desk(ctx, t);
        self.borrowing_record_book.update(t);
        self.record_book_menu.update(ctx, t);
        self.customer_silhouette_menu.update(ctx, t);
        self.on_desk_menu.update(ctx, t);
        self.kosuzu_phrase.update(ctx, t);
        self.info_panel.update(t);

        self.check_task_is_done();
    }

    pub fn finish_customer_event(&mut self, now: Clock) {
        self.sight.finish_customer_event(now);
    }

    pub fn task_is_done(&self) -> bool {
        self.task_is_done
    }

    fn start_borrowing_customer_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        info: BorrowingInformation,
        t: Clock,
    ) {
        for book_info in &info.borrowing {
            let mut obj = factory::create_dobj_book(
                ctx,
                DeskObjectType::CustomerObject,
                book_info.clone(),
                t,
            );
            obj.enable_large();
            self.desk.add_customer_object(obj);
        }

        let mut new_silhouette = SimpleObject::new(
            MovableUniTexture::new(
                ctx.ref_texture(TextureID::JunkoTachieDefault),
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
            ctx.context,
            new_silhouette,
            info.borrower.to_string(),
            CustomerDialogue::new(
                vec!["こんにちは".to_string(), "この本貸してください".to_string()],
                vec![100, 100],
            ),
            t,
        );
    }

    fn start_returning_customer_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        info: ReturnBookInformation,
        t: Clock,
    ) {
        for book_info in &info.returning {
            let mut obj = factory::create_dobj_book(
                ctx,
                DeskObjectType::CustomerObject,
                book_info.clone(),
                t,
            );
            obj.enable_large();
            self.desk.add_customer_object(obj);
        }

        let mut new_silhouette = SimpleObject::new(
            MovableUniTexture::new(
                ctx.ref_texture(TextureID::JunkoTachieDefault),
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
            ctx.context,
            new_silhouette,
            info.borrower.to_string(),
            CustomerDialogue::new(
                vec!["こんにちは".to_string(), "本の返却お願いします".to_string()],
                vec![100, 100],
            ),
            t,
        );
    }

    pub fn start_customer_event(&mut self, ctx: &mut SuzuContext, info: CustomerRequest, t: Clock) {
        self.current_customer_request = Some(info.clone());

        match info {
            CustomerRequest::Borrowing(info) => self.start_borrowing_customer_event(ctx, info, t),
            CustomerRequest::Returning(info) => self.start_returning_customer_event(ctx, info, t),
        }
    }

    fn check_borrowing_task_is_done(&self) -> bool {
        let mut book_count = 0;
        for obj in self.desk.desk_objects.get_raw_container().iter() {
            match obj {
                TaskItem::Book(item) => {
                    if !self
                        .kosuzu_memory
                        .is_in_blacklist(item.get_large_object().get_book_info())
                    {
                        book_count += 1;
                    }
                }
                _ => (),
            }
        }

        book_count += self
            .sight
            .count_not_forbidden_book_items(&self.kosuzu_memory);

        if let Some(dragging) = self.desk.dragging.as_ref() {
            match dragging {
                TaskItem::Book(item) => {
                    if !self
                        .kosuzu_memory
                        .is_in_blacklist(item.get_large_object().get_book_info())
                    {
                        book_count += 1;
                    }
                }
                _ => (),
            }
        }

        book_count == 0
    }

    fn check_returning_task_is_done(&self) -> bool {
        let mut book_count = 0;
        for obj in self.desk.desk_objects.get_raw_container().iter() {
            match obj {
                TaskItem::Book(_) => {
                    book_count += 1;
                }
                _ => (),
            }
        }

        book_count += self
            .sight
            .count_not_forbidden_book_items(&self.kosuzu_memory);

        if let Some(dragging) = self.desk.dragging.as_ref() {
            match dragging {
                TaskItem::Book(item) => {
                    if !self
                        .kosuzu_memory
                        .is_in_blacklist(item.get_large_object().get_book_info())
                    {
                        book_count += 1;
                    }
                }
                _ => (),
            }
        }

        book_count == 0
    }

    fn check_task_is_done(&mut self) {
        if self.current_customer_request.is_none() {
            self.task_is_done = false;
            return;
        }

        self.task_is_done = match self.current_customer_request.as_ref().unwrap() {
            CustomerRequest::Borrowing(_) => self.check_borrowing_task_is_done(),
            CustomerRequest::Returning(_) => self.check_returning_task_is_done(),
        };
    }

    pub fn get_shelving_box(&self) -> &ShelvingBookBox {
        &self.shelving_box
    }

    pub fn get_shelving_box_mut(&mut self) -> &mut ShelvingBookBox {
        &mut self.shelving_box
    }

    /// キーハンドラ
    pub fn key_event_handler<'a>(
        &mut self,
        _ctx: &mut SuzuContext<'a>,
        vkey: VirtualKey,
        t: Clock,
    ) {
        match vkey {
            VirtualKey::Action3 => {
                if self.staging_object.is_some() {
                    self.staging_object.as_mut().unwrap().slide_hide(t);
                    self.event_list.add_event(
                        Box::new(|tt: &mut Self, _, _| tt.staging_object = None),
                        t + 100,
                    );
                }

                self.slide_hide_record_book(t);
            }
            _ => (),
        }
    }

    pub fn export_borrowing_record_book_data(&self) -> BorrowingRecordBookData {
        self.borrowing_record_book.export_book_data()
    }

    pub fn signing_borrowing_handler<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        let price = self.borrowing_record_book.get_calculated_price().unwrap();
        self.event_list.add_event(
            Box::new(move |slf: &mut Self, _, t| {
                slf.slide_hide_record_book(t);
            }),
            t + 30,
        );
        self.show_kosuzu_payment_message(ctx, price, t);

        // 本の情報が帳簿に記載されていた場合
        // 対応する本のハンドオーバーロックを解除する
        let written_books = self
            .borrowing_record_book
            .get_current_page_written_books()
            .unwrap();
        for item in self.desk.desk_objects.get_raw_container_mut().iter_mut() {
            match item {
                TaskItem::Book(book) => {
                    let info = book.get_large_object_mut().get_book_info();

                    if written_books.contains(&info) {
                        book.unlock_handover();
                    }
                }
                _ => (),
            }
        }

        self.sight.silhouette.insert_new_balloon_phrase(
            "どうぞ".to_string(),
            TextBalloonPhraseType::SimplePhrase,
            20,
            t,
        );
    }

    pub fn signing_returning_handler<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        self.event_list.add_event(
            Box::new(move |slf: &mut Self, _, t| {
                slf.slide_hide_record_book(t);
            }),
            t + 30,
        );

        self.show_kosuzu_returning_is_done_message(ctx, t);

        // 本の情報が帳簿に記載されていた場合
        // 対応する本のハンドオーバーロックを解除する
        let written_books = self
            .borrowing_record_book
            .get_current_page_written_books()
            .unwrap();
        for item in self.desk.desk_objects.get_raw_container_mut().iter_mut() {
            match item {
                TaskItem::Book(book) => {
                    let info = book.get_large_object_mut().get_book_info();

                    if written_books.contains(&info) {
                        book.unlock_shelving_box_handover();
                    }
                }
                _ => (),
            }
        }
    }

    ///
    /// メニューのエントリをクリックしていたらtrueを返し、そうでなければfalseを返す
    ///
    fn click_record_book_menu<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        if !self
            .record_book_menu
            .click_book_status_menu(ctx, button, point, t)
            && !self
                .record_book_menu
                .click_book_title_menu(ctx, button, point, t)
            && !self.record_book_menu.click_date_menu(ctx, button, point, t)
            && !self
                .record_book_menu
                .click_customer_name_menu(ctx, button, point, t)
            && !self
                .record_book_menu
                .click_payment_menu(ctx, button, point, t)
        {
            // メニューをクリックしていない場合はfalseをクリックして終了
            return false;
        }

        if let Some(index) = self.record_book_menu.book_status_menu_last_clicked() {
            let menu_position = self
                .record_book_menu
                .get_book_status_menu_position()
                .unwrap();
            self.borrowing_record_book.insert_book_status_via_choice(
                ctx.context,
                index,
                menu_position,
            );

            return true;
        }

        if let Some((index, book_info)) = self.record_book_menu.book_title_menu_last_clicked() {
            let menu_position = self
                .record_book_menu
                .get_book_title_menu_position()
                .unwrap();
            self.borrowing_record_book.insert_book_title_to_books_frame(
                ctx,
                menu_position,
                book_info,
            );

            self.kosuzu_memory.remove_book_info_at(index);

            return true;
        }

        if let Some((_, date)) = self.record_book_menu.date_menu_last_clicked() {
            let menu_position = self.record_book_menu.get_date_menu_position().unwrap();
            let maybe_rental_limit = self.today.rental_limit_type(&date);
            if let Some(rental_limit) = maybe_rental_limit {
                self.borrowing_record_book
                    .insert_date_data_to_customer_info(ctx, menu_position, date, rental_limit);
            }

            return true;
        }

        if let Some((_, name)) = self.record_book_menu.customer_name_menu_last_clicked() {
            let menu_position = self
                .record_book_menu
                .get_customer_name_menu_position()
                .unwrap();
            self.borrowing_record_book
                .insert_customer_name_data_to_customer_info(ctx.context, menu_position, name);

            return true;
        }

        false
    }

    ///
    /// 新しく、客の名前をsightのtext_balloonに表示させる
    ///
    fn insert_custmer_name_phrase(&mut self, t: Clock) {
        if let Some(customer_request) = self.current_customer_request.as_ref() {
            let phrase_text = format!("{}です", customer_request.get_customer_name());
            self.sight.silhouette.insert_new_balloon_phrase(
                phrase_text,
                TextBalloonPhraseType::CustomerName(customer_request.get_customer_name().clone()),
                20,
                t,
            );
        }
    }

    ///
    /// 新しく、客の名前をsightのtext_balloonに表示させる
    ///
    fn insert_rental_limit_phrase(&mut self, t: Clock) {
        if let Some(customer_request) = self.current_customer_request.as_ref() {
            match customer_request {
                CustomerRequest::Borrowing(info) => {
                    let phrase_text = match info.rental_limit {
                        RentalLimit::ShortTerm => "短期貸出でお願いします",
                        RentalLimit::LongTerm => "長期貸出でお願いします",
                        _ => "",
                    }
                    .to_string();

                    self.sight.silhouette.insert_new_balloon_phrase(
                        phrase_text,
                        TextBalloonPhraseType::RentalLimit(info.rental_limit.clone()),
                        20,
                        t,
                    );
                }
                _ => (),
            }
        }
    }

    fn refusing_book_borrowing_conversation(&mut self, t: Clock) {
        self.event_list.add_event(
            Box::new(move |slf: &mut Self, ctx, t| {
                slf.kosuzu_phrase
                    .insert_new_phrase(ctx, "すみません　この本は貸し出せません", t);
            }),
            t + 1,
        );

        self.event_list.add_event(
            Box::new(move |slf: &mut Self, ctx, _| {
                slf.sight.silhouette.replace_text(
                    ctx.context,
                    "あ そうなんですか",
                    TextBalloonPhraseType::SimplePhrase,
                );
            }),
            t + 30,
        );

        if let Some(customer_request) = self.current_customer_request.as_ref() {
            match customer_request {
                CustomerRequest::Borrowing(info) => {
                    if self.kosuzu_memory.full_of_blacklist(&info.borrowing) {}
                }
                _ => (),
            }
        }
    }

    fn show_kosuzu_payment_message<'a>(&mut self, ctx: &mut SuzuContext<'a>, price: u32, t: Clock) {
        self.kosuzu_phrase.insert_new_phrase(
            ctx,
            &format!("合計{}円になります", number_to_jk(price as u64)),
            t,
        );
    }

    fn show_kosuzu_returning_is_done_message<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        self.kosuzu_phrase
            .insert_new_phrase(ctx, "確認しました またお越しください", t);
    }

    ///
    /// メニューのエントリをクリックしていたらtrueを返し、そうでなければfalseを返す
    ///
    fn click_customer_silhouette_menu<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        println!("check customer_silhouette");
        if !self
            .customer_silhouette_menu
            .click_customer_question_menu(ctx, button, point, t)
        {
            // メニューをクリックしていない場合はfalseをクリックして終了
            println!("not clicked");
            return false;
        }

        if let Some(index) = self
            .customer_silhouette_menu
            .question_menu_last_clicked_index()
        {
            match index {
                0 => {
		    if let Some(customer_request) = self.current_customer_request.as_ref() {
			let name = customer_request.get_customer_name();
			self.kosuzu_memory.add_customer_name(name.clone());
			self.info_panel.set_customer_name(ctx, name);
			self.insert_custmer_name_phrase(t);
		    }
		},
                1 => self.insert_rental_limit_phrase(t),
                _ => panic!("Exception"),
            }

            return true;
        }

        false
    }

    ///
    /// メニューのエントリをクリックしていたらtrueを返し、そうでなければfalseを返す
    ///
    fn click_desk_book_menu<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        if !self
            .on_desk_menu
            .click_desk_book_menu(ctx, button, point, t)
        {
            // メニューをクリックしていない場合はfalseをクリックして終了
            println!("not clicked");
            return false;
        }

        if let Some(index) = self.on_desk_menu.desk_book_menu_last_clicked() {
            if let Some(book_info) = self.on_desk_menu.get_desk_menu_target_book_info() {
                return match index {
                    //
                    // 題名を記憶する
                    //
                    0 => {
                        if self.kosuzu_memory.is_in_blacklist(&book_info) {
                            self.kosuzu_phrase.insert_new_phrase(
                                ctx,
                                "これは貸出しないはずの本だ",
                                t,
                            );
                        } else {
                            println!("Ok, This book info is not in blacklist");
                            // info panel
                            self.info_panel
                                .add_book_info(ctx, book_info.clone(), point, t);

                            // internal memory
                            self.kosuzu_memory.add_book_info(book_info);
                        }
                        true
                    }
                    1 => {
                        let target_book_info =
                            self.on_desk_menu.get_desk_menu_target_book_info().unwrap();
                        self.kosuzu_memory.add_book_to_black_list(target_book_info);

                        self.refusing_book_borrowing_conversation(t);
                        true
                    }
                    _ => false,
                };
            }
        }

        false
    }

    ///
    /// book_info_frameに関するメニューを表示する
    ///
    /// book_info_frameをクリックした場合、true, そうでなければ、false
    ///
    fn try_show_menus_regarding_book_info<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        click_point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        let grid_pos = self
            .borrowing_record_book
            .get_book_info_frame_grid_position(ctx.context, click_point);

        if grid_pos.is_some() {
            match grid_pos.unwrap().y {
                0 => self.record_book_menu.show_book_title_menu(
                    ctx,
                    click_point,
                    &self.kosuzu_memory,
                    t,
                ),
                1 => self
                    .record_book_menu
                    .show_book_status_menu(ctx, click_point, t),
                _ => (),
            }

            true
        } else {
            false
        }
    }

    ///
    /// customer_info_frameに関するメニューを表示する
    ///
    /// customer_info_frameをクリックした場合、true, そうでなければ、false
    ///
    fn try_show_menus_regarding_customer_info<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        click_point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        let maybe_grid_pos = self
            .borrowing_record_book
            .get_customer_info_frame_grid_position(ctx.context, click_point);

        if let Some(grid_pos) = maybe_grid_pos {
            if grid_pos == numeric::Vector2u::new(2, 1) {
                self.record_book_menu.show_customer_name_menu(
                    ctx,
                    click_point,
                    &self.kosuzu_memory,
                    t,
                );
            } else if grid_pos == numeric::Vector2u::new(1, 1)
                || grid_pos == numeric::Vector2u::new(0, 1)
            {
                self.record_book_menu
                    .show_date_menu(ctx, click_point, self.today.clone(), t);
            }

            true
        } else {
            false
        }
    }

    ///
    /// customer_info_frameに関するメニューを表示する
    ///
    /// customer_info_frameをクリックした場合、true, そうでなければ、false
    ///
    fn try_show_menus_regarding_record_book_payment<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        click_point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        let maybe_grid_pos = self
            .borrowing_record_book
            .get_payment_frame_grid_position(ctx.context, click_point);

        if let Some(grid_pos) = maybe_grid_pos {
            if grid_pos == numeric::Vector2u::new(0, 1) {
                let price = self.borrowing_record_book.get_calculated_price().unwrap();
                self.record_book_menu
                    .show_payment_menu(ctx, click_point, price, t);
            }

            true
        } else {
            false
        }
    }

    fn try_show_text_balloon_menus<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        click_point: numeric::Point2f,
        t: Clock,
    ) {
        let phrase_type = self.sight.silhouette.get_text_balloon_phrase_type();

        match phrase_type {
            _ => self
                .customer_silhouette_menu
                .show_text_balloon_ok_menu(ctx, click_point, t),
        }
    }

    ///
    /// シルエットに関するメニューを表示する
    ///
    /// customer_menuをクリックした場合、true, そうでなければ、false
    ///
    fn try_show_menus_regarding_customer_silhoutte(
        &mut self,
        ctx: &mut SuzuContext,
        click_point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        let rpoint = self.sight.canvas.relative_point(click_point);
        if self
            .sight
            .silhouette
            .contains_text_balloon(ctx.context, rpoint)
        {
            self.try_show_text_balloon_menus(ctx, click_point, t);
            true
        } else if self
            .sight
            .silhouette
            .contains_character_silhouette(ctx.context, rpoint)
        {
            self.customer_silhouette_menu
                .show_customer_question_menu(ctx, click_point, t);
            true
        } else {
            false
        }
    }

    ///
    /// シルエットに関するメニューを表示する
    ///
    /// customer_menuをクリックした場合、true, そうでなければ、false
    ///
    fn try_show_menus_regarding_ondesk_book_info<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        click_point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        let rpoint = self.desk.canvas.relative_point(click_point);

        for dobj in self.desk.get_desk_objects_list().iter().rev() {
            if dobj.get_object().contains(ctx.context, rpoint) {
                let dobj_ref = &dobj.get_object();
                let obj_type = dobj_ref.get_type();
                let hold_data = dobj_ref.click_hold_data(ctx.context, rpoint);

                match obj_type {
                    OnDeskType::Book => match hold_data {
                        HoldData::BookName(info) => {
                            self.on_desk_menu.show_desk_book_menu(
                                ctx,
                                click_point,
                                info.clone(),
                                t,
                            );
                        }
                        _ => panic!(""),
                    },
                    _ => (),
                }

                return true;
            }
        }

        false
    }

    fn try_show_menus<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        click_point: numeric::Point2f,
        t: Clock,
    ) {
        // 既に表示されている場合は、メニューを消して終了
        if self.record_book_menu.is_some_menu_opened() {
            self.record_book_menu.close_all(t);
            return ();
        }

        // 既に表示されている場合は、メニューを消して終了
        if self.customer_silhouette_menu.is_some_menu_opened() {
            self.customer_silhouette_menu.close_all(t);
            return ();
        }

        if self.on_desk_menu.is_some_menu_opened() {
            self.on_desk_menu.close_all(t);
            return ();
        }

        if self.try_show_menus_regarding_record_book_payment(ctx, click_point, t) {
            return;
        }

        if self.try_show_menus_regarding_book_info(ctx, click_point, t) {
            return;
        }

        if self.try_show_menus_regarding_customer_info(ctx, click_point, t) {
            return;
        }

        if self
            .borrowing_record_book
            .contains(ctx.context, click_point)
        {
            return;
        }

        if self.record_book_is_staged {
            return;
        }

        if !self.try_show_menus_regarding_customer_silhoutte(ctx, click_point, t) {
            self.try_show_menus_regarding_ondesk_book_info(ctx, click_point, t);
        }
    }
}

impl DrawableComponent for TaskTable {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.info_panel.draw(ctx).unwrap();
            self.sight.draw(ctx).unwrap();
            self.desk.draw(ctx).unwrap();
            self.shelving_box.draw(ctx).unwrap();

            if let Some(staging_object) = self.staging_object.as_mut() {
                staging_object.draw(ctx)?;
            }

            self.borrowing_record_book.draw(ctx)?;
            self.kosuzu_phrase.draw(ctx)?;

            self.customer_silhouette_menu.draw(ctx)?;
            self.record_book_menu.draw(ctx)?;
            self.on_desk_menu.draw(ctx)?;

            self.appearance_frame.draw(ctx)?;

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
    fn button_down<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        _: Clock,
        _: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        self.select_dragging_object(ctx, point);
    }

    fn button_up<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        t: Clock,
        _: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        let rpoint = self.canvas.relative_point(point);

        // ボタンが離されたとき、メニュー外にあった場合、すべてのメニューを消す
        if !self
            .record_book_menu
            .is_contains_any_menus(ctx.context, rpoint)
        {
            self.record_book_menu.close_all(t);
        }

        if !self
            .customer_silhouette_menu
            .is_contains_any_menus(ctx.context, rpoint)
        {
            self.customer_silhouette_menu.close_all(t);
        }

        if !self.on_desk_menu.is_contains_any_menus(ctx.context, rpoint) {
            println!("close all!");
            self.on_desk_menu.close_all(t);
        }
    }

    fn on_click<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        t: Clock,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        let rpoint = self.canvas.relative_point(point);

        if self.click_record_book_menu(ctx, button, rpoint, t) {
            self.record_book_menu.close_all(t);
            return;
        }

        if self.borrowing_record_book.click_handler(ctx, t, rpoint) {
            // クリックハンドラが呼び出されたので終了
            return;
        }

        if let Some(sign_entry) = self
            .borrowing_record_book
            .sign_with_mouse_click(ctx, rpoint)
        {
            match sign_entry {
                SignFrameEntry::BorrowingSign => self.signing_borrowing_handler(ctx, t),
                SignFrameEntry::ReturningSign => self.signing_returning_handler(ctx, t),
            }
            return;
        }

        if self.click_customer_silhouette_menu(ctx, button, rpoint, t) {
            self.customer_silhouette_menu.close_all(t);
            return;
        }

        if self.click_desk_book_menu(ctx, button, point, t) {
            self.on_desk_menu.close_all(t);
            return;
        }

        // メニューをクリックしていない場合に、新しいメニュー表示処理を走らせる
        self.try_show_menus(ctx, rpoint, t);

        if self.desk.click_handler(ctx, t, button, rpoint) {
            // クリックハンドラが呼び出されたので終了
            return;
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

        cursor_status
    }
}
