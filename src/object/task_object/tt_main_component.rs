use std::collections::HashMap;

use ggez::graphics as ggraphics;
use ggez::input::mouse::CursorIcon;

use torifune::core::Clock;
use torifune::graphics::drawable::*;
use torifune::graphics::object::shape;
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::graphics::object::*;
use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;
use torifune::roundup2f;
use torifune::{debug, numeric, mintp_new, mintp};

use crate::core::FontID;
use crate::flush_delay_event;
use crate::flush_delay_event_and_redraw_check;
use crate::object::util_object::*;
use crate::object::{effect, move_fn};
use crate::scene::*;
use crate::set_table_frame_cell_center;

use super::tt_menu_component::*;
use super::tt_sub_component::*;
use super::Clickable;
use crate::core::*;

pub enum TaskTableStagingObject {
    BorrowingRecordBook(BorrowingRecordBook),
}

impl TaskTableStagingObject {
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
    pub dragging: Option<TaskItem>,
    pub table_texture: SimpleObject,
    money_box: MovableWrap<MoneyBox>,
    event_list: DelayEventList<Self>,
    appearance_frame: TileBatchFrame,
    money_box_is_pulled: bool,
    draw_request: DrawRequest,
}

impl DeskObjects {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, rect: ggraphics::Rect) -> DeskObjects {
        let appr_frame = TileBatchFrame::new(
            ctx.resource,
            TileBatchTextureID::BlackFrame,
            numeric::Rect::new(0.0, 0.0, rect.w, rect.h),
            numeric::Vector2f::new(1.0, 1.0),
            0,
        );

        let desk_objects = DeskObjectContainer::new();

        DeskObjects {
            canvas: SubScreen::new(
                ctx.context,
                rect,
                0,
                ggraphics::Color::new(0.0, 0.0, 0.0, 0.0),
            ),
            desk_objects: desk_objects,
            dragging: None,
            table_texture: SimpleObject::new(
                MovableUniTexture::new(
                    Box::new(UniTexture::new(
                        ctx.ref_texture(TextureID::Wood1),
                        numeric::Point2f::new(0.0, 0.0),
                        numeric::Vector2f::new(1.0, 1.0),
                        0.0,
                        0,
                    )),
                    move_fn::stop(),
                    0,
                ),
                Vec::new(),
            ),
            money_box: MovableWrap::new(
                Box::new(MoneyBox::new(
                    ctx,
                    numeric::Rect::new(rect.w - 300.0, -270.0, 300.0, 300.0),
                    0,
                )),
                None,
                0,
            ),
            event_list: DelayEventList::new(),
            appearance_frame: appr_frame,
            money_box_is_pulled: false,
            draw_request: DrawRequest::InitDraw,
        }
    }

    fn drag_current_object<'a>(&mut self, ctx: &mut SuzuContext<'a>, point: numeric::Point2f) {
        if let Some(obj) = &mut self.dragging {
            let area = match obj {
                TaskItem::Book(item) => item.get_large_object().get_drawing_area(ctx.context),
                TaskItem::Texture(item) => item.get_large_object().get_drawing_area(ctx.context),
                TaskItem::Coin(item) => item.get_large_object().get_drawing_area(ctx.context),
            };

            let canvas_size = self.canvas.get_drawing_size(ctx.context);
            let rpoint = self.canvas.relative_point(point);

            if (canvas_size.x - rpoint.x) <= 0.0 {
                return;
            }

            let drag_point = obj.get_drag_point();
            let next_position = numeric::Point2f::new(
                rpoint.x - (area.w * drag_point.x),
                rpoint.y - (area.h * drag_point.y),
            );

            obj.get_object_mut().set_position(next_position);

            if obj.is_shelving_box_handover_locked() {
                if next_position.x + area.w > canvas_size.x {
                    obj.get_object_mut()
                        .set_position(numeric::Point2f::new(canvas_size.x - area.w, next_position.y));
                }
            }
	    
	    if next_position.x < 0.0 {
                obj.get_object_mut()
                    .set_position(numeric::Point2f::new(0.0, next_position.y));
            }

	    if next_position.y + area.h > canvas_size.y {
		let np = obj.get_object_mut().get_position();
                obj.get_object_mut()
                    .set_position(numeric::Point2f::new(np.x, canvas_size.y - area.h));
            }

            self.draw_request = DrawRequest::Draw;
        }
    }

    pub fn dragging_handler<'a>(&mut self, ctx: &mut SuzuContext<'a>, point: numeric::Point2f) {
        self.drag_current_object(ctx, point);
    }

    pub fn select_dragging_object<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
    ) {
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
            if obj.get_object().contains(ctx.context, rpoint) {
                obj.as_movable_object_mut().override_move_func(None, 0);
                dragging_object_index = self.desk_objects.len() - index - 1;
                drag_start = true;
                break;
            }
        }

        if drag_start {
            // 元々、最前面に表示されていたオブジェクトのdepthに設定する
            let mut dragging = self
                .desk_objects
                .get_raw_container_mut()
                .swap_remove(dragging_object_index);

            dragging.get_object_mut().start_dragging(ctx);

            let object_area = match &dragging {
                TaskItem::Book(item) => item.get_large_object().get_drawing_area(ctx.context),
                TaskItem::Texture(item) => item.get_large_object().get_drawing_area(ctx.context),
                TaskItem::Coin(item) => item.get_large_object().get_drawing_area(ctx.context),
            };

            dragging.set_drag_point(numeric::Vector2f::new(
                (rpoint.x - object_area.x) / object_area.w,
                (rpoint.y - object_area.y) / object_area.h,
            ));

            self.dragging = Some(dragging);

            self.desk_objects.sort_with_depth();
            self.draw_request = DrawRequest::Draw;
        }
    }

    fn moneybox_hand_over<'a>(&mut self, mut item: TaskItem) {
        let point = item.get_object().get_position();
        let mbox_position = self.money_box.relative_point(point);
        item.get_object_mut().set_position(mbox_position);
        self.money_box.add_coin(item);
    }

    fn try_insert_money_box<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        item: TaskItem,
    ) -> Option<TaskItem> {
        let moneybox_area = self.money_box.get_drawing_area(ctx.context);
        let item_area = item.get_object().get_drawing_area(ctx.context);

        if !moneybox_area.contains(item_area.point())
            || !moneybox_area.contains(mintp_new!(item_area.right(), item_area.bottom()))
            || !self.money_box_is_pulled
        {
            return Some(item);
        }

        match item {
            TaskItem::Coin(_) => {
                self.moneybox_hand_over(item);
                None
            }
            _ => Some(item),
        }
    }

    pub fn unselect_dragging_object<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        if self.dragging.is_some() {
            let dragged = self.release_dragging().unwrap();

            if let Some(mut item) = self.try_insert_money_box(ctx, dragged) {
                let min = self.desk_objects.get_minimum_depth();
                item.get_object_mut().set_drawing_depth(min);
                item.get_object_mut().finish_dragging(ctx);
                self.desk_objects.change_depth_equally(1);

                self.desk_objects.add_item(item);
            }

            self.desk_objects.sort_with_depth();
            self.draw_request = DrawRequest::Draw;
	    ctx.process_utility.redraw();
        }
    }

    ///
    /// # 再描画要求有り
    ///
    pub fn update<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        flush_delay_event_and_redraw_check!(self, self.event_list, ctx, t, {});

        for p in self.desk_objects.get_raw_container_mut() {
            if !p.as_movable_object().is_stop() || !p.as_effectable_object().is_empty_effect() {
                self.draw_request = DrawRequest::Draw;
                ctx.process_utility.redraw();
            }

            p.as_movable_object_mut().move_with_func(t);
            p.as_effectable_object().effect(ctx.context, t);
        }

        if !self.money_box.is_stop() {
            self.money_box.move_with_func(t);
            self.draw_request = DrawRequest::Draw;
            ctx.process_utility.redraw();
        }
    }

    pub fn check_clicked_desk_object_type<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
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
            if obj.get_object().contains(ctx.context, rpoint) {
                click_flag = true;

                object_type = Some(obj.get_object().get_type());
                break;
            }
        }

        if click_flag {
            self.desk_objects.sort_with_depth();
            self.draw_request = DrawRequest::Draw;
        }

        object_type
    }

    pub fn add_object(&mut self, obj: TaskItem) {
        self.draw_request = DrawRequest::Draw;
        self.desk_objects.add_item(obj);
        self.desk_objects.sort_with_depth();
    }

    pub fn add_customer_object(&mut self, obj: TaskItem) {
        self.add_object(obj);
    }

    pub fn add_customer_object_vec(&mut self, mut obj_vec: Vec<TaskItem>) {
        while obj_vec.len() != 0 {
            self.add_object(obj_vec.pop().unwrap());
        }
    }

    pub fn has_dragging(&self) -> bool {
        self.dragging.is_some()
    }

    pub fn insert_dragging(&mut self, obj: TaskItem) {
        let d = std::mem::replace(&mut self.dragging, Some(obj));
        if d.is_some() {
            self.desk_objects.add_item(d.unwrap());
            self.draw_request = DrawRequest::Draw;
        }
    }

    pub fn release_dragging(&mut self) -> Option<TaskItem> {
        self.draw_request = DrawRequest::Draw;
        std::mem::replace(&mut self.dragging, None)
    }

    pub fn ref_dragging(&self) -> Option<&TaskItem> {
        self.dragging.as_ref()
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

    pub fn click_handler<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        t: Clock,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
    ) -> bool {
        if !self.canvas.contains(point) {
            return false;
        }

        let rpoint = self.canvas.relative_point(point);

        for dobj in self.desk_objects.get_raw_container_mut().iter_mut().rev() {
            if dobj.get_object_mut().contains(ctx.context, rpoint) {
                self.draw_request = DrawRequest::Draw;
                dobj.get_object_mut().button_up(ctx, t, button, rpoint);

                return true;
            }
        }

        let canvas_size = self.canvas.get_drawing_size(ctx.context);
        if self.money_box.contains(ctx.context, rpoint) {
            if !self.money_box_is_pulled {
                self.money_box.override_move_func(
                    move_fn::devide_distance(
                        numeric::Point2f::new(canvas_size.x - 300.0, 0.0),
                        0.3,
                    ),
                    t,
                );
            } else {
                self.money_box.override_move_func(
                    move_fn::devide_distance(
                        numeric::Point2f::new(canvas_size.x - 300.0, -270.0),
                        0.3,
                    ),
                    t,
                );
            }

            self.money_box_is_pulled = !self.money_box_is_pulled;
        } else {
            if self.money_box_is_pulled {
                self.money_box.override_move_func(
                    move_fn::devide_distance(
                        numeric::Point2f::new(canvas_size.x - 300.0, -270.0),
                        0.3,
                    ),
                    t,
                );
                self.money_box_is_pulled = false;
            }
        }

        return false;
    }

    pub fn check_mouse_cursor_status(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> CursorIcon {
        if self.canvas.get_drawing_area(ctx).contains(mintp!(point)) {
            let rpoint = self.canvas.relative_point(point);

            // オブジェクトは深度が深い順にソートされているので、
            // 逆順から検索していくことで、最も手前に表示されているオブジェクトを
            // 取り出すことができる
            for obj in self.desk_objects.get_raw_container_mut().iter_mut().rev() {
                if obj.get_object().get_drawing_area(ctx).contains(mintp!(rpoint)) {
                    return CursorIcon::Grabbing;
                }
            }

            if let Some(dragging) = self.dragging.as_ref() {
                if dragging.get_object().contains(ctx, rpoint) {
                    return CursorIcon::Grabbing;
                }
            }
        }

        CursorIcon::Default
    }

    pub fn get_desk_objects_list(&self) -> &Vec<TaskItem> {
        self.desk_objects.get_raw_container()
    }
}

impl DrawableComponent for DeskObjects {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            if self.draw_request != DrawRequest::Skip {
                self.draw_request = DrawRequest::Skip;
                sub_screen::stack_screen(ctx, &self.canvas);

                self.table_texture.draw(ctx)?;

                for obj in self.desk_objects.get_raw_container_mut() {
                    obj.get_object_mut().draw(ctx)?;
                }

                self.money_box.draw(ctx)?;

                if let Some(d) = self.dragging.as_mut() {
                    d.get_object_mut().draw(ctx)?;
                }

                self.appearance_frame.draw(ctx)?;

                sub_screen::pop_screen(ctx);
            }

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
    impl_drawable_object_for_wrapped! {canvas}
}

struct TaskSilhouette {
    character: Option<SimpleObject>,
    name: Option<String>,
    canvas: SubScreen,
}

impl TaskSilhouette {
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
    ) -> ggez::input::mouse::CursorIcon {
        if let Some(character) = &self.character {
            if character.get_drawing_area(ctx).contains(mintp!(point)) {
                return CursorIcon::Grab;
            }
        }

        CursorIcon::Default
    }
}

impl OnDesk for TaskSilhouette {
    fn ondesk_whose(&self) -> i32 {
        0
    }

    fn click_hold_data(&self, _: &mut ggez::Context, _: numeric::Point2f) -> HoldData {
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

#[derive(Clone, PartialEq)]
pub enum TextBalloonPhraseType {
    SimplePhrase,
    CustomerName(String),
    RentalLimit(RentalLimit),
}

pub struct CustomerDialogue {
    dialogue: Vec<String>,
    time_step: Vec<u64>,
    phrase_type: Vec<TextBalloonPhraseType>,
    current_index: usize,
}

impl CustomerDialogue {
    pub fn new(dialogue: Vec<String>, time_step: Vec<u64>) -> Self {
        CustomerDialogue {
            phrase_type: vec![TextBalloonPhraseType::SimplePhrase; dialogue.len()],
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
    back_canvas: SubScreen,
    text: VerticalText,
    phrase_type: TextBalloonPhraseType,
    text_balloon: shape::FramedLeadingRect,
}

impl TextBalloon {
    pub fn new(
        ctx: &mut ggez::Context,
        balloon_rect: numeric::Rect,
        text: &str,
        phrase_type: TextBalloonPhraseType,
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

        let balloon = shape::FramedLeadingRect::new(
            ctx,
            numeric::Rect::new(25.0, 0.0, vtext_size.x + 100.0, vtext_size.y + 50.0),
	    0.0,
            2.0,
	    numeric::Vector2f::new(-25.0, 50.0),
	    numeric::Vector2f::new(15.0, 15.0),
            ggraphics::Color::WHITE,
            ggraphics::Color::from_rgb_u32(0x111111),
            0,
        );

	let balloon_area = balloon.get_drawing_area();
	vtext.make_center(
            ctx,
            numeric::Point2f::new(balloon_area.x + (balloon_area.w / 2.0), balloon_area.y + (balloon_area.h / 2.0)),
        );

        TextBalloon {
            back_canvas: SubScreen::new(
                ctx,
                balloon_rect,
                0,
                ggraphics::Color::from_rgba_u32(0x00),
            ),
            canvas: SubScreen::new(ctx, balloon_rect, 0, ggraphics::Color::from_rgba_u32(0x00)),
            text: vtext,
            phrase_type: phrase_type,
            text_balloon: balloon,
        }
    }

    pub fn replace_text(
        &mut self,
        ctx: &mut ggez::Context,
        text: &str,
        phrase_type: TextBalloonPhraseType,
    ) {
        self.text.replace_text(text.to_string());
        let vtext_size = self.text.get_drawing_size(ctx);

        self.text_balloon = shape::FramedLeadingRect::new(
            ctx,
            numeric::Rect::new(25.0, 0.0, vtext_size.x + 100.0, vtext_size.y + 50.0),
	    0.0,
            2.0,
	    numeric::Vector2f::new(-25.0, 50.0),
	    numeric::Vector2f::new(15.0, 15.0),
            ggraphics::Color::WHITE,
            ggraphics::Color::from_rgb_u32(0x111111),
            0,
        );

	let balloon_area = self.text_balloon.get_drawing_area();	
	self.text.make_center(
            ctx,
            numeric::Point2f::new(balloon_area.x + (balloon_area.w / 2.0), balloon_area.y + (balloon_area.h / 2.0)),
        );


        self.phrase_type = phrase_type;
    }

    pub fn get_phrase_type(&self) -> &TextBalloonPhraseType {
        &self.phrase_type
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

    fn replace_texture(&mut self, _: ggraphics::Image) {}

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
            sub_screen::stack_screen(ctx, &self.back_canvas);

            self.text_balloon.draw(ctx)?;

            sub_screen::pop_screen(ctx);
            self.back_canvas.draw(ctx).unwrap();

            sub_screen::stack_screen(ctx, &self.canvas);

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
    background: UniTexture,
    silhouette: TaskSilhouette,
    text_balloon: EffectableWrap<MovableWrap<TextBalloon>>,
    customer_dialogue: CustomerDialogue,
    chat_box: ChatBox,
    canvas: SubScreen,
}

impl SuzuMiniSightSilhouette {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        rect: numeric::Rect,
        background: UniTexture,
        t: Clock,
    ) -> Self {
        let mut text_balloon = Box::new(TextBalloon::new(
            ctx.context,
            numeric::Rect::new(330.0, 10.0, 200.0, 300.0),
            "",
            TextBalloonPhraseType::SimplePhrase,
            FontInformation::new(
                ctx.resource.get_font(FontID::JpFude1),
                numeric::Vector2f::new(22.0, 22.0),
                ggraphics::Color::from_rgba_u32(0xff),
            ),
        ));
        text_balloon.hide();

        let mut chat_box = ChatBox::new(
            ctx,
            numeric::Rect::new(766.0, 0.0, 300.0, 300.0),
            0,
            None,
            Some("小鈴".to_string()),
        );
        chat_box.add_message_as_mine(ctx, "いらっしゃいませ".to_string());

        SuzuMiniSightSilhouette {
            event_list: DelayEventList::new(),
            background: background,
            silhouette: TaskSilhouette::new_empty(
                ctx.context,
                numeric::Rect::new(0.0, 0.0, 350.0, 300.0),
            ),
            text_balloon: EffectableWrap::new(
                MovableWrap::new(text_balloon, None, 0),
                vec![effect::fade_in(10, t)],
            ),
            customer_dialogue: CustomerDialogue::new(Vec::new(), Vec::new()),
            chat_box: chat_box,
            canvas: SubScreen::new(
                ctx.context,
                numeric::Rect::new(0.0, 0.0, rect.w, rect.h),
                0,
                ggraphics::Color::from_rgba_u32(0x00000000),
            ),
        }
    }

    fn mouse_wheel_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        x: f32,
        y: f32,
    ) {
        let rpoint = self.canvas.relative_point(point);
        if self.chat_box.contains(ctx.context, rpoint) {
            self.chat_box
                .scroll(ctx, point, numeric::Vector2f::new(x, y));
        }
    }

    fn replace_character(&mut self, chara: SimpleObject, name: String) {
        self.silhouette.change_character(chara).update_name(name);
    }

    pub fn get_text_balloon_phrase_type(&self) -> &TextBalloonPhraseType {
        &self.text_balloon.get_phrase_type()
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
                Box::new(move |silhouette, ctx, called| {
                    silhouette.replace_text(
                        ctx.context,
                        &line,
                        TextBalloonPhraseType::SimplePhrase,
                    );

                    silhouette
                        .text_balloon
                        .add_effect(vec![effect::fade_in(20, called)]);
                    silhouette.chat_box.add_message_as_partner(ctx, line);
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

    fn run_effect<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) -> DrawRequest {
        let mut draw_request = DrawRequest::Skip;

        if flush_delay_event!(self, self.event_list, ctx, t) > 0 {
            draw_request = DrawRequest::Draw;
        }

        if let Some(obj) = self.silhouette.get_object_mut() {
            if !obj.is_stop() || !obj.is_empty_effect() {
                draw_request = DrawRequest::Draw;
            }
            obj.move_with_func(t);
            obj.effect(ctx.context, t);
        }

        if !self.text_balloon.is_empty_effect() {
            self.text_balloon.effect(ctx.context, t);
            draw_request = DrawRequest::Draw;
        }

        draw_request
    }

    pub fn replace_text(
        &mut self,
        ctx: &mut ggez::Context,
        text: &str,
        phrase_type: TextBalloonPhraseType,
    ) {
        self.text_balloon.replace_text(ctx, text, phrase_type);
        self.text_balloon.appear();
    }

    pub fn insert_new_balloon_phrase(
        &mut self,
        text: String,
        phrase_type: TextBalloonPhraseType,
        delay_time: Clock,
        now: Clock,
    ) {
        self.event_list.add(DelayEvent::new(
            Box::new(move |silhouette, ctx, _| {
                silhouette.replace_text(ctx.context, &text, phrase_type);
                silhouette
                    .text_balloon
                    .add_effect(vec![effect::fade_in(20, now + delay_time)]);
            }),
            now + delay_time,
        ));
    }

    pub fn insert_kosuzu_message_in_chatbox<'a>(&mut self, ctx: &mut SuzuContext<'a>, s: String) {
        self.chat_box.add_message_as_mine(ctx, s);
    }

    pub fn insert_customer_message_in_chatbox<'a>(&mut self, ctx: &mut SuzuContext<'a>, s: String) {
        self.chat_box.add_message_as_partner(ctx, s);
    }

    pub fn set_partner_name_to_chatbox(&mut self, name: String) {
        self.chat_box.set_partner_name(name);
    }

    pub fn run_hide_effect(&mut self, now: Clock) {
        //self.silhouette.get_object_mut().unwrap().add_effect(vec![effect::fade_out(20, now)]);
        self.text_balloon
            .add_effect(vec![effect::fade_out(20, now)]);
    }

    pub fn contains_text_balloon(&self, ctx: &mut ggez::Context, point: numeric::Point2f) -> bool {
        let rpoint = self.canvas.relative_point(point);
        self.text_balloon.contains(ctx, rpoint)
    }

    pub fn contains_character_silhouette(
        &self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> bool {
        let rpoint = self.canvas.relative_point(point);
        self.silhouette.contains(ctx, rpoint)
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
            self.chat_box.draw(ctx)?;

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
    ) -> ggez::input::mouse::CursorIcon {
        self.silhouette.clickable_status(ctx, point)
    }
}

impl OnDesk for SuzuMiniSightSilhouette {
    fn ondesk_whose(&self) -> i32 {
        0
    }

    fn click_hold_data(&self, ctx: &mut ggez::Context, point: numeric::Point2f) -> HoldData {
        if self.silhouette.get_drawing_area(ctx).contains(mintp!(point)) {
            self.silhouette.click_hold_data(ctx, point)
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
    pub dragging: Option<TaskItem>,
    pub dropping: Vec<TaskItem>,
    pub dropping_to_desk: Vec<TaskItem>,
    pub silhouette: SuzuMiniSightSilhouette,
    appearance_frame: TileBatchFrame,
    draw_request: DrawRequest,
}

impl SuzuMiniSight {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, rect: ggraphics::Rect, t: Clock) -> Self {
        let appr_frame = TileBatchFrame::new(
            ctx.resource,
            TileBatchTextureID::BlackFrame,
            numeric::Rect::new(0.0, 0.0, rect.w, rect.h),
            numeric::Vector2f::new(1.0, 1.0),
            0,
        );

        let silhouette_paper_texture = UniTexture::new(
            ctx.ref_texture(TextureID::Library),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(0.95, 0.95),
            0.0,
            0,
        );
        let silhouette = SuzuMiniSightSilhouette::new(ctx, rect, silhouette_paper_texture, t);

        SuzuMiniSight {
            canvas: SubScreen::new(ctx.context, rect, 0, ggraphics::Color::from_rgba_u32(0)),
            dragging: None,
            dropping: Vec::new(),
            dropping_to_desk: Vec::new(),
            silhouette: silhouette,
            appearance_frame: appr_frame,
            draw_request: DrawRequest::InitDraw,
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
        self.draw_request = DrawRequest::Draw;
    }

    pub fn count_not_forbidden_book_items(&self, kosuzu_memory: &KosuzuMemory) -> usize {
        let mut count = 0;

        if let Some(dragging) = self.dragging.as_ref() {
            match dragging {
                TaskItem::Book(item) => {
                    if !kosuzu_memory.is_in_blacklist(item.get_large_object().get_book_info()) {
                        count += 1;
                    }
                }
                _ => (),
            }
        }

        for obj in self.dropping_to_desk.iter() {
            match obj {
                TaskItem::Book(item) => {
                    if !kosuzu_memory.is_in_blacklist(item.get_large_object().get_book_info()) {
                        count += 1;
                    }
                }
                _ => (),
            }
        }

        count
    }

    pub fn dragging_handler<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        last: numeric::Point2f,
    ) {
        if let Some(obj) = &mut self.dragging {
            obj.get_object_mut()
                .move_diff(numeric::Vector2f::new(point.x - last.x, point.y - last.y));

            let obj_area = obj.get_object().get_drawing_area(ctx.context);
            if obj_area.right() > 766.0 {
                obj.get_object_mut()
                    .set_position(numeric::Point2f::new(766.0 - obj_area.w, obj_area.y));
            }
            if obj_area.x < 0.0 {
                obj.get_object_mut()
                    .set_position(numeric::Point2f::new(0.0, obj_area.y));
            }

            self.draw_request = DrawRequest::Draw;
        }
    }

    fn check_object_drop(&self, ctx: &mut ggez::Context, desk_obj: &TaskItem) -> bool {
        if desk_obj.is_handover_locked() {
            // 客への手渡しがロックされているので、手渡しが発生しないようにfalseを返す
            return false;
        } else {
            let area = desk_obj.get_object().get_drawing_area(ctx);
            return area.y + area.h < self.canvas.get_drawing_area(ctx).h;
        }
    }

    pub fn finish_customer_event(&mut self, now: Clock) {
        self.silhouette.run_hide_effect(now);
        self.draw_request = DrawRequest::Draw;
    }

    ///
    /// # 再描画要求有り
    ///
    pub fn update<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        self.dropping.retain(|d| !d.as_movable_object().is_stop());

        for d in &mut self.dropping {
            if !d.as_movable_object().is_stop() || !d.as_effectable_object().is_empty_effect() {
                self.draw_request = DrawRequest::Draw;
                ctx.process_utility.redraw();
            }
            d.as_movable_object_mut().move_with_func(t);
            d.as_effectable_object().effect(ctx.context, t);
        }

        for d in &mut self.dropping_to_desk {
            if !d.as_movable_object().is_stop() || !d.as_effectable_object().is_empty_effect() {
                self.draw_request = DrawRequest::Draw;
                ctx.process_utility.redraw();
            }
            d.as_movable_object_mut().move_with_func(t);
            d.as_effectable_object().effect(ctx.context, t);
        }

        if self.silhouette.run_effect(ctx, t) == DrawRequest::Draw {
            self.draw_request = DrawRequest::Draw;
            ctx.process_utility.redraw();
        }
    }

    pub fn check_drop_desk(&mut self) -> Vec<TaskItem> {
        let mut drop_to_desk = Vec::new();

        let mut index = 0;
        while index < self.dropping_to_desk.len() {
            let stop = self
                .dropping_to_desk
                .get(index)
                .unwrap()
                .as_movable_object()
                .is_stop();
            if stop {
                self.draw_request = DrawRequest::Draw;
                drop_to_desk.push(self.dropping_to_desk.swap_remove(index));
            }
            index += 1;
        }

        drop_to_desk
    }

    pub fn add_object(&mut self, obj: TaskItem) {
        self.draw_request = DrawRequest::Draw;
        self.dropping.push(obj);
    }

    pub fn has_dragging(&self) -> bool {
        self.dragging.is_some()
    }

    pub fn ref_dragging(&self) -> Option<&TaskItem> {
        self.dragging.as_ref()
    }

    pub fn insert_dragging(&mut self, obj: TaskItem) {
        let d = std::mem::replace(&mut self.dragging, Some(obj));
        if d.is_some() {
            self.draw_request = DrawRequest::Draw;
            self.dropping.push(d.unwrap());
        }
    }

    pub fn mouse_wheel_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        x: f32,
        y: f32,
    ) {
        let rpoint = self.canvas.relative_point(point);
        self.silhouette.mouse_wheel_event(ctx, rpoint, x, y);
        self.draw_request = DrawRequest::Draw;
    }

    pub fn unselect_dragging_object(&mut self, ctx: &mut ggez::Context, t: Clock) {
        if self.dragging.is_some() {
            let mut dragged = self.release_dragging().unwrap();

            if self.check_object_drop(ctx, &dragged) {
                dragged
                    .as_movable_object_mut()
                    .override_move_func(move_fn::gravity_move(1.0, 10.0, 310.0, 0.5), t);
                dragged.as_effectable_object().add_effect(vec![Box::new(
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
                    .as_movable_object_mut()
                    .override_move_func(move_fn::gravity_move(1.0, 10.0, 310.0, 0.5), t);
                dragged.as_effectable_object().add_effect(vec![Box::new(
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
        self.draw_request = DrawRequest::Draw;

        self.silhouette.click_hold_data(ctx, rpoint)
    }

    pub fn release_dragging(&mut self) -> Option<TaskItem> {
        self.draw_request = DrawRequest::Draw;
        std::mem::replace(&mut self.dragging, None)
    }

    pub fn out_of_desk(&self, point: numeric::Point2f) -> bool {
        !self.canvas.contains(point)
    }

    pub fn check_mouse_cursor_status(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> CursorIcon {
        if self.canvas.get_drawing_area(ctx).contains(mintp!(point)) {
            let rpoint = self.canvas.relative_point(point);
            return self.silhouette.clickable_status(ctx, rpoint);
        }

        CursorIcon::Default
    }
}

impl DrawableComponent for SuzuMiniSight {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            if self.draw_request != DrawRequest::Skip {
                self.draw_request = DrawRequest::Skip;
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

                self.appearance_frame.draw(ctx)?;

                sub_screen::pop_screen(ctx);
            }
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

#[derive(Clone)]
pub enum CustomerRequest {
    Borrowing(BorrowingInformation),
    Returning(ReturnBookInformation),
}

impl CustomerRequest {
    pub fn get_customer_name(&self) -> String {
        match self {
            CustomerRequest::Borrowing(info) => info.borrower.clone(),
            CustomerRequest::Returning(info) => info.borrower.clone(),
        }
    }
}

impl ToString for CustomerRequest {
    fn to_string(&self) -> String {
        match self {
            CustomerRequest::Borrowing(_) => "貸出",
            CustomerRequest::Returning(_) => "返却",
        }
        .to_string()
    }
}

#[derive(Clone)]
pub enum CustomerRequestOrder {
    BorrowingOrder,
    ReturningOrder,
}

pub struct ShelvingBookBox {
    pub canvas: SubScreen,
    pub shelved: Vec<TaskItem>,
    pub dragging: Option<TaskItem>,
    pub table_texture: SimpleObject,
    box_back: UniTexture,
    box_front: UniTexture,
    appearance_frame: TileBatchFrame,
    draw_request: DrawRequest,
}

impl ShelvingBookBox {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, rect: ggraphics::Rect) -> ShelvingBookBox {
        let box_back = UniTexture::new(
            ctx.ref_texture(TextureID::BookBoxBack),
            numeric::Point2f::new(0.0, rect.h - 300.0),
            numeric::Vector2f::new(0.586, 0.586),
            0.0,
            0,
        );

        let box_front = UniTexture::new(
            ctx.ref_texture(TextureID::BookBoxFront),
            numeric::Point2f::new(0.0, rect.h - 300.0),
            numeric::Vector2f::new(0.586, 0.586),
            0.0,
            0,
        );

        let appr_frame = TileBatchFrame::new(
            ctx.resource,
            TileBatchTextureID::BlackFrame,
            numeric::Rect::new(0.0, 0.0, rect.w, rect.h),
            numeric::Vector2f::new(1.0, 1.0),
            0,
        );

        ShelvingBookBox {
            canvas: SubScreen::new(
                ctx.context,
                rect,
                0,
                ggraphics::Color::new(0.0, 0.0, 0.0, 0.0),
            ),
            shelved: Vec::new(),
            dragging: None,
            table_texture: SimpleObject::new(
                MovableUniTexture::new(
                    Box::new(UniTexture::new(
                        ctx.ref_texture(TextureID::Wood1),
                        numeric::Point2f::new(0.0, 0.0),
                        numeric::Vector2f::new(1.0, 1.0),
                        0.0,
                        0,
                    )),
                    move_fn::stop(),
                    0,
                ),
                Vec::new(),
            ),
            box_front: box_front,
            box_back: box_back,
            appearance_frame: appr_frame,
            draw_request: DrawRequest::InitDraw,
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
            let contains = obj.get_object().get_drawing_area(ctx).contains(mintp!(rpoint));
            if contains {
                clicked_data = obj.get_object_mut().click_hold_data(ctx, rpoint);
                self.draw_request = DrawRequest::Draw;
                break;
            }
        }

        clicked_data
    }

    pub fn dragging_handler(&mut self, point: numeric::Point2f, last: numeric::Point2f) {
        if let Some(obj) = &mut self.dragging {
            obj.get_object_mut()
                .move_diff(numeric::Vector2f::new(point.x - last.x, point.y - last.y));
            self.draw_request = DrawRequest::Draw;
        }
    }

    pub fn unselect_dragging_object(&mut self, t: Clock) {
        if let Some(dragged) = &mut self.dragging {
            dragged
                .as_movable_object_mut()
                .override_move_func(move_fn::gravity_move(1.0, 10.0, 310.0, 0.5), t);
            dragged.as_effectable_object().add_effect(vec![Box::new(
                |obj: &mut dyn MovableObject, _: &ggez::Context, t: Clock| {
                    println!("{}", obj.get_position().y);
                    if obj.get_position().y >= 310.0 {
                        obj.override_move_func(None, t);
                        EffectFnStatus::EffectFinish
                    } else {
                        EffectFnStatus::EffectContinue
                    }
                },
            )]);
            let dragged_object = std::mem::replace(&mut self.dragging, None);
            self.shelved.push(dragged_object.unwrap());
            self.draw_request = DrawRequest::Draw;
        }
    }

    ///
    /// # 再描画要求有り
    ///
    pub fn update<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        for p in &mut self.shelved {
            if !p.as_movable_object().is_stop() || !p.as_effectable_object().is_empty_effect() {
                self.draw_request = DrawRequest::Draw;
                ctx.process_utility.redraw();
            }

            p.as_movable_object_mut().move_with_func(t);
            p.as_effectable_object().effect(ctx.context, t);
        }
    }

    pub fn add_object(&mut self, obj: TaskItem) {
        self.shelved.push(obj);
        self.draw_request = DrawRequest::Draw;
    }

    pub fn add_customer_object_vec(&mut self, mut obj_vec: Vec<TaskItem>) {
        while obj_vec.len() != 0 {
            self.add_object(obj_vec.pop().unwrap());
        }
        self.draw_request = DrawRequest::Draw;
    }

    pub fn has_dragging(&self) -> bool {
        self.dragging.is_some()
    }

    pub fn insert_dragging(&mut self, obj: TaskItem) {
        let d = std::mem::replace(&mut self.dragging, Some(obj));
        if d.is_some() {
            self.add_object(d.unwrap());
            self.draw_request = DrawRequest::Draw;
        }
    }

    pub fn release_dragging(&mut self) -> Option<TaskItem> {
        self.draw_request = DrawRequest::Draw;
        std::mem::replace(&mut self.dragging, None)
    }

    pub fn ref_dragging(&self) -> &Option<TaskItem> {
        &self.dragging
    }

    pub fn out_of_desk(&self, point: numeric::Point2f) -> bool {
        !self.canvas.contains(point)
    }

    fn button_up_handler<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        t: Clock,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
        _: &KosuzuMemory,
    ) {
        let rpoint = self.canvas.relative_point(point);

        for dobj in &mut self.shelved {
            if dobj.get_object_mut().contains(ctx.context, rpoint) {
                dobj.get_object_mut().button_up(ctx, t, button, rpoint);
                self.draw_request = DrawRequest::Draw;
            }
        }
    }

    pub fn check_mouse_cursor_status(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> CursorIcon {
        if self.canvas.get_drawing_area(ctx).contains(mintp!(point)) {
            let rpoint = self.canvas.relative_point(point);

            // オブジェクトは深度が深い順にソートされているので、
            // 逆順から検索していくことで、最も手前に表示されているオブジェクトを
            // 取り出すことができる
            for obj in self.shelved.iter_mut().rev() {
                if obj.get_object().get_drawing_area(ctx).contains(mintp!(rpoint)) {
                    return CursorIcon::Grab;
                }
            }
        }

        CursorIcon::Default
    }
}

impl DrawableComponent for ShelvingBookBox {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            if self.draw_request != DrawRequest::Skip {
                self.draw_request = DrawRequest::Skip;

                sub_screen::stack_screen(ctx, &self.canvas);

                self.table_texture.draw(ctx)?;

                self.box_back.draw(ctx)?;

                for obj in &mut self.shelved {
                    obj.get_object_mut().draw(ctx)?;
                }

                if let Some(ref mut d) = self.dragging {
                    d.get_object_mut().draw(ctx)?;
                }

                self.box_front.draw(ctx)?;

                self.appearance_frame.draw(ctx)?;

                sub_screen::pop_screen(ctx);
            }
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

pub struct KosuzuPhrase {
    text_balloon: Option<EffectableWrap<MovableWrap<TextBalloon>>>,
    event_list: DelayEventList<Self>,
    canvas: SubScreen,
}

impl KosuzuPhrase {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, depth: i8) -> Self {
        KosuzuPhrase {
            text_balloon: None,
            event_list: DelayEventList::new(),
            canvas: SubScreen::new(
                ctx.context,
                numeric::Rect::new(800.0, 300.0, 300.0, 500.0),
                depth,
                ggraphics::Color::from_rgba_u32(0),
            ),
        }
    }

    pub fn insert_new_phrase<'a>(&mut self, ctx: &mut SuzuContext<'a>, text: &str, t: Clock) {
        self.text_balloon = Some(EffectableWrap::new(
            MovableWrap::new(
                Box::new(TextBalloon::new(
                    ctx.context,
                    numeric::Rect::new(0.0, 0.0, 300.0, 500.0),
                    text,
                    TextBalloonPhraseType::SimplePhrase,
                    FontInformation::new(
                        ctx.resource.get_font(FontID::JpFude1),
                        numeric::Vector2f::new(21.0, 21.0),
                        ggraphics::Color::BLACK,
                    ),
                )),
                None,
                t,
            ),
            vec![effect::fade_in(10, t)],
        ));

        self.event_list.add_event(
            Box::new(|slf: &mut Self, _, t| slf.close_text_balloon(t)),
            t + 240,
        );
    }

    fn close_text_balloon(&mut self, t: Clock) {
        if let Some(balloon) = self.text_balloon.as_mut() {
            balloon.add_effect(vec![effect::fade_out(10, t)]);
            self.event_list.add_event(
                Box::new(|slf: &mut Self, _, _| slf.text_balloon = None),
                t + 11,
            );
        }
    }

    ///
    /// # 再描画要求有り
    ///
    pub fn update<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        flush_delay_event_and_redraw_check!(self, self.event_list, ctx, t, {});

        if let Some(balloon) = self.text_balloon.as_mut() {
            if !balloon.is_stop() || !balloon.is_empty_effect() {
                ctx.process_utility.redraw();
            }

            balloon.effect(ctx.context, t);
            balloon.move_with_func(t);
        }
    }
}

impl DrawableComponent for KosuzuPhrase {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            if let Some(balloon) = self.text_balloon.as_mut() {
                balloon.draw(ctx)?;
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

struct FloatingMemoryObject {
    header_text: UniText,
    text: Vec<MovableWrap<UniText>>,
    appearance_frame: TileBatchFrame,
    canvas: SubScreen,
    padding: f32,
    font_info: FontInformation,
}

impl FloatingMemoryObject {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        init_rect: numeric::Rect,
        title: String,
        padding: f32,
        appear_frame_id: TileBatchTextureID,
        depth: i8,
    ) -> Self {
        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(28.0, 28.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let mut header_text = UniText::new(
            title,
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            font_info.clone(),
        );
        header_text.make_center(
            ctx.context,
            numeric::Point2f::new(init_rect.w / 2.0, padding + 28.0),
        );

        FloatingMemoryObject {
            header_text: header_text,
            text: Vec::new(),
            appearance_frame: TileBatchFrame::new(
                ctx.resource,
                appear_frame_id,
                numeric::Rect::new(0.0, 0.0, 250.0, 200.0),
                numeric::Vector2f::new(0.6, 0.6),
                0,
            ),
            canvas: SubScreen::new(
                ctx.context,
                init_rect,
                depth,
                ggraphics::Color::from_rgba_u32(0x0),
            ),
            padding: padding,
            font_info: font_info,
        }
    }

    fn next_position_y(&self) -> f32 {
        ((self.padding + self.font_info.scale.y) * (self.text.len() as f32)) + 74.0
    }

    fn center_position_x(&self, ctx: &mut ggez::Context) -> f32 {
        self.canvas.get_drawing_size(ctx).x / 2.0
    }

    fn required_canvas_size(&self, _ctx: &mut ggez::Context) -> numeric::Vector2f {
        let outer_frame_width = 2.0 * self.padding;
        let height = self.next_position_y() + outer_frame_width + 16.0;

        numeric::Vector2f::new(250.0, height)
    }

    fn update_canvas<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        let new_size = self.required_canvas_size(ctx.context);
        let position = self.canvas.get_position();
        let depth = self.canvas.get_drawing_depth();

        println!("required size: {:?}", new_size);

        let new_canvas = SubScreen::new(
            ctx.context,
            numeric::Rect::new(position.x, position.y, new_size.x, new_size.y),
            depth,
            ggraphics::Color::from_rgba_u32(0),
        );

        self.canvas = new_canvas;

        let canvas_size = self.canvas.get_drawing_size(ctx.context);

        let appr_frame = TileBatchFrame::new(
            ctx.resource,
            self.appearance_frame.get_frame_texture_id(),
            numeric::Rect::new(0.0, 0.0, canvas_size.x, canvas_size.y),
            numeric::Vector2f::new(0.6, 0.6),
            0,
        );
        self.appearance_frame = appr_frame;
    }

    fn contains_text(&self, text: &str) -> bool {
        for uni_text in self.text.iter() {
            if uni_text.get_text() == text {
                return true;
            }
        }

        false
    }

    pub fn add_text_line<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        text: String,
        init_text_pos: numeric::Point2f,
        now: Clock,
    ) {
        if self.contains_text(&text) {
            return;
        }

        let mut uni_text = Box::new(UniText::new(
            text,
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            self.font_info.clone(),
        ));

        let center_position =
            numeric::Point2f::new(self.center_position_x(ctx.context), self.next_position_y());
        uni_text.make_center(ctx.context, center_position);

        let goal = uni_text.get_position();
        uni_text.set_position(init_text_pos);

        self.text.push(MovableWrap::new(
            uni_text,
            move_fn::devide_distance(goal, 0.3),
            now,
        ));

        self.update_canvas(ctx);
    }

    pub fn update(&mut self, t: Clock) -> DrawRequest {
        let mut request = DrawRequest::Skip;

        for text in self.text.iter_mut() {
            if !text.is_stop() {
                request = DrawRequest::Draw;
            }
            text.move_with_func(t);
        }

        request
    }
}

impl DrawableComponent for FloatingMemoryObject {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.header_text.draw(ctx)?;

            for uni_text in self.text.iter_mut() {
                uni_text.draw(ctx)?;
            }

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

    fn set_drawing_depth(&mut self, depth: i8) {
        self.canvas.set_drawing_depth(depth);
    }

    fn get_drawing_depth(&self) -> i8 {
        self.canvas.get_drawing_depth()
    }
}

///
/// メニューに表示するやつ
///
struct TaskInfoContents {
    book_info_memory: FloatingMemoryObject,
    general_table_frame: TableFrame,
    header_text: UniText,
    desc_text: Vec<VerticalText>,
    request_info_text: HashMap<String, VerticalText>,
    drwob_essential: DrawableObjectEssential,
}

impl TaskInfoContents {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, customer_request: Option<CustomerRequest>) -> Self {
        let normal_scale_font = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(24.0, 24.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );

        let large_scale_font = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(40.0, 40.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );

        let general_frame = TableFrame::new(
            ctx.resource,
            numeric::Point2f::new(50.0, 95.0),
            TileBatchTextureID::OldStyleFrame,
            FrameData::new(vec![150.0, 170.0], vec![45.0; 4]),
            numeric::Vector2f::new(0.25, 0.25),
            0,
        );

        let mut header_text = UniText::new(
            "Board".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            large_scale_font,
        );
        header_text.make_center(ctx.context, numeric::Point2f::new(150.0, 60.0));

        let mut desc_text = Vec::new();
        let mut request_text = HashMap::new();

        for (index, s) in vec!["本日", "要件", "氏名", "期限"]
            .iter()
            .enumerate()
        {
            let mut vtext = VerticalText::new(
                s.to_string(),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                normal_scale_font,
            );

            set_table_frame_cell_center!(
                ctx.context,
                general_frame,
                vtext,
                numeric::Vector2u::new(index as u32, 0)
            );

            desc_text.push(vtext);
        }

        let mut request_type_vtext = VerticalText::new(
            if let Some(request) = customer_request.as_ref() {
                request.to_string()
            } else {
                "".to_string()
            },
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            normal_scale_font,
        );

        set_table_frame_cell_center!(
            ctx.context,
            general_frame,
            request_type_vtext,
            numeric::Vector2u::new(1, 1)
        );

        request_text.insert("youken".to_string(), request_type_vtext);

	let mut today_vtext = VerticalText::new(
	    ctx.take_save_data().date.to_short_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            normal_scale_font,
        );

        set_table_frame_cell_center!(
            ctx.context,
            general_frame,
            today_vtext,
            numeric::Vector2u::new(0, 1)
        );

        request_text.insert("date".to_string(), today_vtext);

        let book_floating = FloatingMemoryObject::new(
            ctx,
            numeric::Rect::new(25.0, 440.0, 250.0, 250.0),
            "-Books-".to_string(),
            10.0,
            TileBatchTextureID::TaishoStyle1,
            0,
        );
        TaskInfoContents {
            book_info_memory: book_floating,
            general_table_frame: general_frame,
            header_text: header_text,
            desc_text: desc_text,
            request_info_text: request_text,
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }

    pub fn add_book_info<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        book_info: BookInformation,
        init_text_pos: numeric::Point2f,
        now: Clock,
    ) {
        self.book_info_memory
            .add_text_line(ctx, book_info.name, init_text_pos, now);
    }

    pub fn set_customer_name<'a>(&mut self, ctx: &mut SuzuContext<'a>, name: String) {
        let normal_scale_font = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(24.0, 24.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );

        let key = "name";

        println!("set name !! => {}", name);

        if self.request_info_text.contains_key(key) {
            self.request_info_text.remove(key);
        }

        let mut vtext = VerticalText::new(
            name.to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            normal_scale_font,
        );

        set_table_frame_cell_center!(
            ctx.context,
            self.general_table_frame,
            vtext,
            numeric::Vector2u::new(2, 1)
        );

        self.request_info_text.insert(key.to_string(), vtext);
    }

    pub fn set_rental_limit<'a>(&mut self, ctx: &mut SuzuContext<'a>, rental_limit: RentalLimit) {
        let normal_scale_font = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(24.0, 24.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );

        let key = "limit";

        if self.request_info_text.contains_key(key) {
            self.request_info_text.remove(key);
        }

        let mut vtext = VerticalText::new(
	    rental_limit.to_str().to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            normal_scale_font,
        );

        set_table_frame_cell_center!(
            ctx.context,
            self.general_table_frame,
            vtext,
            numeric::Vector2u::new(3, 1)
        );

        self.request_info_text.insert(key.to_string(), vtext);
    }

    pub fn update(&mut self, t: Clock) -> DrawRequest {
        self.book_info_memory.update(t)
    }
}

impl DrawableComponent for TaskInfoContents {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.book_info_memory.draw(ctx)?;
            self.general_table_frame.draw(ctx)?;

            self.header_text.draw(ctx)?;

            for vtext in self.desc_text.iter_mut() {
                vtext.draw(ctx).unwrap();
            }

            for (_, vtext) in self.request_info_text.iter_mut() {
                vtext.draw(ctx).unwrap();
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

pub struct TaskInfoPanel {
    canvas: SubScreen,
    draw_request: DrawRequest,
    contents: TaskInfoContents,
    background: UniTexture,
}

impl TaskInfoPanel {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        size: numeric::Rect,
        customer_request: Option<CustomerRequest>,
    ) -> Self {
        TaskInfoPanel {
            canvas: SubScreen::new(
                ctx.context,
                size,
                0,
                ggraphics::Color::from_rgba_u32(0xffffffff),
            ),
            draw_request: DrawRequest::InitDraw,
            background: UniTexture::new(
                ctx.ref_texture(TextureID::BaraBG),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
            ),
            contents: TaskInfoContents::new(ctx, customer_request),
        }
    }

    pub fn add_book_info<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        book_info: BookInformation,
        init_text_pos: numeric::Point2f,
        now: Clock,
    ) {
        self.contents
            .add_book_info(ctx, book_info, init_text_pos, now);
    }

    pub fn set_customer_name<'a>(&mut self, ctx: &mut SuzuContext<'a>, name: String) {
        self.contents.set_customer_name(ctx, name);
        self.draw_request = DrawRequest::Draw;
    }

    ///
    /// # 再描画要求有り
    ///
    pub fn update<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        let go_draw = self.contents.update(t);
        if self.draw_request != DrawRequest::InitDraw {
            self.draw_request |= go_draw;
        }

        match self.draw_request {
            DrawRequest::Draw | DrawRequest::InitDraw => {
                ctx.process_utility.redraw();
            }
            _ => (),
        }
    }

    pub fn set_rental_limit<'a>(&mut self, ctx: &mut SuzuContext<'a>, rental_limit: RentalLimit) {
	self.contents.set_rental_limit(ctx, rental_limit);
        self.draw_request = DrawRequest::Draw;
    } 
}

impl DrawableComponent for TaskInfoPanel {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.draw_request != DrawRequest::Skip {
            self.draw_request = DrawRequest::Skip;
            sub_screen::stack_screen(ctx, &self.canvas);

            self.background.draw(ctx)?;
            self.contents.draw(ctx)?;

            sub_screen::pop_screen(ctx);
        }
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

pub struct MoneyBox {
    draw_request: DrawRequest,
    canvas: SubScreen,
    coin_set: DeskObjectContainer,
    box_texture: UniTexture,
}

impl MoneyBox {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, pos_rect: numeric::Rect, depth: i8) -> Self {
        MoneyBox {
            draw_request: DrawRequest::InitDraw,
            canvas: SubScreen::new(
                ctx.context,
                pos_rect,
                depth,
                ggraphics::Color::from_rgba_u32(0),
            ),
            box_texture: UniTexture::new(
                ctx.ref_texture(TextureID::MoneyBox),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                1,
            ),
            coin_set: DeskObjectContainer::new(),
        }
    }

    pub fn is_acceptable_for_moneybox(item: &TaskItem) -> bool {
        match item {
            TaskItem::Coin(_) => true,
            _ => false,
        }
    }

    pub fn add_coin(&mut self, mut coin_item: TaskItem) {
        match coin_item {
            TaskItem::Coin(ref mut texture) => {
                texture.get_small_object_mut().disable_shadow();
                texture.get_large_object_mut().disable_shadow();

                self.draw_request = DrawRequest::Draw;
                self.coin_set.add_item(coin_item);
            }
            _ => (),
        }
    }

    pub fn relative_point(&self, point: numeric::Point2f) -> numeric::Point2f {
        self.canvas.relative_point(point)
    }
}

impl DrawableComponent for MoneyBox {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            if self.draw_request != DrawRequest::Skip {
                self.draw_request = DrawRequest::Skip;
                sub_screen::stack_screen(ctx, &self.canvas);

                self.box_texture.draw(ctx)?;

                for obj in self.coin_set.get_raw_container_mut() {
                    obj.get_object_mut().draw(ctx)?;
                }

                sub_screen::pop_screen(ctx);
            }
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

impl DrawableObject for MoneyBox {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for MoneyBox {
    impl_texture_object_for_wrapped! {canvas}
}

#[derive(Clone, Copy, PartialEq)]
enum LineJustified {
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq)]
enum ChatBoxPerson {
    Me,
    Partner,
    SystemName(LineJustified),
}

pub struct ChatBox {
    canvas: SubScreen,
    messages: Vec<(ChatBoxPerson, UniText)>,
    background: UniTexture,
    partner_name: Option<String>,
    my_name: Option<String>,
    draw_request: DrawRequest,
}

impl ChatBox {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        rect: numeric::Rect,
        depth: i8,
        partner_name: Option<String>,
        my_name: Option<String>,
    ) -> Self {
        ChatBox {
            canvas: SubScreen::new(ctx.context, rect, depth, ggraphics::Color::from_rgba_u32(0)),
            messages: Vec::new(),
            background: UniTexture::new(
                ctx.ref_texture(TextureID::TextBackground),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
            ),
            partner_name: partner_name,
            my_name: my_name,
            draw_request: DrawRequest::InitDraw,
        }
    }

    pub fn set_partner_name(&mut self, name: String) {
        self.partner_name = Some(name);
        self.draw_request = DrawRequest::Draw;
    }

    pub fn set_my_name(&mut self, name: String) {
        self.my_name = Some(name);
        self.draw_request = DrawRequest::Draw;
    }

    fn put_message_in_default_place<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        //balloonをいい感じに
        let canvas_size = self.canvas.get_drawing_size(ctx.context);
        let mut left_buttom_y = canvas_size.y;
        for (person, text) in self.messages.iter_mut().rev() {
            let size = text.get_drawing_size(ctx.context);
            let x_pos = match person {
                ChatBoxPerson::Me => canvas_size.x - size.x - 10.0,
                ChatBoxPerson::Partner => 10.0,
                ChatBoxPerson::SystemName(j) => {
                    left_buttom_y -= 5.0;
                    match j {
                        LineJustified::Left => 10.0,
                        LineJustified::Right => canvas_size.x - size.x - 10.0,
                    }
                }
            };
            left_buttom_y -= size.y + 10.0;

            text.set_position(numeric::Point2f::new(x_pos, left_buttom_y));
        }
        self.draw_request = DrawRequest::Draw;
    }

    fn last_person(&self) -> Option<ChatBoxPerson> {
        if let Some((person, _)) = self.messages.last() {
            Some(person.clone())
        } else {
            None
        }
    }

    fn add_name_message<'a>(&mut self, ctx: &mut SuzuContext<'a>, person: ChatBoxPerson) {
        let name = match person {
            ChatBoxPerson::Me => {
                if let Some(name) = self.my_name.as_ref() {
                    name.clone()
                } else {
                    "？".to_string()
                }
            }
            ChatBoxPerson::Partner => {
                if let Some(name) = self.partner_name.as_ref() {
                    name.clone()
                } else {
                    "？".to_string()
                }
            }
            ChatBoxPerson::SystemName(_) => "？".to_string(),
        };

        let system_text = UniText::new(
            name,
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                ctx.resource.get_font(FontID::BitMap1),
                numeric::Vector2f::new(12.0, 12.0),
                ggraphics::Color::BLACK,
            ),
        );

        let justified = match person {
            ChatBoxPerson::Me => LineJustified::Right,
            ChatBoxPerson::Partner | ChatBoxPerson::SystemName(_) => LineJustified::Left,
        };
        self.messages
            .push((ChatBoxPerson::SystemName(justified), system_text));
        self.draw_request = DrawRequest::Draw;
    }

    fn add_message<'a>(&mut self, ctx: &mut SuzuContext<'a>, msg: String, person: ChatBoxPerson) {
        let text = UniText::new(
            msg,
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                ctx.resource.get_font(FontID::BitMap1),
                numeric::Vector2f::new(12.0, 12.0),
                ggraphics::Color::BLACK,
            ),
        );

        if let Some(last_person) = self.last_person() {
            if last_person != person {
                self.add_name_message(ctx, person);
            }
        } else {
            self.add_name_message(ctx, person);
        }

        self.messages.push((person, text));
        self.put_message_in_default_place(ctx);
        self.draw_request = DrawRequest::Draw;
    }

    pub fn add_message_as_partner<'a>(&mut self, ctx: &mut SuzuContext<'a>, msg: String) {
        self.draw_request = DrawRequest::Draw;
        self.add_message(ctx, msg, ChatBoxPerson::Partner)
    }

    pub fn add_message_as_mine<'a>(&mut self, ctx: &mut SuzuContext<'a>, msg: String) {
        self.draw_request = DrawRequest::Draw;
        self.add_message(ctx, msg, ChatBoxPerson::Me)
    }
}

impl DrawableComponent for ChatBox {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            if self.draw_request != DrawRequest::Skip {
                self.draw_request = DrawRequest::Skip;
                sub_screen::stack_screen(ctx, &self.canvas);

                self.background.draw(ctx)?;

                for (_, text) in self.messages.iter_mut() {
                    text.draw(ctx)?;
                }

                sub_screen::pop_screen(ctx);
            }
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

impl DrawableObject for ChatBox {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for ChatBox {
    impl_texture_object_for_wrapped! {canvas}
}

impl Scrollable for ChatBox {
    fn scroll<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        _point: numeric::Point2f,
        offset: numeric::Vector2f,
    ) {
        if self.messages.is_empty() {
            return;
        }

        let mut scroll_size = 0.0;
        let offset = numeric::Vector2f::new(offset.x, -offset.y);

        // フォーカスを下へ -> 全体をマイナス
        if offset.y > 0.0 {
            let last_text = &self.messages.last().unwrap().1;
            if last_text.get_drawing_area(ctx.context).bottom() - offset.y + 10.0
                >= self.canvas.get_drawing_size(ctx.context).y
            {
                scroll_size = -offset.y;
            } else if last_text.get_drawing_area(ctx.context).bottom() + 10.0
                >= self.canvas.get_drawing_size(ctx.context).y
            {
                scroll_size = -(last_text.get_drawing_area(ctx.context).bottom()
                    - self.canvas.get_drawing_size(ctx.context).y);
            }
        } else {
            // フォーカスを上へ -> 全体をプラス
            let first_text = &self.messages.first().unwrap().1;
            if first_text.get_drawing_area(ctx.context).top() - offset.y <= 0.0 {
                scroll_size = -offset.y;
            } else if first_text.get_drawing_area(ctx.context).top() <= 0.0 {
                scroll_size = first_text.get_drawing_area(ctx.context).top();
            }
        }

        for (_, text) in self.messages.iter_mut() {
            text.move_diff(numeric::Vector2f::new(0.0, scroll_size * 4.0));
        }

        self.draw_request = DrawRequest::Draw;
        ctx.process_utility.redraw();
    }
}
