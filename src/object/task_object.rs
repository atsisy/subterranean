pub mod factory;
pub mod task_table_elements;

use std::rc::Rc;

use ggez::graphics as ggraphics;
use ggez::input as ginput;
use ginput::mouse::MouseButton;
use ginput::mouse::MouseCursor;

use torifune::graphics::object::*;
use torifune::graphics::*;
use torifune::numeric;
use torifune::impl_texture_object_for_wrapped;
use torifune::impl_drawable_object_for_wrapped;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::graphics::object::sub_screen;
use torifune::core::Clock;

use crate::object::{move_fn, effect};
use task_table_elements::*;

use super::Clickable;
use crate::core::{TextureID, GameData};

pub struct TaskTable {
    canvas: SubScreen,
    sight: SuzuMiniSight,
    desk: DeskObjects,
    goods: Goods,
    book_wagon: BookWagon,
    hold_data: HoldData,
}

impl TaskTable {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData,
               pos: numeric::Rect,
               sight_rect: numeric::Rect,
	       goods_rect: numeric::Rect,
	       desk_rect: numeric::Rect,
	       book_wagon_rect: numeric::Rect,
	       t: Clock) -> Self {
	let sight = SuzuMiniSight::new(ctx, game_data, sight_rect);
        let mut desk = DeskObjects::new(ctx, game_data, desk_rect);
        
        desk.add_object(DeskObject::new(
            Box::new(OnDeskTexture::new(
                UniTexture::new(
                    game_data.ref_texture(TextureID::LotusPink),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(0.1, 0.1),
                    0.0, -1))),
            Box::new(OnDeskTexture::new(
                UniTexture::new(
                    game_data.ref_texture(TextureID::LotusPink),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(0.1, 0.1),
                    0.0, -1))), 1, DeskObjectType::SuzunaObject, t));
	
        let mut record_book = Box::new(BorrowingRecordBook::new(ggraphics::Rect::new(0.0, 0.0, 400.0, 290.0)));
        record_book.add_empty_page(ctx,
				   game_data, 0);
	let mut record_book = DeskObject::new(
            Box::new(
		OnDeskTexture::new(
                    UniTexture::new(
			game_data.ref_texture(TextureID::Chobo1),
			numeric::Point2f::new(0.0, 0.0),
			numeric::Vector2f::new(0.2, 0.2),
			0.0, -1))),
            record_book, 0,
	    DeskObjectType::BorrowRecordBook, t);
	record_book.enable_large();
        desk.add_object(record_book);

	let book_wagon = BookWagon::new(ctx, game_data, book_wagon_rect);
        
        TaskTable {
            canvas: SubScreen::new(ctx, pos, 0, ggraphics::Color::from_rgba_u32(0x00000000)),
            sight: sight,
            desk: desk,
	    goods: Goods::new(ctx, game_data, goods_rect),
	    book_wagon: book_wagon,
	    hold_data: HoldData::None,
        }
    }
    
    fn select_dragging_object(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        let rpoint = self.canvas.relative_point(point);
        self.desk.select_dragging_object(ctx, rpoint);
    }

    pub fn double_click_handler(&mut self,
                                ctx: &mut ggez::Context,
                                point: numeric::Point2f,
                                game_data: &GameData) {
        let rpoint = self.canvas.relative_point(point);
        self.desk.double_click_handler(ctx, rpoint, game_data);
    }

    pub fn dragging_handler(&mut self,
                            point: numeric::Point2f,
                            last: numeric::Point2f) {
        let rpoint = self.canvas.relative_point(point);
        let rlast = self.canvas.relative_point(last);
        
        self.sight.dragging_handler(rpoint, rlast);
        self.desk.dragging_handler(rpoint, rlast);
    }

    pub fn unselect_dragging_object(&mut self, ctx: &mut ggez::Context, t: Clock) {
	self.sight.unselect_dragging_object(ctx, t);
        self.desk.unselect_dragging_object();
    }

    pub fn hand_over_check(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        let rpoint = self.canvas.relative_point(point);
        self.hand_over_check_d2s(ctx, rpoint);
        self.hand_over_check_s2d(ctx, rpoint);
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

    fn hand_over_check_d2s(&mut self, ctx: &mut ggez::Context, rpoint: numeric::Point2f) {
        let border = self.desk_border(ctx);
        
        if self.desk.has_dragging() && border > rpoint.y {
            if let Some(mut dragging) = self.desk.release_dragging() {
		self.apply_d2s_point_convertion(ctx, &mut dragging);
                self.sight.insert_dragging(dragging);
            }
        }
    }

    fn hand_over_check_s2d(&mut self, ctx: &mut ggez::Context, rpoint: numeric::Point2f) {
        let border = self.desk_border(ctx);
	
        if self.sight.has_dragging() && border < rpoint.y {
            if let Some(mut dragging) = self.sight.release_dragging() {
		self.apply_s2d_point_convertion(ctx, &mut dragging);
                self.desk.insert_dragging(dragging);
            }
        }
    }

    fn desk_border(&mut self, ctx: &mut ggez::Context) -> f32 {
        let sight_edge = self.sight.canvas.get_position().y + self.sight.canvas.get_texture_size(ctx).y;
        let diff = (sight_edge - self.desk.canvas.get_position().y).abs();
        sight_edge + diff
    }

    fn desk_edge_to_sight_edge(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) -> numeric::Point2f {
        numeric::Point2f::new(point.x,
                              self.sight.canvas.get_texture_size(ctx).y)
    }

    fn sight_edge_to_desk_edge(&mut self, rpoint: numeric::Point2f) -> numeric::Point2f {
        numeric::Point2f::new(rpoint.x,
                              0.0)
    }

    fn check_sight_drop_to_desk(&mut self, ctx: &mut ggez::Context, t: Clock) {
	let converted = self.sight.check_drop_desk();
	if converted.len() == 0 {
	    return ()
	}
	
	let min = self.desk.desk_objects.get_minimum_depth();
	let converted = converted.into_iter()
	    .map(|mut obj| {
		self.apply_s2d_point_convertion(ctx, &mut obj);
		obj.get_object_mut().clear_effect();
		obj.get_object_mut().override_move_func(move_fn::gravity_move(1.0, 10.0, 400.0, 0.3), t);
		obj.get_object_mut().set_drawing_depth(min);
		obj.get_object_mut().add_effect(vec![
		    Box::new(|obj: &mut dyn MovableObject, _: &ggez::Context, t: Clock| {
			if obj.get_position().y > 150.0 { obj.override_move_func(None, t); EffectFnStatus::EffectFinish }
			else { EffectFnStatus::EffectContinue }
		    })
		]);
		obj
	    })
	    .collect::<Vec<_>>();
	self.desk.desk_objects.change_depth_equally(1);
	self.desk.add_customer_object_vec(converted);
    }

    pub fn update(&mut self, ctx: &mut ggez::Context, _: &GameData, t: Clock) {
	self.sight.update(ctx, t);
	self.desk.update(ctx, t);
	self.check_sight_drop_to_desk(ctx, t);
    }

    pub fn get_remaining_customer_object_number(&self) -> usize {
	self.desk.count_object_by_type(DeskObjectType::CustomerObject)
    }

    fn start_borrowing_customer_event(&mut self,
				      ctx: &mut ggez::Context,
				      game_data: &GameData,
				      info: BorrowingInformation, t: Clock) {
        for _ in &info.borrowing {
	    let mut obj = factory::create_dobj_book_random(ctx, game_data,
							   DeskObjectType::CustomerObject, t);
	    obj.enable_large();
            self.desk.add_customer_object(obj);
        }
	
	let mut new_silhouette = SimpleObject::new(
	    MovableUniTexture::new(
		game_data.ref_texture(TextureID::JunkoTachieDefault),
		numeric::Point2f::new(100.0, 20.0),
		numeric::Vector2f::new(0.1, 0.1),
		0.0, 0, None, t),
	    vec![effect::appear_bale_down_from_top(50, t), effect::fade_in(50, t)]);
	new_silhouette.set_alpha(0.0);
	self.sight.replace_character_silhouette(new_silhouette, info.borrower.to_string());
    }

    fn start_returning_customer_event(&mut self,
				      ctx: &mut ggez::Context,
				      game_data: &GameData,
				      info: ReturnBookInformation, t: Clock) {
        for _ in &info.returning {
	    let mut obj = factory::create_dobj_book_random(ctx, game_data,
							   DeskObjectType::CustomerObject, t);
	    obj.enable_large();
            self.desk.add_customer_object(obj);
        }
	
	let mut new_silhouette = SimpleObject::new(
	    MovableUniTexture::new(
		game_data.ref_texture(TextureID::JunkoTachieDefault),
		numeric::Point2f::new(100.0, 20.0),
		numeric::Vector2f::new(0.1, 0.1),
		0.0, 0, None, t),
	    vec![effect::appear_bale_down_from_top(50, t), effect::fade_in(50, t)]);
	new_silhouette.set_alpha(0.0);
	self.sight.replace_character_silhouette(new_silhouette, info.borrower.to_string());
    }

    fn start_copying_request_event(&mut self,
				       ctx: &mut ggez::Context,
				       game_data: &GameData,
				       info: CopyingRequestInformation, t: Clock) {
	let paper_info = CopyingRequestInformation::new_random(game_data,
							       GensoDate::new(128, 12, 8),
                                                               GensoDate::new(128, 12, 8));
	let paper_obj = DeskObject::new(
            Box::new(OnDeskTexture::new(
		UniTexture::new(
                    game_data.ref_texture(TextureID::Paper1),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(0.1, 0.1),
                    0.0, 0))),
            Box::new(CopyingRequestPaper::new(ctx, ggraphics::Rect::new(0.0, 0.0, 420.0, 350.0), TextureID::Paper1,
                                              paper_info,
                                              game_data, t)), 1, DeskObjectType::CustomerObject, t);
	
	let mut new_silhouette = SimpleObject::new(
	    MovableUniTexture::new(
		game_data.ref_texture(TextureID::JunkoTachieDefault),
		numeric::Point2f::new(100.0, 20.0),
		numeric::Vector2f::new(0.1, 0.1),
		0.0, 0, None, t),
	    vec![effect::fade_in(50, t)]);
	self.desk.add_customer_object(paper_obj);
	
	new_silhouette.set_alpha(0.0);
	self.sight.replace_character_silhouette(new_silhouette, info.customer.to_string());
    }

    pub fn start_customer_event(&mut self,
				ctx: &mut ggez::Context,
				game_data: &GameData,
				info: CustomerRequest, t: Clock) {
	match info {
	    CustomerRequest::Borrowing(info) => self.start_borrowing_customer_event(ctx, game_data, info, t),
	    CustomerRequest::Returning(info) => self.start_returning_customer_event(ctx, game_data, info, t),
	    CustomerRequest::Copying(info) => self.start_copying_request_event(ctx, game_data, info, t),
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

    fn update_hold_data(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
	if self.hold_data.is_none() {
	    let clicked_data = self.desk.check_data_click(ctx, point);
	    self.update_hold_data_if_some(clicked_data);

	    let clicked_data = self.sight.check_data_click(ctx, point);
	    self.update_hold_data_if_some(clicked_data);
	    
	    let clicked_data = self.goods.check_data_click(ctx, point);
	    self.update_hold_data_if_some(clicked_data);
	} else {
	    if self.desk.check_insert_data(ctx, point, &self.hold_data) {
		self.hold_data = HoldData::None;
	    }
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
	    self.book_wagon.draw(ctx).unwrap();

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
    impl_drawable_object_for_wrapped!{canvas}
}

impl TextureObject for TaskTable {
    impl_texture_object_for_wrapped!{canvas}
}

impl Clickable for TaskTable {
    fn button_down(&mut self,
                   ctx: &mut ggez::Context,
		   _: &GameData,
		   _: Clock,
                   _button: ggez::input::mouse::MouseButton,
                   point: numeric::Point2f) {
	self.select_dragging_object(ctx, point);
    }
    
    fn button_up(&mut self,
                 ctx: &mut ggez::Context,
		 game_data: &GameData,
		 t: Clock,
                 button: ggez::input::mouse::MouseButton,
                 point: numeric::Point2f) {
	let rpoint = self.canvas.relative_point(point);
	self.desk.button_up_handler(ctx, game_data, t, button, rpoint);
    }
    
    fn on_click(&mut self,
                ctx: &mut ggez::Context,
		_: &GameData,
		_: Clock,
                button: ggez::input::mouse::MouseButton,
                point: numeric::Point2f) {
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

    fn clickable_status(&mut self,
			ctx: &mut ggez::Context,
			point: numeric::Point2f) -> ggez::input::mouse::MouseCursor {
	let rpoint = self.canvas.relative_point(point);
	
	let mut cursor_status = self.desk.check_mouse_cursor_status(ctx, rpoint);
	
	if cursor_status != MouseCursor::Default { return cursor_status; }
	
	cursor_status = self.sight.check_mouse_cursor_status(ctx, rpoint);
	if cursor_status != MouseCursor::Default { return cursor_status; }

	cursor_status = self.goods.check_mouse_cursor_status(ctx, rpoint);
	if cursor_status != MouseCursor::Default { return cursor_status; }

	cursor_status
    }
}
