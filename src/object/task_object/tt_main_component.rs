use std::collections::HashMap;
use std::rc::Rc;

use ggez::graphics as ggraphics;
use ggez::input as ginput;
use ginput::mouse::MouseCursor;

use torifune::core::{Clock, Updatable};
use torifune::graphics::object::shape;
use torifune::graphics::object::shape::MeshShape;
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::graphics::object::*;
use torifune::graphics::*;
use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;
use torifune::{debug, numeric};

use crate::object::util_object::*;
use crate::object::{effect, move_fn};
use crate::scene::*;

use super::Clickable;
use crate::core::{BookInformation, FontID, GameData, TextureID};

use super::tt_sub_component::*;

pub enum TaskTableStagingObject {
    BorrowingRecordBook(BorrowingRecordBook),
}

impl TaskTableStagingObject {
    pub fn insert_data(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
        hold_data: &HoldData,
    ) -> bool {
        match self {
            TaskTableStagingObject::BorrowingRecordBook(record_book) => {
                record_book.insert_data(ctx, point, hold_data)
            }
        }
    }

    ///
    /// 移動関数を変更しスライドアウトするように見せる
    ///
    pub fn slide_hide(&mut self, _t: Clock) {
        match self {
            TaskTableStagingObject::BorrowingRecordBook(_) => {
                debug::debug_screen_push_text("Implement slide hide, TaskTableStagingObject");
            }
        }
    }
}

impl DrawableComponent for TaskTableStagingObject {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        match self {
            TaskTableStagingObject::BorrowingRecordBook(p) => p.draw(ctx).unwrap(),
        }

        Ok(())
    }

    fn hide(&mut self) {
        match self {
            TaskTableStagingObject::BorrowingRecordBook(p) => p.hide(),
        }
    }

    fn appear(&mut self) {
        match self {
            TaskTableStagingObject::BorrowingRecordBook(p) => p.appear(),
        }
    }

    fn is_visible(&self) -> bool {
        match self {
            TaskTableStagingObject::BorrowingRecordBook(p) => p.is_visible(),
        }
    }

    fn set_drawing_depth(&mut self, depth: i8) {
        match self {
            TaskTableStagingObject::BorrowingRecordBook(p) => p.set_drawing_depth(depth),
        }
    }

    fn get_drawing_depth(&self) -> i8 {
        match self {
            TaskTableStagingObject::BorrowingRecordBook(p) => p.get_drawing_depth(),
        }
    }
}

pub struct DeskObjects {
    pub canvas: SubScreen,
    pub desk_objects: DeskObjectContainer,
    pub dragging: Option<DeskObject>,
    pub table_texture: SimpleObject,
    desk_book_drop_menu: Option<DeskBookDropMenu>,
}

impl DeskObjects {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        rect: ggraphics::Rect,
    ) -> DeskObjects {
        let mut dparam = ggraphics::DrawParam::default();
        dparam.dest = numeric::Point2f::new(rect.x, rect.y).into();

        let desk_objects = DeskObjectContainer::new();

        DeskObjects {
            canvas: SubScreen::new(ctx, rect, 0, ggraphics::Color::new(0.0, 0.0, 0.0, 0.0)),
            desk_objects: desk_objects,
            dragging: None,
            table_texture: SimpleObject::new(
                MovableUniTexture::new(
                    game_data.ref_texture(TextureID::Wood1),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    move_fn::stop(),
                    0,
                ),
                Vec::new(),
            ),
            desk_book_drop_menu: None,
        }
    }

    pub fn check_data_click(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> HoldData {
        let rpoint = self.canvas.relative_point(point);
        let mut clicked_data = HoldData::None;

        // オブジェクトは深度が深い順にソートされているので、
        // 逆順から検索していくことで、最も手前に表示されているオブジェクトを
        // 取り出すことができる
        for obj in self.desk_objects.get_raw_container_mut().iter_mut().rev() {
            let contains = obj.get_object().get_drawing_area(ctx).contains(rpoint);
            if contains {
                clicked_data = obj
                    .get_object_mut()
                    .ref_wrapped_object_mut()
                    .ref_wrapped_object_mut()
                    .click_data(ctx, rpoint);
                break;
            }
        }

        clicked_data
    }

    pub fn check_insert_data(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
        data: &HoldData,
    ) -> bool {
        let rpoint = self.canvas.relative_point(point);

        // オブジェクトは深度が深い順にソートされているので、
        // 逆順から検索していくことで、最も手前に表示されているオブジェクトを
        // 取り出すことができる
        for obj in self.desk_objects.get_raw_container_mut().iter_mut().rev() {
            let contains = obj.get_object().get_drawing_area(ctx).contains(rpoint);
            if contains {
                return obj
                    .get_object_mut()
                    .ref_wrapped_object_mut()
                    .ref_wrapped_object_mut()
                    .insert_data(ctx, rpoint, data);
            }
        }

        false
    }

    pub fn dragging_handler(&mut self, point: numeric::Point2f, last: numeric::Point2f) {
        if let Some(obj) = &mut self.dragging {
            obj.get_object_mut()
                .move_diff(numeric::Vector2f::new(point.x - last.x, point.y - last.y));
        }
    }

    pub fn select_dragging_object(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        let mut dragging_object_index = 0;
        let mut drag_start = false;

        let rpoint = self.canvas.relative_point(point);

        // オブジェクトは深度が深い順にソートされているので、
        // 逆順から検索していくことで、最も手前に表示されているオブジェクトを
        // 取り出すことができる
        for (index, obj) in self
            .desk_objects
            .get_raw_container_mut()
            .iter_mut()
            .rev()
            .enumerate()
        {
            if obj.get_object().get_drawing_area(ctx).contains(rpoint) {
                obj.get_object_mut().override_move_func(None, 0);
                dragging_object_index = self.desk_objects.len() - index - 1;
                drag_start = true;
                break;
            }
        }
        if drag_start {
            // 元々、最前面に表示されていたオブジェクトのdepthに設定する
            self.dragging = Some(
                self.desk_objects
                    .get_raw_container_mut()
                    .swap_remove(dragging_object_index),
            );
        }
    }

    pub fn unselect_dragging_object(&mut self) {
        if let Some(obj) = &mut self.dragging {
            let min = self.desk_objects.get_minimum_depth();
            obj.get_object_mut().set_drawing_depth(min);
            self.desk_objects.change_depth_equally(1);
        }
        match self.dragging {
            None => (),
            _ => {
                let dragged = self.release_dragging().unwrap();
                self.desk_objects.add(dragged);
                self.desk_objects.sort_with_depth();
            }
        }
    }

    pub fn update(&mut self, ctx: &mut ggez::Context, t: Clock) {
        for p in self.desk_objects.get_raw_container_mut() {
            p.get_object_mut().move_with_func(t);
            p.get_object_mut().effect(ctx, t);
        }
    }

    pub fn double_click_handler(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
        _game_data: &GameData,
    ) -> Option<OnDeskType> {
        let rpoint = self.canvas.relative_point(point);
        let mut click_flag = false;
        let mut object_type: Option<OnDeskType> = None;

        // オブジェクトは深度が深い順にソートされているので、
        // 逆順から検索していくことで、最も手前に表示されているオブジェクトを
        // 取り出すことができる
        for (_, obj) in self
            .desk_objects
            .get_raw_container_mut()
            .iter_mut()
            .rev()
            .enumerate()
        {
            if obj.get_object().get_drawing_area(ctx).contains(rpoint) {
                click_flag = true;

                object_type = Some(
                    obj.get_object()
                        .ref_wrapped_object()
                        .ref_wrapped_object()
                        .get_type(),
                );
                break;
            }
        }

        if click_flag {
            self.desk_objects.sort_with_depth();
        }

        object_type
    }

    pub fn add_object(&mut self, obj: DeskObject) {
        self.desk_objects.add(obj);
        self.desk_objects.sort_with_depth();
    }

    pub fn add_customer_object(&mut self, obj: DeskObject) {
        self.add_object(obj);
    }

    pub fn add_customer_object_vec(&mut self, mut obj_vec: Vec<DeskObject>) {
        while obj_vec.len() != 0 {
            self.add_object(obj_vec.pop().unwrap());
        }
    }

    pub fn has_dragging(&self) -> bool {
        self.dragging.is_some()
    }

    pub fn insert_dragging(&mut self, obj: DeskObject) {
        let d = std::mem::replace(&mut self.dragging, Some(obj));
        if d.is_some() {
            self.desk_objects.add(d.unwrap());
        }
    }

    pub fn release_dragging(&mut self) -> Option<DeskObject> {
        std::mem::replace(&mut self.dragging, None)
    }

    pub fn ref_dragging(&self) -> &Option<DeskObject> {
        &self.dragging
    }

    pub fn out_of_desk(&self, point: numeric::Point2f) -> bool {
        !self.canvas.contains(point)
    }

    pub fn count_object_by_type(&self, object_type: DeskObjectType) -> usize {
        let count = self
            .desk_objects
            .get_raw_container()
            .iter()
            .fold(0, |sum, obj| {
                sum + if obj.get_object_type() == object_type {
                    1
                } else {
                    0
                }
            });
        count + if self.dragging.is_some() { 1 } else { 0 }
    }

    fn show_book_drop_down_menu(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        point: numeric::Point2f,
        book_info: BookInformation,
        t: Clock,
    ) {
        // 既に表示されている場合は、消してすぐにreturn
        if self.desk_book_drop_menu.is_some() {
            self.desk_book_drop_menu = None;
            return ();
        }

        let menu = DeskBookMenu::new(
            ctx,
            game_data,
	    book_info,
            vec!["記憶".to_string(), "状態確認".to_string()],
            0,
	);

        self.desk_book_drop_menu = Some(DropDownArea::new(
            ctx,
            numeric::Rect::new(point.x, point.y, 200.0, 300.0),
            0,
            menu,
            t,
        ));
    }

    fn show_desk_object_drop_down_menu(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
	on_desk_type: OnDeskType,
	hold_data: HoldData,
        point: numeric::Point2f,
        t: Clock,
    ) {
        match on_desk_type {
            OnDeskType::Book => match hold_data {
                HoldData::BookName(info) => {
                    self.show_book_drop_down_menu(ctx, game_data, point, info.clone(), t);
                }
                _ => panic!(""),
            },
            _ => (),
        }
    }

    fn do_book_menu_action(&mut self, kosuzu_memory: &mut KosuzuMemory) -> bool {
	if let Some(book_menu) = self.desk_book_drop_menu.as_ref() {
	    let choice_group = book_menu.get_component();
	    let maybe_menu_id = choice_group.get_last_clicked();

	    if let Some(menu_id) = maybe_menu_id {
		return match menu_id {
		    0 => {
			panic!("ここに状態獲得の処理を記述する");
		    },
		    1 => {
			kosuzu_memory.add_book_info(choice_group.get_target_book_info());
			true
		    },
		    _ => false,
		}
	    }
	}

	false
    }

    pub fn button_up_handler(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        t: Clock,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
	kosuzu_memory: &mut KosuzuMemory,
    ) {
        let rpoint = self.canvas.relative_point(point);

        if let Some(book_menu) = self.desk_book_drop_menu.as_mut() {
	    if book_menu.contains(ctx, rpoint) {
	        book_menu.on_click(ctx, game_data, t, button, rpoint);
		if self.do_book_menu_action(kosuzu_memory) {
		    return ();
		}	
	    }
        }

        for dobj in self.desk_objects.get_raw_container_mut() {
            if dobj.get_object_mut().contains(ctx, rpoint) {
                dobj.get_object_mut()
                    .ref_wrapped_object_mut()
                    .ref_wrapped_object_mut()
                    .button_up(ctx, game_data, t, button, rpoint);

		let dobj_ref = dobj.get_object().ref_wrapped_object().ref_wrapped_object();
		let obj_type = dobj_ref.get_type();
		let hold_data = dobj_ref.click_data(ctx, rpoint);

		// オブジェクトの種類によってメニューを表示する
                self.show_desk_object_drop_down_menu(
                    ctx,
                    game_data,
		    obj_type,
		    hold_data,
                    rpoint,
                    t,
                );

                break;
            }
        }
    }

    pub fn check_mouse_cursor_status(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> MouseCursor {
        if self.canvas.get_drawing_area(ctx).contains(point) {
            let rpoint = self.canvas.relative_point(point);

            // オブジェクトは深度が深い順にソートされているので、
            // 逆順から検索していくことで、最も手前に表示されているオブジェクトを
            // 取り出すことができる
            for obj in self.desk_objects.get_raw_container_mut().iter_mut().rev() {
                if obj.get_object().get_drawing_area(ctx).contains(rpoint) {
                    return MouseCursor::Grab;
                }
            }
        }

        MouseCursor::Default
    }
}

impl DrawableComponent for DeskObjects {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.table_texture.draw(ctx)?;

            for obj in self.desk_objects.get_raw_container_mut() {
                obj.get_object_mut().draw(ctx)?;
            }

            if let Some(d) = self.dragging.as_mut() {
                d.get_object_mut().draw(ctx)?;
            }

            if let Some(book_menu) = self.desk_book_drop_menu.as_mut() {
                book_menu.draw(ctx)?;
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

    /// 描画順序を設定する
    fn set_drawing_depth(&mut self, depth: i8) {
        self.canvas.set_drawing_depth(depth);
    }

    /// 描画順序を返す
    fn get_drawing_depth(&self) -> i8 {
        self.canvas.get_drawing_depth()
    }
}

impl DrawableObject for DeskObjects {
    /// 描画開始地点を設定する
    fn set_position(&mut self, pos: numeric::Point2f) {
        self.canvas.set_position(pos);
    }

    /// 描画開始地点を返す
    fn get_position(&self) -> numeric::Point2f {
        self.canvas.get_position()
    }

    /// offsetで指定しただけ描画位置を動かす
    fn move_diff(&mut self, offset: numeric::Vector2f) {
        self.canvas.move_diff(offset)
    }
}

pub struct CustomerInformationUI {
    canvas: MovableWrap<SubScreen>,
    table_frame: TableFrame,
    head_text: MovableText,
    info_hash_map: HashMap<numeric::Vector2u, HoldDataVText>,
    header_texts: Vec<VerticalText>,
    ui_size: numeric::Vector2f,
    default_position: numeric::Point2f,
    now_appear: bool,
}

impl CustomerInformationUI {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        rect: numeric::Rect,
        t: Clock,
    ) -> Self {
        let table_frame = TableFrame::new(
            game_data,
            numeric::Point2f::new(50.0, 80.0),
            FrameData::new(vec![150.0, 150.0], vec![40.0; 8]),
            numeric::Vector2f::new(0.2, 0.2),
            0,
        );

        let head_text = MovableText::new(
            "小鈴覚書".to_string(),
            numeric::Point2f::new(100.0, 30.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            None,
            FontInformation::new(
                game_data.get_font(FontID::JpFude1),
                numeric::Vector2f::new(28.0, 28.0),
                ggraphics::Color::from_rgba_u32(0xff),
            ),
            t,
        );

        let mut header_texts = Vec::new();
        let header_font = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(24.0, 24.0),
            ggraphics::Color::from_rgba_u32(0xffffffff),
        );

        let mut customer_name_header = VerticalText::new(
            "氏名".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            header_font,
        );
        customer_name_header.make_center(
            ctx,
            table_frame.get_center_of(numeric::Vector2u::new(2, 0), table_frame.get_position()),
        );
        header_texts.push(customer_name_header);

        CustomerInformationUI {
            ui_size: numeric::Vector2f::new(rect.w, rect.h),
            default_position: numeric::Point2f::new(rect.x, rect.y),
            canvas: MovableWrap::new(
                Box::new(SubScreen::new(
                    ctx,
                    rect,
                    0,
                    ggraphics::Color::from_rgba_u32(0xffffffff),
                )),
                None,
                t,
            ),
            header_texts: header_texts,
            info_hash_map: HashMap::new(),
            table_frame: table_frame,
            head_text: head_text,
            now_appear: false,
        }
    }

    pub fn slide_toggle(&mut self, t: Clock) {
        if self.now_appear {
            self.canvas
                .override_move_func(move_fn::devide_distance(self.default_position, 0.5), t);
            self.now_appear = false;
        } else {
            self.canvas.override_move_func(
                move_fn::devide_distance(
                    numeric::Point2f::new(1366.0 - self.ui_size.x, self.default_position.y),
                    0.5,
                ),
                t,
            );
            self.now_appear = true;
        }
    }

    pub fn appearing_now(&self) -> bool {
        self.now_appear
    }

    pub fn insert_data_in_table(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        position: numeric::Vector2u,
        hold_data: HoldData,
    ) {
        let mut hold_data_vtext = HoldDataVText::new(
            hold_data,
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0,
            FontInformation::new(
                game_data.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0xff),
            ),
        );

        hold_data_vtext.make_center(
            ctx,
            self.table_frame
                .get_center_of(position, self.table_frame.get_position()),
        );
        debug::debug_screen_push_text(&format!("hold_data: {:?}", hold_data_vtext.get_position()));

        self.info_hash_map.insert(position, hold_data_vtext);
    }

    pub fn try_insert_hold_data_with_click(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        point: numeric::Point2f,
        hold_data: HoldData,
    ) -> bool {
        let rpoint = self.canvas.ref_wrapped_object().relative_point(point);
        let maybe_position = self.table_frame.get_grid_position(ctx, rpoint);

        if let Some(position) = maybe_position {
            self.insert_data_in_table(ctx, game_data, position, hold_data);
            true
        } else {
            false
        }
    }

    ///
    /// TableFrameに含まれるHoldDataを取り出すメソッド
    /// クリックした場所に何もない場合は、HoldData::Noneを返す
    ///
    pub fn check_data_click(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> HoldData {
        let rpoint = self.canvas.ref_wrapped_object().relative_point(point);
        let grid_pos = self.table_frame.get_grid_position(ctx, rpoint);

        if grid_pos.is_some() && self.info_hash_map.contains_key(&grid_pos.as_ref().unwrap()) {
            self.info_hash_map
                .get(grid_pos.as_ref().unwrap())
                .unwrap()
                .data
                .clone()
        } else {
            HoldData::None
        }
    }
}

impl DrawableComponent for CustomerInformationUI {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, self.canvas.ref_wrapped_object());

            self.table_frame.draw(ctx)?;
            self.head_text.draw(ctx)?;

            for (_, customer_info) in &mut self.info_hash_map {
                customer_info.draw(ctx)?;
            }

            for vtext in &mut self.header_texts {
                vtext.draw(ctx)?;
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
        self.canvas.appear()
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

impl DrawableObject for CustomerInformationUI {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for CustomerInformationUI {
    impl_texture_object_for_wrapped! {canvas}
}

impl Updatable for CustomerInformationUI {
    fn update(&mut self, _ctx: &ggez::Context, t: Clock) {
        self.canvas.move_with_func(t);
    }
}

struct TaskSilhouette {
    character: Option<SimpleObject>,
    name: Option<String>,
    canvas: SubScreen,
}

impl TaskSilhouette {
    pub fn new(
        ctx: &mut ggez::Context,
        pos_rect: numeric::Rect,
        char_obj: SimpleObject,
        name: &str,
    ) -> Self {
        TaskSilhouette {
            character: Some(char_obj),
            name: Some(name.to_string()),
            canvas: SubScreen::new(ctx, pos_rect, 0, ggraphics::Color::from_rgba_u32(0)),
        }
    }

    pub fn new_empty(ctx: &mut ggez::Context, pos_rect: numeric::Rect) -> Self {
        TaskSilhouette {
            character: None,
            name: None,
            canvas: SubScreen::new(ctx, pos_rect, 0, ggraphics::Color::from_rgba_u32(0)),
        }
    }

    pub fn is_some(&self) -> bool {
        self.character.is_some()
    }

    pub fn get_name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    pub fn change_character(&mut self, character: SimpleObject) -> &mut Self {
        self.character = Some(character);
        self
    }

    pub fn update_name(&mut self, name: String) -> &mut Self {
        self.name = Some(name);
        self
    }

    pub fn get_object(&self) -> Option<&SimpleObject> {
        self.character.as_ref()
    }

    pub fn get_object_mut(&mut self) -> Option<&mut SimpleObject> {
        self.character.as_mut()
    }
}

impl DrawableComponent for TaskSilhouette {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            if let Some(character) = &mut self.character {
                character.draw(ctx)?;
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

impl DrawableObject for TaskSilhouette {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for TaskSilhouette {
    impl_texture_object_for_wrapped! {canvas}
}

impl Clickable for TaskSilhouette {
    fn clickable_status(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> ggez::input::mouse::MouseCursor {
        if let Some(character) = &self.character {
            if character.get_drawing_area(ctx).contains(point) {
                return MouseCursor::Grab;
            }
        }

        MouseCursor::Default
    }
}

impl OnDesk for TaskSilhouette {
    fn ondesk_whose(&self) -> i32 {
        0
    }

    fn click_data(&self, _: &mut ggez::Context, _: numeric::Point2f) -> HoldData {
        if let Some(name) = &self.name {
            HoldData::CustomerName(name.to_string())
        } else {
            HoldData::None
        }
    }

    fn get_type(&self) -> OnDeskType {
        OnDeskType::Silhouette
    }
}

pub struct CustomerDialogue {
    dialogue: Vec<String>,
    time_step: Vec<u64>,
    current_index: usize,
}

impl CustomerDialogue {
    pub fn new(dialogue: Vec<String>, time_step: Vec<u64>) -> Self {
        CustomerDialogue {
            dialogue: dialogue,
            time_step: time_step,
            current_index: 0,
        }
    }

    pub fn get_current_dialogue_line(&self) -> String {
        self.dialogue.get(self.current_index).unwrap().to_string()
    }

    pub fn get_current_time_step(&self) -> u64 {
        if let Some(time) = self.time_step.get(self.current_index) {
            *time
        } else {
            0
        }
    }

    pub fn next(&mut self) {
        if self.current_index < (self.dialogue.len() - 1) {
            self.current_index += 1;
        }
    }

    pub fn last(&self) -> bool {
        (self.dialogue.len() - 1) == self.current_index as usize
    }
}

pub struct TextBalloon {
    canvas: SubScreen,
    text: VerticalText,
    balloon_inner: shape::Ellipse,
    balloon_outer: shape::Ellipse,
    mesh: ggraphics::Mesh,
}

impl TextBalloon {
    pub fn new(
        ctx: &mut ggez::Context,
        balloon_rect: numeric::Rect,
        text: &str,
        font_info: FontInformation,
    ) -> Self {
        let mut vtext = VerticalText::new(
            text.to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            font_info,
        );

        let vtext_size = vtext.get_drawing_size(ctx);

        vtext.make_center(
            ctx,
            numeric::Point2f::new((vtext_size.x + 80.0) / 2.0, (vtext_size.y + 60.0) / 2.0),
        );

        let ellipse = shape::Ellipse::new(
            numeric::Point2f::new((vtext_size.x + 100.0) / 2.0, (vtext_size.y + 60.0) / 2.0),
            (vtext_size.x + 60.0) / 2.0,
            (vtext_size.y + 50.0) / 2.0,
            0.1,
            ggraphics::DrawMode::fill(),
            ggraphics::Color::from_rgba_u32(0xffffeeff),
        );
        let ellipse_outer = shape::Ellipse::new(
            numeric::Point2f::new((vtext_size.x + 100.0) / 2.0, (vtext_size.y + 60.0) / 2.0),
            ((vtext_size.x + 60.0) / 2.0) + 5.0,
            ((vtext_size.y + 50.0) / 2.0) + 5.0,
            0.1,
            ggraphics::DrawMode::fill(),
            ggraphics::Color::from_rgba_u32(0x371905ff),
        );

        let mut mesh_builder = ggraphics::MeshBuilder::new();
        ellipse.add_to_builder(ellipse_outer.add_to_builder(&mut mesh_builder));

        TextBalloon {
            canvas: SubScreen::new(ctx, balloon_rect, 0, ggraphics::Color::from_rgba_u32(0x00)),
            text: vtext,
            balloon_inner: ellipse,
            balloon_outer: ellipse_outer,
            mesh: mesh_builder.build(ctx).unwrap(),
        }
    }

    pub fn replace_text(&mut self, ctx: &mut ggez::Context, text: &str) {
        self.text.replace_text(text.to_string());
        let vtext_size = self.text.get_drawing_size(ctx);

        self.balloon_inner = shape::Ellipse::new(
            numeric::Point2f::new((vtext_size.x + 100.0) / 2.0, (vtext_size.y + 60.0) / 2.0),
            (vtext_size.x + 60.0) / 2.0,
            (vtext_size.y + 50.0) / 2.0,
            0.1,
            ggraphics::DrawMode::fill(),
            ggraphics::Color::from_rgba_u32(0xffffeeff),
        );
        self.balloon_outer = shape::Ellipse::new(
            numeric::Point2f::new((vtext_size.x + 100.0) / 2.0, (vtext_size.y + 60.0) / 2.0),
            ((vtext_size.x + 60.0) / 2.0) + 5.0,
            ((vtext_size.y + 50.0) / 2.0) + 5.0,
            0.1,
            ggraphics::DrawMode::fill(),
            ggraphics::Color::from_rgba_u32(0x371905ff),
        );

        self.text.make_center(
            ctx,
            numeric::Point2f::new((vtext_size.x + 100.0) / 2.0, (vtext_size.y + 60.0) / 2.0),
        );

        self.update_mesh(ctx);
    }

    pub fn update_mesh(&mut self, ctx: &mut ggez::Context) {
        let mut mesh_builder = ggraphics::MeshBuilder::new();
        self.balloon_inner
            .add_to_builder(self.balloon_outer.add_to_builder(&mut mesh_builder));
        self.mesh = mesh_builder.build(ctx).unwrap();
    }
}

impl DrawableObject for TextBalloon {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for TextBalloon {
    fn set_scale(&mut self, scale: numeric::Vector2f) {
        self.canvas.set_scale(scale);
    }

    fn get_scale(&self) -> numeric::Vector2f {
        self.canvas.get_scale()
    }

    fn set_rotation(&mut self, rad: f32) {
        self.canvas.set_rotation(rad);
    }

    fn get_rotation(&self) -> f32 {
        self.canvas.get_rotation()
    }

    fn set_crop(&mut self, crop: ggraphics::Rect) {
        self.canvas.set_crop(crop);
    }

    fn get_crop(&self) -> ggraphics::Rect {
        self.canvas.get_crop()
    }

    fn set_drawing_color(&mut self, color: ggraphics::Color) {
        self.canvas.set_drawing_color(color);
    }

    fn get_drawing_color(&self) -> ggraphics::Color {
        self.canvas.get_drawing_color()
    }

    fn set_alpha(&mut self, alpha: f32) {
        self.text.set_alpha(alpha);
        self.balloon_inner.set_alpha(alpha);
        self.balloon_outer.set_alpha(alpha);
    }

    fn get_alpha(&self) -> f32 {
        self.text.get_alpha()
    }

    fn set_transform_offset(&mut self, offset: numeric::Point2f) {
        self.canvas.set_transform_offset(offset);
    }

    fn get_transform_offset(&self) -> numeric::Point2f {
        self.canvas.get_transform_offset()
    }

    fn get_texture_size(&self, ctx: &mut ggez::Context) -> numeric::Vector2f {
        self.canvas.get_texture_size(ctx)
    }

    fn replace_texture(&mut self, _: Rc<ggraphics::Image>) {}

    fn set_color(&mut self, color: ggraphics::Color) {
        self.canvas.set_color(color);
    }

    fn get_color(&mut self) -> ggraphics::Color {
        self.canvas.get_color()
    }
}

impl DrawableComponent for TextBalloon {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            ggraphics::draw(ctx, &self.mesh, ggraphics::DrawParam::default())?;
            self.text.draw(ctx)?;

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

pub struct SuzuMiniSightSilhouette {
    event_list: DelayEventList<Self>,
    background: MovableUniTexture,
    silhouette: TaskSilhouette,
    text_balloon: EffectableWrap<MovableWrap<TextBalloon>>,
    customer_dialogue: CustomerDialogue,
    canvas: SubScreen,
}

impl SuzuMiniSightSilhouette {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        rect: numeric::Rect,
        background: MovableUniTexture,
        t: Clock,
    ) -> Self {
        let mut text_balloon = Box::new(TextBalloon::new(
            ctx,
            numeric::Rect::new(280.0, 0.0, 200.0, 300.0),
            "",
            FontInformation::new(
                game_data.get_font(FontID::JpFude1),
                numeric::Vector2f::new(22.0, 22.0),
                ggraphics::Color::from_rgba_u32(0xff),
            ),
        ));
        text_balloon.hide();

        SuzuMiniSightSilhouette {
            event_list: DelayEventList::new(),
            background: background,
            silhouette: TaskSilhouette::new_empty(ctx, rect),
            text_balloon: EffectableWrap::new(
                MovableWrap::new(text_balloon, None, 0),
                vec![effect::fade_in(10, t)],
            ),
            customer_dialogue: CustomerDialogue::new(Vec::new(), Vec::new()),
            canvas: SubScreen::new(ctx, rect, 0, ggraphics::Color::from_rgba_u32(0x00000000)),
        }
    }

    fn replace_character(&mut self, chara: SimpleObject, name: String) {
        self.silhouette.change_character(chara).update_name(name);
    }

    pub fn new_customer_update(
        &mut self,
        _: &mut ggez::Context,
        chara: SimpleObject,
        name: String,
        dialogue: CustomerDialogue,
        t: Clock,
    ) {
        self.customer_dialogue = dialogue;

        let mut delay_time = 0;
        loop {
            let line = self.customer_dialogue.get_current_dialogue_line();
            delay_time += self.customer_dialogue.get_current_time_step();
            self.event_list.add(DelayEvent::new(
                Box::new(move |silhouette, ctx, _| {
                    silhouette.replace_text(ctx, &line);
                    silhouette
                        .text_balloon
                        .add_effect(vec![effect::fade_in(20, t + delay_time)]);
                }),
                t + delay_time,
            ));

            if self.customer_dialogue.last() {
                break;
            }

            self.customer_dialogue.next();
        }

        self.replace_character(chara, name);
    }

    fn run_effect(&mut self, ctx: &mut ggez::Context, game_data: &GameData, t: Clock) {
        while let Some(event) = self.event_list.move_top() {
            // 時間が来ていない場合は、取り出した要素をリストに戻して処理ループを抜ける
            if event.run_time > t {
                self.event_list.add(event);
                break;
            }

            // 所有権を移動しているため、selfを渡してもエラーにならない
            (event.func)(self, ctx, game_data);
        }

        if self.silhouette.is_some() {
            self.silhouette.get_object_mut().unwrap().move_with_func(t);
            self.silhouette.get_object_mut().unwrap().effect(ctx, t);
        }

        self.text_balloon
            .ref_wrapped_object_mut()
            .ref_wrapped_object_mut()
            .update_mesh(ctx);
        self.text_balloon.effect(ctx, t);
    }

    pub fn replace_text(&mut self, ctx: &mut ggez::Context, text: &str) {
        self.text_balloon
            .ref_wrapped_object_mut()
            .ref_wrapped_object_mut()
            .replace_text(ctx, text);
        self.text_balloon.appear();
    }

    pub fn run_hide_effect(&mut self, now: Clock) {
        //self.silhouette.get_object_mut().unwrap().add_effect(vec![effect::fade_out(20, now)]);
        self.text_balloon
            .add_effect(vec![effect::fade_out(20, now)]);
    }
}

impl DrawableComponent for SuzuMiniSightSilhouette {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.background.draw(ctx)?;
            if self.silhouette.is_some() {
                self.silhouette.draw(ctx)?;
            }

            self.text_balloon.draw(ctx)?;

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

impl DrawableObject for SuzuMiniSightSilhouette {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for SuzuMiniSightSilhouette {
    impl_texture_object_for_wrapped! {canvas}
}

impl Clickable for SuzuMiniSightSilhouette {
    fn clickable_status(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> ggez::input::mouse::MouseCursor {
        self.silhouette.clickable_status(ctx, point)
    }
}

impl OnDesk for SuzuMiniSightSilhouette {
    fn ondesk_whose(&self) -> i32 {
        0
    }

    fn click_data(&self, ctx: &mut ggez::Context, point: numeric::Point2f) -> HoldData {
        if self.silhouette.get_drawing_area(ctx).contains(point) {
            self.silhouette.click_data(ctx, point)
        } else {
            HoldData::None
        }
    }

    fn get_type(&self) -> OnDeskType {
        OnDeskType::Silhouette
    }
}

pub struct SuzuMiniSight {
    pub canvas: SubScreen,
    pub dragging: Option<DeskObject>,
    pub dropping: Vec<DeskObject>,
    pub dropping_to_desk: Vec<DeskObject>,
    pub silhouette: SuzuMiniSightSilhouette,
    object_handover_lock: bool,
}

impl SuzuMiniSight {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        rect: ggraphics::Rect,
        t: Clock,
    ) -> Self {
        SuzuMiniSight {
            canvas: SubScreen::new(ctx, rect, 0, ggraphics::Color::new(0.0, 0.0, 0.0, 0.0)),
            dragging: None,
            dropping: Vec::new(),
            dropping_to_desk: Vec::new(),
            silhouette: SuzuMiniSightSilhouette::new(
                ctx,
                game_data,
                rect,
                MovableUniTexture::new(
                    game_data.ref_texture(TextureID::Paper1),
                    numeric::Point2f::new(-100.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    move_fn::stop(),
                    0,
                ),
                t,
            ),
            object_handover_lock: false,
        }
    }

    pub fn silhouette_new_customer_update(
        &mut self,
        ctx: &mut ggez::Context,
        chara: SimpleObject,
        name: String,
        dialogue: CustomerDialogue,
        t: Clock,
    ) {
        self.silhouette
            .new_customer_update(ctx, chara, name, dialogue, t);
    }

    pub fn dragging_handler(&mut self, point: numeric::Point2f, last: numeric::Point2f) {
        if let Some(obj) = &mut self.dragging {
            obj.get_object_mut()
                .move_diff(numeric::Vector2f::new(point.x - last.x, point.y - last.y));
        }
    }

    fn check_object_drop(&self, ctx: &mut ggez::Context, desk_obj: &DeskObject) -> bool {
        if self.object_handover_lock {
            // 客への手渡しがロックされているので、手渡しが発生しないようにfalseを返す
            return false;
        } else {
            let area = desk_obj.get_object().get_drawing_area(ctx);
            return area.y + area.h < self.canvas.get_drawing_area(ctx).h;
        }
    }

    pub fn lock_object_handover(&mut self) {
        self.object_handover_lock = true;
    }

    pub fn unlock_object_handover(&mut self) {
        self.object_handover_lock = false;
    }

    pub fn finish_customer_event(&mut self, now: Clock) {
        self.silhouette.run_hide_effect(now);
    }

    pub fn update(&mut self, ctx: &mut ggez::Context, game_data: &GameData, t: Clock) {
        self.dropping.retain(|d| !d.get_object().is_stop());

        for d in &mut self.dropping {
            d.get_object_mut().move_with_func(t);
            d.get_object_mut().effect(ctx, t);
        }

        for d in &mut self.dropping_to_desk {
            d.get_object_mut().move_with_func(t);
            d.get_object_mut().effect(ctx, t);
        }

        self.silhouette.run_effect(ctx, game_data, t);
    }

    pub fn check_drop_desk(&mut self) -> Vec<DeskObject> {
        let mut drop_to_desk = Vec::new();

        let mut index = 0;
        while index < self.dropping_to_desk.len() {
            let stop = self
                .dropping_to_desk
                .get(index)
                .unwrap()
                .get_object()
                .is_stop();
            if stop {
                drop_to_desk.push(self.dropping_to_desk.swap_remove(index));
            }
            index += 1;
        }

        drop_to_desk
    }

    pub fn add_object(&mut self, obj: DeskObject) {
        self.dropping.push(obj);
    }

    pub fn has_dragging(&self) -> bool {
        self.dragging.is_some()
    }

    pub fn insert_dragging(&mut self, obj: DeskObject) {
        let d = std::mem::replace(&mut self.dragging, Some(obj));
        if d.is_some() {
            self.dropping.push(d.unwrap());
        }
    }

    pub fn unselect_dragging_object(&mut self, ctx: &mut ggez::Context, t: Clock) {
        if self.dragging.is_some() {
            let mut dragged = self.release_dragging().unwrap();

            if self.check_object_drop(ctx, &dragged) {
                dragged
                    .get_object_mut()
                    .override_move_func(move_fn::gravity_move(1.0, 10.0, 310.0, 0.3), t);
                dragged.get_object_mut().add_effect(vec![Box::new(
                    |obj: &mut dyn MovableObject, _: &ggez::Context, t: Clock| {
                        if obj.get_position().y > 350.0 {
                            obj.override_move_func(None, t);
                            EffectFnStatus::EffectFinish
                        } else {
                            EffectFnStatus::EffectContinue
                        }
                    },
                )]);
                self.dropping.push(dragged);
            } else {
                dragged
                    .get_object_mut()
                    .override_move_func(move_fn::gravity_move(1.0, 10.0, 310.0, 0.3), t);
                dragged.get_object_mut().add_effect(vec![Box::new(
                    |obj: &mut dyn MovableObject, _: &ggez::Context, t: Clock| {
                        if obj.get_position().y > 300.0 {
                            obj.override_move_func(None, t);
                            EffectFnStatus::EffectFinish
                        } else {
                            EffectFnStatus::EffectContinue
                        }
                    },
                )]);
                self.dropping_to_desk.push(dragged);
            }
        }
    }

    pub fn check_data_click(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> HoldData {
        let rpoint = self.canvas.relative_point(point);

        self.silhouette.click_data(ctx, rpoint)
    }

    pub fn release_dragging(&mut self) -> Option<DeskObject> {
        std::mem::replace(&mut self.dragging, None)
    }

    pub fn ref_dragging(&self) -> &Option<DeskObject> {
        &self.dragging
    }

    pub fn out_of_desk(&self, point: numeric::Point2f) -> bool {
        !self.canvas.contains(point)
    }

    pub fn check_mouse_cursor_status(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> MouseCursor {
        if self.canvas.get_drawing_area(ctx).contains(point) {
            let rpoint = self.canvas.relative_point(point);
            return self.silhouette.clickable_status(ctx, rpoint);
        }

        MouseCursor::Default
    }
}

impl DrawableComponent for SuzuMiniSight {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.silhouette.draw(ctx)?;

            for d in &mut self.dropping {
                d.get_object_mut().draw(ctx)?;
            }

            for d in &mut self.dropping_to_desk {
                d.get_object_mut().draw(ctx)?;
            }

            if let Some(ref mut d) = self.dragging {
                d.get_object_mut().draw(ctx)?;
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

impl DrawableObject for SuzuMiniSight {
    /// 描画開始地点を設定する
    fn set_position(&mut self, pos: numeric::Point2f) {
        self.canvas.set_position(pos);
    }

    /// 描画開始地点を返す
    fn get_position(&self) -> numeric::Point2f {
        self.canvas.get_position()
    }

    /// offsetで指定しただけ描画位置を動かす
    fn move_diff(&mut self, offset: numeric::Vector2f) {
        self.canvas.move_diff(offset)
    }
}

impl HoldData {
    pub fn is_none(&self) -> bool {
        match self {
            &HoldData::None => true,
            _ => false,
        }
    }

    pub fn is_some(&self) -> bool {
        match self {
            &HoldData::None => false,
            _ => true,
        }
    }
}

pub struct Goods {
    calendar: DrawableCalendar,
    canvas: SubScreen,
}

impl Goods {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData, pos_rect: numeric::Rect) -> Self {
        Goods {
            calendar: DrawableCalendar::new(
                ctx,
                game_data,
                numeric::Rect::new(0.0, 0.0, 100.0, 100.0),
                GensoDate::new(12, 12, 12),
                TextureID::Paper1,
            ),
            canvas: SubScreen::new(
                ctx,
                pos_rect,
                0,
                ggraphics::Color::from_rgba_u32(0x00000000),
            ),
        }
    }

    pub fn check_data_click(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> HoldData {
        let rpoint = self.canvas.relative_point(point);

        if self.calendar.get_drawing_area(ctx).contains(rpoint) {
            return self.calendar.click_data(ctx, rpoint);
        }

        HoldData::None
    }

    pub fn check_mouse_cursor_status(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> MouseCursor {
        if self.canvas.get_drawing_area(ctx).contains(point) {
            let rpoint = self.canvas.relative_point(point);
            return self.calendar.clickable_status(ctx, rpoint);
        }

        MouseCursor::Default
    }
}

impl DrawableComponent for Goods {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.calendar.draw(ctx)?;

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

pub struct KosuzuMemory {
    books_info: Vec<BookInformation>,
    customers_name: Vec<String>,
    dates: Vec<GensoDate>,
}

impl KosuzuMemory {
    pub fn new() -> Self {
	KosuzuMemory {
	    books_info: Vec::new(),
	    customers_name: Vec::new(),
	    dates: Vec::new(),
	}
    }

    pub fn add_book_info(&mut self, book_info: BookInformation) {
	self.books_info.push(book_info);
    }

    pub fn add_customer_name(&mut self, name: String) {
	self.customers_name.push(name);
    }

    pub fn add_date(&mut self, date: GensoDate) {
	self.dates.push(date);
    }

    pub fn get_book_info_remove(&mut self, index: usize) -> Option<BookInformation> {
	if self.books_info.len() <= index {
	    None
	} else {
	    Some(self.books_info.swap_remove(index))
	}
    }

    pub fn get_customer_name_remove(&mut self, index: usize) -> Option<String> {
	if self.customers_name.len() <= index {
	    None
	} else {
	    Some(self.customers_name.swap_remove(index))
	}
    }

    pub fn get_dates_remove(&mut self, index: usize) -> Option<GensoDate> {
	if self.dates.len() <= index {
	    None
	} else {
	    Some(self.dates.swap_remove(index))
	}
    }
}

#[derive(Clone)]
pub enum CustomerRequest {
    Borrowing(BorrowingInformation),
    Returning(ReturnBookInformation),
    Copying(CopyingRequestInformation),
}

pub struct ShelvingBookBox {
    pub canvas: SubScreen,
    pub shelved: Vec<DeskObject>,
    pub dragging: Option<DeskObject>,
    pub table_texture: SimpleObject,
}

impl ShelvingBookBox {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        rect: ggraphics::Rect,
    ) -> ShelvingBookBox {
        let mut dparam = ggraphics::DrawParam::default();
        dparam.dest = numeric::Point2f::new(rect.x, rect.y).into();

        ShelvingBookBox {
            canvas: SubScreen::new(ctx, rect, 0, ggraphics::Color::new(0.0, 0.0, 0.0, 0.0)),
            shelved: Vec::new(),
            dragging: None,
            table_texture: SimpleObject::new(
                MovableUniTexture::new(
                    game_data.ref_texture(TextureID::Wood1),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    move_fn::stop(),
                    0,
                ),
                Vec::new(),
            ),
        }
    }

    pub fn check_data_click(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> HoldData {
        let rpoint = self.canvas.relative_point(point);
        let mut clicked_data = HoldData::None;

        // オブジェクトは深度が深い順にソートされているので、
        // 逆順から検索していくことで、最も手前に表示されているオブジェクトを
        // 取り出すことができる
        for obj in self.shelved.iter_mut().rev() {
            let contains = obj.get_object().get_drawing_area(ctx).contains(rpoint);
            if contains {
                clicked_data = obj
                    .get_object_mut()
                    .ref_wrapped_object_mut()
                    .ref_wrapped_object_mut()
                    .click_data(ctx, rpoint);
                break;
            }
        }

        clicked_data
    }

    pub fn check_insert_data(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
        data: &HoldData,
    ) -> bool {
        let rpoint = self.canvas.relative_point(point);

        // オブジェクトは深度が深い順にソートされているので、
        // 逆順から検索していくことで、最も手前に表示されているオブジェクトを
        // 取り出すことができる
        for obj in self.shelved.iter_mut().rev() {
            let contains = obj.get_object().get_drawing_area(ctx).contains(rpoint);
            if contains {
                return obj
                    .get_object_mut()
                    .ref_wrapped_object_mut()
                    .ref_wrapped_object_mut()
                    .insert_data(ctx, rpoint, data);
            }
        }

        false
    }

    pub fn dragging_handler(&mut self, point: numeric::Point2f, last: numeric::Point2f) {
        if let Some(obj) = &mut self.dragging {
            obj.get_object_mut()
                .move_diff(numeric::Vector2f::new(point.x - last.x, point.y - last.y));
        }
    }

    pub fn unselect_dragging_object(&mut self, t: Clock) {
        if let Some(dragged) = &mut self.dragging {
            dragged
                .get_object_mut()
                .override_move_func(move_fn::gravity_move(1.0, 10.0, 310.0, 0.3), t);
            dragged.get_object_mut().add_effect(vec![Box::new(
                |obj: &mut dyn MovableObject, _: &ggez::Context, t: Clock| {
                    if obj.get_position().y > 350.0 {
                        obj.override_move_func(None, t);
                        EffectFnStatus::EffectFinish
                    } else {
                        EffectFnStatus::EffectContinue
                    }
                },
            )]);
            let dragged_object = std::mem::replace(&mut self.dragging, None);
            self.shelved.push(dragged_object.unwrap());
        }
    }

    pub fn update(&mut self, ctx: &mut ggez::Context, t: Clock) {
        for p in &mut self.shelved {
            p.get_object_mut().move_with_func(t);
            p.get_object_mut().effect(ctx, t);
        }
    }

    pub fn add_object(&mut self, obj: DeskObject) {
        self.shelved.push(obj);
    }

    pub fn add_customer_object_vec(&mut self, mut obj_vec: Vec<DeskObject>) {
        while obj_vec.len() != 0 {
            self.add_object(obj_vec.pop().unwrap());
        }
    }

    pub fn has_dragging(&self) -> bool {
        self.dragging.is_some()
    }

    pub fn insert_dragging(&mut self, obj: DeskObject) {
        let d = std::mem::replace(&mut self.dragging, Some(obj));
        if d.is_some() {
            self.add_object(d.unwrap());
        }
    }

    pub fn release_dragging(&mut self) -> Option<DeskObject> {
        std::mem::replace(&mut self.dragging, None)
    }

    pub fn ref_dragging(&self) -> &Option<DeskObject> {
        &self.dragging
    }

    pub fn out_of_desk(&self, point: numeric::Point2f) -> bool {
        !self.canvas.contains(point)
    }

    fn button_up_handler(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        t: Clock,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        let rpoint = self.canvas.relative_point(point);

        for dobj in &mut self.shelved {
            if dobj.get_object_mut().get_drawing_area(ctx).contains(rpoint) {
                dobj.get_object_mut()
                    .ref_wrapped_object_mut()
                    .ref_wrapped_object_mut()
                    .button_up(ctx, game_data, t, button, rpoint);
            }
        }
    }

    pub fn check_mouse_cursor_status(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> MouseCursor {
        if self.canvas.get_drawing_area(ctx).contains(point) {
            let rpoint = self.canvas.relative_point(point);

            // オブジェクトは深度が深い順にソートされているので、
            // 逆順から検索していくことで、最も手前に表示されているオブジェクトを
            // 取り出すことができる
            for obj in self.shelved.iter_mut().rev() {
                if obj.get_object().get_drawing_area(ctx).contains(rpoint) {
                    return MouseCursor::Grab;
                }
            }
        }

        MouseCursor::Default
    }
}

impl DrawableComponent for ShelvingBookBox {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.table_texture.draw(ctx)?;

            for obj in &mut self.shelved {
                obj.get_object_mut().draw(ctx)?;
            }

            if let Some(ref mut d) = self.dragging {
                d.get_object_mut().draw(ctx)?;
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

    /// 描画順序を設定する
    fn set_drawing_depth(&mut self, depth: i8) {
        self.canvas.set_drawing_depth(depth);
    }

    /// 描画順序を返す
    fn get_drawing_depth(&self) -> i8 {
        self.canvas.get_drawing_depth()
    }
}

pub struct RingRingEffect {
    ring: shape::Circle,
    mesh: ggraphics::Mesh,
    drwob_essential: DrawableObjectEssential,
}

impl RingRingEffect {
    pub fn new(ctx: &mut ggez::Context, pos: numeric::Point2f) -> Self {
        let ring = shape::Circle::new(
            pos,
            200.0,
            20.0,
            ggraphics::DrawMode::fill(),
            ggraphics::Color::from_rgb_u32(0xffff00ff),
        );
        let mut mesh_builder = ggraphics::MeshBuilder::new();
        ring.add_to_builder(&mut mesh_builder);

        RingRingEffect {
            ring: ring,
            mesh: mesh_builder.build(ctx).unwrap(),
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }
}

impl DrawableComponent for RingRingEffect {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            ggraphics::draw(ctx, &self.mesh, ggraphics::DrawParam::default()).unwrap();
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
