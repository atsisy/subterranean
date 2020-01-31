pub mod factory;

use std::collections::HashMap;

use ggez::graphics as ggraphics;
use ggez::input as ginput;
use ginput::mouse::MouseButton;
use ginput::mouse::MouseCursor;

use torifune::graphics::object::*;
use torifune::graphics::*;
use torifune::numeric;
use torifune::impl_texture_object_for_wrapped;
use torifune::impl_drawable_object_for_wrapped;

use crate::core::BookInformation;

use torifune::hash;

use super::*;
use crate::object::effect;

use super::Clickable;
use crate::core::{TextureID, FontID, GameData};

use number_to_jk::number_to_jk;

#[derive(Clone, Copy)]
pub struct GensoDate {
    pub season: u32,
    pub month: u8,
    pub day: u8,
}

impl GensoDate {
    pub fn new(season: u32, month: u8, day: u8) -> Self {
        GensoDate {
            season: season,
            month: month,
            day: day,
        }
    }

    pub fn new_empty() -> Self {
        GensoDate {
            season: 0,
            month: 0,
            day: 0,
        }
    }
    
    pub fn to_string(&self) -> String {
        format!("第{}季 {}月 {}日",
		number_to_jk(self.season as u64),
		number_to_jk(self.month as u64),
		number_to_jk(self.day as u64))
    }
}

#[derive(Clone)]
pub struct BorrowingInformation {
    pub borrowing: Vec<BookInformation>,
    pub borrower: String,
    pub borrow_date: GensoDate,
    pub return_date: GensoDate,
}

impl BorrowingInformation {
    pub fn new(borrowing: Vec<BookInformation>,
               borrower: &str,
               borrow_date: GensoDate,
               return_date: GensoDate) -> Self {
        BorrowingInformation {
            borrowing: borrowing,
            borrower: borrower.to_string(),
            borrow_date,
            return_date,
        }
    }

    pub fn new_random(game_data: &GameData,
                      borrow_date: GensoDate,
                      return_date: GensoDate) -> Self {
        let borrowing_num = rand::random::<u32>() % 5;
        let mut borrow_books = Vec::new();

        for _ in 0..borrowing_num {
            borrow_books.push(game_data.book_random_select().clone());
        }

        Self::new(borrow_books, game_data.customer_random_select(), borrow_date, return_date)
    }
}

pub enum HoldData {
    BookName(String),
    CustomerName(String),
    Date(GensoDate),
    None,
}

pub trait OnDesk : TextureObject + Clickable {
    fn ondesk_whose(&self) -> i32;

    fn click_data(&self, ctx: &mut ggez::Context, point: numeric::Point2f) -> HoldData;

    fn insert_data(&mut self, _: &mut ggez::Context, _: numeric::Point2f, _: &HoldData) -> bool {
	false
    }
}

pub struct DrawableCalendar {
    date_data: GensoDate,
    paper: UniTexture,
    season_text: VerticalText,
    month_text: VerticalText,
    day_text: VerticalText,
    canvas: SubScreen,
}

impl DrawableCalendar {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData,
	       rect: numeric::Rect, date: GensoDate, paper_tid: TextureID) -> Self {
	let font_info = FontInformation::new(
	    game_data.get_font(FontID::JpFude1),
	    numeric::Vector2f::new(14.0, 14.0),
	    ggraphics::Color::from_rgba_u32(0x000000ff));
	let default_scale = numeric::Vector2f::new(1.0, 1.0);
	
	DrawableCalendar {
	    paper: UniTexture::new(game_data.ref_texture(paper_tid), numeric::Point2f::new(0.0, 0.0), default_scale, 0.0, 0),
	    season_text: VerticalText::new(format!("{}季", number_to_jk(date.season as u64)),
					   numeric::Point2f::new(50.0, 4.0), default_scale,
					   0.0, 0, font_info),
	    month_text: VerticalText::new(format!("{}月", number_to_jk(date.month as u64)),
					  numeric::Point2f::new(32.0, 4.0), default_scale,
					  0.0, 0, font_info),
	    day_text: VerticalText::new(format!("{}日", number_to_jk(date.day as u64)),
					numeric::Point2f::new(16.0, 4.0), default_scale,
					0.0, 0, font_info),
	    date_data: date,
	    canvas: SubScreen::new(ctx, rect, 0,ggraphics::Color::from_rgba_u32(0x00000000)),
	}
    }
}

impl DrawableComponent for DrawableCalendar {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.canvas.begin_drawing(ctx);
	    
	    self.paper.draw(ctx)?;
	    self.season_text.draw(ctx)?;
	    self.month_text.draw(ctx)?;
	    self.day_text.draw(ctx)?;

            self.canvas.end_drawing(ctx);
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

impl DrawableObject for DrawableCalendar {
    #[inline(always)]
    fn set_position(&mut self, pos: numeric::Point2f) {
	self.canvas.set_position(pos);
    }

    #[inline(always)]
    fn get_position(&self) -> numeric::Point2f {
	self.canvas.get_position()
    }

    #[inline(always)]
    fn move_diff(&mut self, offset: numeric::Vector2f) {
	self.canvas.move_diff(offset);
    }
}

impl TextureObject for DrawableCalendar {
    impl_texture_object_for_wrapped!{canvas}
}

impl Clickable for DrawableCalendar {
    fn clickable_status(&mut self,
			ctx: &mut ggez::Context,
			point: numeric::Point2f) -> MouseCursor {
	if self.canvas.get_drawing_area(ctx).contains(point) {
	    MouseCursor::Grab
	} else {
	    MouseCursor::Default
	}
    }
}

impl OnDesk for DrawableCalendar {
    fn ondesk_whose(&self) -> i32 {
	0
    }
    
    fn click_data(&self, _: &mut ggez::Context, _: numeric::Point2f) -> HoldData {
	HoldData::Date(self.date_data)
    }
}

struct OnDeskTexture {
    texture: UniTexture,
}

impl OnDeskTexture {
    pub fn new(obj: UniTexture) -> Self {
	OnDeskTexture {
	    texture: obj,
	}
    }
}

impl DrawableComponent for OnDeskTexture {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
	self.texture.draw(ctx)
    }

    #[inline(always)]
    fn hide(&mut self) {
	self.texture.hide();
    }

    #[inline(always)]
    fn appear(&mut self) {
	self.texture.appear();
    }

    #[inline(always)]
    fn is_visible(&self) -> bool {
	self.texture.is_visible()
    }

    #[inline(always)]
    fn set_drawing_depth(&mut self, depth: i8) {
	self.texture.set_drawing_depth(depth);
    }

    #[inline(always)]
    fn get_drawing_depth(&self) -> i8 {	
	self.texture.get_drawing_depth()
    }
}

impl DrawableObject for OnDeskTexture {
    #[inline(always)]
    fn set_position(&mut self, pos: numeric::Point2f) {
	self.texture.set_position(pos);
    }

    #[inline(always)]
    fn get_position(&self) -> numeric::Point2f {
	self.texture.get_position()
    }

    #[inline(always)]
    fn move_diff(&mut self, offset: numeric::Vector2f) {
	self.texture.move_diff(offset);
    }
}

impl TextureObject for OnDeskTexture {
    impl_texture_object_for_wrapped!{texture}
}

pub struct OnDeskBook {
    info: BookInformation,
    book_texture: UniTexture,
    title: VerticalText,
    canvas: SubScreen,
}

impl OnDeskBook {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData, texture_id: TextureID, info: BookInformation) -> Self {
	let texture = game_data.ref_texture(texture_id);
	let book_texture = UniTexture::new(
	    texture,
	    numeric::Point2f::new(0.0, 0.0),
	    numeric::Vector2f::new(0.2, 0.2),
	    0.0, 0);
	let book_area = book_texture.get_drawing_area(ctx);
	let book_title = info.get_name().to_string();
	OnDeskBook {
	    info: info,
	    book_texture: book_texture,
	    title: VerticalText::new(book_title,
				     numeric::Point2f::new(40.0, 30.0),
				     numeric::Vector2f::new(1.0, 1.0),
				     0.0, 0,
				     FontInformation::new(game_data.get_font(FontID::JpFude1),
							  numeric::Vector2f::new(18.0, 18.0),
							  ggraphics::Color::from_rgba_u32(0x000000ff))),
	    canvas: SubScreen::new(ctx, book_area, 0, ggraphics::Color::from_rgba_u32(0x00000000)),
	}
    }

    pub fn get_book_info(&self) -> &BookInformation {
	&self.info
    }
}

impl DrawableComponent for OnDeskBook {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.canvas.begin_drawing(ctx);
	    
	    self.book_texture.draw(ctx)?;
	    self.title.draw(ctx)?;

            self.canvas.end_drawing(ctx);
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

impl DrawableObject for OnDeskBook {

    fn set_position(&mut self, pos: numeric::Point2f) {
        self.canvas.set_position(pos);
    }

    fn get_position(&self) -> numeric::Point2f {
        self.canvas.get_position()
    }

    fn move_diff(&mut self, offset: numeric::Vector2f) {
        self.canvas.move_diff(offset);
    }
    
}

impl TextureObject for OnDeskBook {
    impl_texture_object_for_wrapped!{canvas}
}

impl Clickable for OnDeskBook {
}

impl OnDesk for OnDeskBook {
    fn ondesk_whose(&self) -> i32 {
	0
    }

    fn click_data(&self, _: &mut ggez::Context, _: numeric::Point2f) -> HoldData {
	return HoldData::BookName(self.info.get_name().to_string())
    }
}

impl Clickable for OnDeskTexture {
}

impl OnDesk for OnDeskTexture {
    fn ondesk_whose(&self) -> i32 {
	0
    }

    fn click_data(&self, _: &mut ggez::Context, _: numeric::Point2f) -> HoldData {
	HoldData::None
    }
}

pub struct BorrowingPaper {
    title: SimpleText,
    borrowing: Vec<SimpleText>,
    book_head: SimpleText,
    borrower: SimpleText,
    borrow_date: SimpleText,
    return_date: SimpleText,
    paper_texture: SimpleObject,
    canvas: SubScreen,
}

impl BorrowingPaper {
    pub fn new(ctx: &mut ggez::Context, rect: ggraphics::Rect, paper_tid: TextureID,
               info: &BorrowingInformation, game_data: &GameData, t: Clock) -> Self {
        let mut pos = numeric::Point2f::new(210.0, 370.0);
        let borrowing = info.borrowing.iter()
            .map(|book_info| {
                pos += numeric::Vector2f::new(0.0, 30.0);
                SimpleText::new(MovableText::new(book_info.name.to_string(),
                                                 pos,
                                                 numeric::Vector2f::new(1.0, 1.0),
                                                 0.0,
                                                 0,
                                                 move_fn::halt(pos),
                                                 FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                                      numeric::Vector2f::new(24.0, 24.0),
                                                                      ggraphics::Color::from_rgba_u32(0x000000ff)),
                                                 t),
                                Vec::new()) }).collect();
        
        let paper_texture = SimpleObject::new(MovableUniTexture::new(game_data.ref_texture(paper_tid),
                                                                     numeric::Point2f::new(0.0, 0.0),
                                                                     numeric::Vector2f::new(1.0, 1.0),
                                                                     0.0,
                                                                     0,
                                                                     move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                                                                     t),
                                              Vec::new());
	
        let book_head = SimpleText::new(MovableText::new("貸出本".to_string(),
                                                          numeric::Point2f::new(50.0, 400.0),
                                                          numeric::Vector2f::new(1.0, 1.0),
                                                          0.0,
                                                          0,
                                                          move_fn::halt(numeric::Point2f::new(50.0, 350.0)),
                                                          FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                                               numeric::Vector2f::new(28.0, 28.0),
                                                                               ggraphics::Color::from_rgba_u32(0x000000ff)),
                                                          t),
                                         Vec::new());
        
        let title_text = SimpleText::new(MovableText::new("鈴奈庵 貸出票".to_string(),
                                                          numeric::Point2f::new(270.0, 100.0),
                                                          numeric::Vector2f::new(1.0, 1.0),
                                                          0.0,
                                                          0,
                                                          move_fn::halt(numeric::Point2f::new(250.0, 100.0)),
                                                          FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                                               numeric::Vector2f::new(28.0, 28.0),
                                                                               ggraphics::Color::from_rgba_u32(0x000000ff)),
                                                          t),
                                         Vec::new());

        let borrower = SimpleText::new(MovableText::new(format!("借りた人   {}", info.borrower),
                                                        numeric::Point2f::new(50.0, 200.0),
                                                        numeric::Vector2f::new(1.0, 1.0),
                                                        0.0,
                                                        0,
                                                        move_fn::halt(numeric::Point2f::new(250.0, 100.0)),
                                                        FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                                             numeric::Vector2f::new(28.0, 28.0),
                                                                             ggraphics::Color::from_rgba_u32(0x000000ff)),
                                                        t),
                                       Vec::new());

        let borrow_date = SimpleText::new(MovableText::new(format!("貸出日     {}", info.borrow_date.to_string()),
                                                        numeric::Point2f::new(50.0, 250.0),
                                                        numeric::Vector2f::new(1.0, 1.0),
                                                        0.0,
                                                        0,
                                                        move_fn::halt(numeric::Point2f::new(250.0, 100.0)),
                                                        FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                                             numeric::Vector2f::new(28.0, 28.0),
                                                                             ggraphics::Color::from_rgba_u32(0x000000ff)),
                                                        t),
                                          Vec::new());

        let return_date = SimpleText::new(MovableText::new(format!("返却期限   {}", info.return_date.to_string()),
                                                           numeric::Point2f::new(50.0, 300.0),
                                                           numeric::Vector2f::new(1.0, 1.0),
                                                           0.0,
                                                           0,
                                                           move_fn::halt(numeric::Point2f::new(50.0, 300.0)),
                                                           FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                                                numeric::Vector2f::new(28.0, 28.0),
                                                                                ggraphics::Color::from_rgba_u32(0x000000ff)),
                                                           t),
                                          Vec::new());
        BorrowingPaper {
            title: title_text,
            borrower: borrower,
            book_head: book_head,
            paper_texture: paper_texture,
            borrowing: borrowing,
            borrow_date: borrow_date,
            return_date: return_date,
            canvas: SubScreen::new(ctx, rect, 0, ggraphics::BLACK),
        }
    }

    pub fn new_random(ctx: &mut ggez::Context, rect: ggraphics::Rect, paper_tid: TextureID,
                      borrow_date: GensoDate, return_date: GensoDate,
                      game_data: &GameData, _t: Clock) -> Self {

        let mut borrowing = Vec::new();

        for _ in 0..(rand::random::<u32>() % 7) {
            borrowing.push(game_data.book_random_select().clone());
        }

        let borrow_info = &BorrowingInformation::new(
            borrowing,
            game_data.customer_random_select(),
            borrow_date,
            return_date);
        
        Self::new(ctx, rect, paper_tid, &borrow_info,
                  game_data, 0)
    }
}

impl DrawableComponent for BorrowingPaper {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.canvas.begin_drawing(ctx);

            self.paper_texture.draw(ctx)?;
            self.title.draw(ctx)?;
            self.book_head.draw(ctx)?;
            self.borrower.draw(ctx)?;
            self.borrow_date.draw(ctx)?;
            self.return_date.draw(ctx)?;

            for d in &mut self.borrowing {
                d.draw(ctx)?;
            }

            self.canvas.end_drawing(ctx);
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

impl DrawableObject for BorrowingPaper {
    impl_drawable_object_for_wrapped!{canvas}
}

impl TextureObject for BorrowingPaper {
    impl_texture_object_for_wrapped!{canvas}
}

impl Clickable for BorrowingPaper {
    fn button_up(&mut self,
                 ctx: &mut ggez::Context,
		 _: &GameData,
		 _: Clock,
                 _button: ggez::input::mouse::MouseButton,
                 point: numeric::Point2f) {
        let rp = self.canvas.relative_point(point);
        
        if self.title.get_drawing_area(ctx).contains(rp) {
            println!("aaaaaaaaa");
        }
    }
}

#[derive(Clone)]
pub struct CopyingRequestInformation {
    pub book_info: BookInformation,
    pub customer: String,
    pub request_date: GensoDate,
    pub return_date: GensoDate,
}

impl CopyingRequestInformation {
    pub fn new(book_info: BookInformation,
               customer: String,
               request_date: GensoDate,
               return_date: GensoDate) -> Self {
        CopyingRequestInformation {
            book_info: book_info,
            customer: customer,
            request_date: request_date,
            return_date: return_date,
        }
    }

    pub fn new_random(game_data: &GameData,
                      request_date: GensoDate,
                      return_date: GensoDate) -> Self {
        let book_info = game_data.book_random_select();
        CopyingRequestInformation {
            book_info: book_info.clone(),
            customer: game_data.customer_random_select().to_string(),
            request_date: request_date,
            return_date: return_date,
        }
    }
}

pub struct CopyingRequestPaper {
    title: VerticalText,
    request_book: VerticalText,
    customer: VerticalText,
    request_date: VerticalText,
    return_date: VerticalText,
    book_type: VerticalText,
    pages: VerticalText,
    canvas: SubScreen,
    paper_texture: SimpleObject,
    raw_info: CopyingRequestInformation,
}

impl CopyingRequestPaper {
    pub fn new(ctx: &mut ggez::Context, rect: ggraphics::Rect, paper_tid: TextureID,
               info: CopyingRequestInformation, game_data: &GameData, t: Clock) -> Self {
        let default_scale = numeric::Vector2f::new(1.0, 1.0);
	let mut font_info = FontInformation::new(
	    game_data.get_font(FontID::JpFude1),
	    numeric::Vector2f::new(16.0, 16.0),
	    ggraphics::Color::from_rgba_u32(0x000000ff));
	let mut pos = numeric::Point2f::new(rect.w - 50.0, 0.0);
	
        let paper_texture = SimpleObject::new(MovableUniTexture::new(game_data.ref_texture(paper_tid),
                                                                     numeric::Point2f::new(0.0, 0.0),
                                                                     numeric::Vector2f::new(1.0, 1.0),
                                                                     0.0,
                                                                     0,
                                                                     move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                                                                     t),
                                              Vec::new());
	
	let title_text  = VerticalText::new("鈴奈庵 転写依頼票".to_string(),
					    numeric::Point2f::new(pos.x, 50.0), default_scale,
					    0.0, 0, font_info);
	pos.x -= 60.0;

	font_info.scale = numeric::Vector2f::new(20.0, 20.0);

	let request_book = VerticalText::new(format!("転写本    {}", info.book_info.name),
					     numeric::Point2f::new(pos.x, 50.0), default_scale,
					     0.0, 0, font_info);
	pos.x -= 24.0;
        
	let pages = VerticalText::new(format!("頁数   {}", number_to_jk(info.book_info.pages as u64)),
				      numeric::Point2f::new(pos.x, 50.0), default_scale,
				      0.0, 0, font_info);
	pos.x -= 24.0;
	
	let book_type = VerticalText::new(format!("寸法   {}", info.book_info.size),
					  numeric::Point2f::new(pos.x, 50.0), default_scale,
					  0.0, 0, font_info);
	pos.x -= 180.0;

	font_info.scale = numeric::Vector2f::new(16.0, 16.0);

	let customer = VerticalText::new(format!("依頼者 {}", info.customer),
					 numeric::Point2f::new(pos.x, 50.0), default_scale,
					 0.0, 0, font_info);
	pos.x -= 24.0;

	let request_date = VerticalText::new(format!("依頼日 {}", info.request_date.to_string()),
					     numeric::Point2f::new(pos.x, 50.0), default_scale,
					     0.0, 0, font_info);
	pos.x -= 24.0;
	
	let return_date = VerticalText::new(format!("完了予定 {}", info.return_date.to_string()),
					    numeric::Point2f::new(pos.x, 50.0), default_scale,
					    0.0, 0, font_info);
	pos.x -= 24.0;
        
        CopyingRequestPaper {
            title: title_text,
            request_book: request_book,
            customer: customer,
            paper_texture: paper_texture,
            request_date: request_date,
            return_date: return_date,
            pages: pages,
            canvas: SubScreen::new(ctx, rect, 0, ggraphics::BLACK),
            book_type: book_type,
	    raw_info: info
        }
    }
}

impl DrawableComponent for CopyingRequestPaper {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.canvas.begin_drawing(ctx);

            self.paper_texture.draw(ctx)?;
            self.title.draw(ctx)?;
            self.customer.draw(ctx)?;
            self.request_date.draw(ctx)?;
            self.return_date.draw(ctx)?;
            self.pages.draw(ctx)?;
            self.book_type.draw(ctx)?;
            self.request_book.draw(ctx)?;

            self.canvas.end_drawing(ctx);
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

impl DrawableObject for CopyingRequestPaper {
    impl_drawable_object_for_wrapped!{canvas}
}

impl TextureObject for CopyingRequestPaper {
    impl_texture_object_for_wrapped!{canvas}
}

impl Clickable for CopyingRequestPaper {
}

impl OnDesk for CopyingRequestPaper {
    fn ondesk_whose(&self) -> i32 {
	0
    }

    fn click_data(&self, ctx: &mut ggez::Context, point: numeric::Point2f) -> HoldData {
        let rpoint = self.canvas.relative_point(point);

	if self.request_book.get_drawing_area(ctx).contains(rpoint) {
	    return HoldData::BookName(self.raw_info.book_info.get_name().to_string())
	}

	if self.customer.get_drawing_area(ctx).contains(rpoint) {
	    return HoldData::CustomerName(self.raw_info.customer.to_string())
	}

	HoldData::None
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct MouseActionRecord {
    pub point: numeric::Point2f,
    pub t: Clock,
}

impl MouseActionRecord {
    fn new(point: numeric::Point2f, t: Clock) -> MouseActionRecord {
        MouseActionRecord {
            point: point,
            t: t
        }
    }

    fn new_empty() -> MouseActionRecord {
        MouseActionRecord {
            point: numeric::Point2f::new(0.0, 0.0),
            t: 0
        }
    }
}

pub struct MouseInformation {
    pub last_clicked: HashMap<MouseButton, MouseActionRecord>,
    pub last_dragged: HashMap<MouseButton, MouseActionRecord>,
    pub last_down: HashMap<MouseButton, MouseActionRecord>,
    pub last_up: HashMap<MouseButton, MouseActionRecord>,
    pub dragging: HashMap<MouseButton, bool>,
}

impl MouseInformation {

    pub fn new() -> MouseInformation {
        MouseInformation {
            last_clicked: hash![(MouseButton::Left, MouseActionRecord::new_empty()),
                                (MouseButton::Right, MouseActionRecord::new_empty()),
                                (MouseButton::Middle, MouseActionRecord::new_empty())],
            last_dragged: hash![(MouseButton::Left, MouseActionRecord::new_empty()),
                                (MouseButton::Right, MouseActionRecord::new_empty()),
                                (MouseButton::Middle, MouseActionRecord::new_empty())],
	    last_down: hash![(MouseButton::Left, MouseActionRecord::new_empty()),
				(MouseButton::Right, MouseActionRecord::new_empty()),
                             (MouseButton::Middle, MouseActionRecord::new_empty())],
	    last_up: hash![(MouseButton::Left, MouseActionRecord::new_empty()),
			     (MouseButton::Right, MouseActionRecord::new_empty()),
                             (MouseButton::Middle, MouseActionRecord::new_empty())],
            dragging: hash![(MouseButton::Left, false),
                            (MouseButton::Right, false),
                            (MouseButton::Middle, false)]
        }
    }

    pub fn get_last_clicked(&self, button: MouseButton) -> numeric::Point2f {
        match self.last_clicked.get(&button) {
            Some(x) => x.point,
            None => panic!("No such a mouse button"),
        }
    }

    pub fn set_last_clicked(&mut self, button: MouseButton, point: numeric::Point2f, t: Clock) {
        if self.last_clicked.insert(button, MouseActionRecord::new(point, t)) == None {
            panic!("No such a mouse button")
        }
    }

    pub fn get_last_dragged(&self, button: MouseButton) -> numeric::Point2f {
        match self.last_dragged.get(&button) {
            Some(x) => x.point,
            None => panic!("No such a mouse button"),
        }
    }

    pub fn set_last_dragged(&mut self, button: MouseButton, point: numeric::Point2f, t: Clock) {
        if self.last_dragged.insert(button, MouseActionRecord::new(point, t)) == None {
            panic!("No such a mouse button")
        }
    }
    
    pub fn get_last_down(&self, button: MouseButton) -> numeric::Point2f {
        match self.last_down.get(&button) {
            Some(x) => x.point,
            None => panic!("No such a mouse button"),
        }
    }

    pub fn set_last_down(&mut self, button: MouseButton, point: numeric::Point2f, t: Clock) {
        if self.last_down.insert(button, MouseActionRecord::new(point, t)) == None {
            panic!("No such a mouse button")
        }
    }
    
    pub fn get_last_up(&self, button: MouseButton) -> numeric::Point2f {
        match self.last_up.get(&button) {
            Some(x) => x.point,
            None => panic!("No such a mouse button"),
        }
    }

    pub fn set_last_up(&mut self, button: MouseButton, point: numeric::Point2f, t: Clock) {
        if self.last_up.insert(button, MouseActionRecord::new(point, t)) == None {
            panic!("No such a mouse button")
        }
    }

    pub fn is_dragging(&self, button: ginput::mouse::MouseButton) -> bool {
        match self.dragging.get(&button) {
            Some(x) => *x,
            None => panic!("No such a mouse button"),
        }
    }

    pub fn update_dragging(&mut self, button: MouseButton, drag: bool) {
        if self.dragging.insert(button, drag) == None {
            panic!("No such a mouse button")
        }
    }
    
}

pub struct BorrowingRecordBookPage {
    raw_info: BorrowingInformation,
    borrow_book: Vec<VerticalText>,
    book_head: VerticalText,
    borrower: VerticalText,
    borrow_date: VerticalText,
    return_date: VerticalText,
    paper_texture: SimpleObject,
    canvas: SubScreen,
}


impl BorrowingRecordBookPage {
    pub fn new(ctx: &mut ggez::Context, rect: ggraphics::Rect, paper_tid: TextureID,
               info: &BorrowingInformation, game_data: &GameData, t: Clock) -> Self {
        let mut pos = numeric::Point2f::new(rect.w - 40.0, 30.0);
        
        let borrower = VerticalText::new(format!("借りた人　{}", info.borrower),
                                         pos,
                                         numeric::Vector2f::new(1.0, 1.0),
                                         0.0,
                                         0,
                                         FontInformation::new(game_data.get_font(FontID::JpFude1),
                                                              numeric::Vector2f::new(16.0, 16.0),
                                                              ggraphics::Color::from_rgba_u32(0x000000ff)));
	pos.x -= 30.0;
	
        let book_head = VerticalText::new("貸出本".to_string(),
                                          pos,
                                          numeric::Vector2f::new(1.0, 1.0),
                                          0.0,
                                          0,
                                          FontInformation::new(game_data.get_font(FontID::JpFude1),
                                                               numeric::Vector2f::new(18.0, 18.0),
                                                               ggraphics::Color::from_rgba_u32(0x000000ff)));
	let mut borrowing: Vec<VerticalText> = info.borrowing.iter()
            .map(|book_info| {
                pos += numeric::Vector2f::new(-30.0, 0.0);
                VerticalText::new(book_info.name.to_string(),
                                  numeric::Point2f::new(pos.x, pos.y + 100.0),
                                  numeric::Vector2f::new(1.0, 1.0),
                                  0.0,
                                  0,
                                  FontInformation::new(game_data.get_font(FontID::JpFude1),
                                                       numeric::Vector2f::new(18.0, 18.0),
                                                       ggraphics::Color::from_rgba_u32(0x000000ff))) }).collect();

	for _ in 0..(6 - borrowing.len()) {
	    pos += numeric::Vector2f::new(-30.0, 0.0);
	    borrowing.push(VerticalText::new("　　　　　　".to_string(),
                                  numeric::Point2f::new(pos.x, pos.y + 100.0),
                                  numeric::Vector2f::new(1.0, 1.0),
                                  0.0,
                                  0,
                                  FontInformation::new(game_data.get_font(FontID::JpFude1),
                                                       numeric::Vector2f::new(18.0, 18.0),
                                                       ggraphics::Color::from_rgba_u32(0x000000ff))));
	}

        pos.x -= 30.0;

        let paper_texture = SimpleObject::new(MovableUniTexture::new(game_data.ref_texture(paper_tid),
                                                                     numeric::Point2f::new(0.0, 0.0),
                                                                     numeric::Vector2f::new(1.0, 1.0),
                                                                     0.0,
                                                                     0,
                                                                     move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                                                                     t),
                                              Vec::new());

        let borrow_date = VerticalText::new(format!("貸出日 {}", info.borrow_date.to_string()),
                                            numeric::Point2f::new(100.0, 50.0),
                                            numeric::Vector2f::new(1.0, 1.0),
                                            0.0,
                                            0,
                                            FontInformation::new(game_data.get_font(FontID::JpFude1),
                                                                 numeric::Vector2f::new(14.0, 14.0),
                                                                 ggraphics::Color::from_rgba_u32(0x000000ff)));
	pos.x -= 30.0;
        
        let return_date = VerticalText::new(format!("返却期限 {}", info.return_date.to_string()),
                                            numeric::Point2f::new(70.0, 50.0),
                                            numeric::Vector2f::new(1.0, 1.0),
                                            0.0,
                                            0,
                                            FontInformation::new(game_data.get_font(FontID::JpFude1),
                                                                 numeric::Vector2f::new(14.0, 14.0),
                                                                 ggraphics::Color::from_rgba_u32(0x000000ff)));
        
        BorrowingRecordBookPage {
	    raw_info: BorrowingInformation::new(info.borrowing.clone(), &info.borrower, info.borrow_date, info.return_date),
            borrow_book: borrowing,
            borrower: borrower,
            book_head: book_head,
            paper_texture: paper_texture,
            borrow_date: borrow_date,
            return_date: return_date,
            canvas: SubScreen::new(ctx, rect, 0, ggraphics::BLACK),
        }
    }

    pub fn new_empty(ctx: &mut ggez::Context, rect: ggraphics::Rect,
		     paper_tid: TextureID, game_data: &GameData, t: Clock) -> Self {
        let mut pos = numeric::Point2f::new(rect.w - 40.0, 30.0);
        
        let borrower = VerticalText::new("借りた人　".to_string(),
                                         pos,
                                         numeric::Vector2f::new(1.0, 1.0),
                                         0.0,
                                         0,
                                         FontInformation::new(game_data.get_font(FontID::JpFude1),
                                                              numeric::Vector2f::new(16.0, 16.0),
                                                              ggraphics::Color::from_rgba_u32(0x000000ff)));
	pos.x -= 30.0;
	
        let book_head = VerticalText::new("貸出本".to_string(),
                                          pos,
                                          numeric::Vector2f::new(1.0, 1.0),
                                          0.0,
                                          0,
                                          FontInformation::new(game_data.get_font(FontID::JpFude1),
                                                               numeric::Vector2f::new(18.0, 18.0),
                                                               ggraphics::Color::from_rgba_u32(0x000000ff)));
	let mut borrowing: Vec<VerticalText> = Vec::new();

	for _ in 0..(6 - borrowing.len()) {
	    pos += numeric::Vector2f::new(-30.0, 0.0);
	    borrowing.push(VerticalText::new("　　　　　　".to_string(),
                                  numeric::Point2f::new(pos.x, pos.y + 100.0),
                                  numeric::Vector2f::new(1.0, 1.0),
                                  0.0,
                                  0,
                                  FontInformation::new(game_data.get_font(FontID::JpFude1),
                                                       numeric::Vector2f::new(18.0, 18.0),
                                                       ggraphics::Color::from_rgba_u32(0x000000ff))));
	}

        pos.x -= 30.0;

        let paper_texture = SimpleObject::new(MovableUniTexture::new(game_data.ref_texture(paper_tid),
                                                                     numeric::Point2f::new(0.0, 0.0),
                                                                     numeric::Vector2f::new(1.0, 1.0),
                                                                     0.0,
                                                                     0,
                                                                     move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                                                                     t),
                                              Vec::new());

        let borrow_date = VerticalText::new("貸出日".to_string(),
                                            numeric::Point2f::new(100.0, 50.0),
                                            numeric::Vector2f::new(1.0, 1.0),
                                            0.0,
                                            0,
                                            FontInformation::new(game_data.get_font(FontID::JpFude1),
                                                                 numeric::Vector2f::new(14.0, 14.0),
                                                                 ggraphics::Color::from_rgba_u32(0x000000ff)));
	pos.x -= 30.0;
        
        let return_date = VerticalText::new("返却期限".to_string(),
                                            numeric::Point2f::new(70.0, 50.0),
                                            numeric::Vector2f::new(1.0, 1.0),
                                            0.0,
                                            0,
                                            FontInformation::new(game_data.get_font(FontID::JpFude1),
                                                                 numeric::Vector2f::new(14.0, 14.0),
                                                                 ggraphics::Color::from_rgba_u32(0x000000ff)));
        
        BorrowingRecordBookPage {
	    raw_info: BorrowingInformation::new(Vec::new(), "", GensoDate::new_empty(), GensoDate::new_empty()),
            borrow_book: borrowing,
            borrower: borrower,
            book_head: book_head,
            paper_texture: paper_texture,
            borrow_date: borrow_date,
            return_date: return_date,
            canvas: SubScreen::new(ctx, rect, 0, ggraphics::BLACK),
        }
    }

    pub fn get_borrowing_info(&self) -> &BorrowingInformation {
	&self.raw_info
    }

    pub fn get_borrowing_info_mut(&mut self) -> &mut BorrowingInformation {
	&mut self.raw_info
    }

    pub fn relative_point(&self, point: numeric::Point2f) -> numeric::Point2f {
	self.canvas.relative_point(point)
    }

    pub fn replace_borrower_name(&mut self, game_data: &GameData, name: &str) -> &mut Self {
	let pos = self.borrower.get_position();
	self.borrower = VerticalText::new(format!("借りた人   {}", name),
                                          pos,
                                          numeric::Vector2f::new(1.0, 1.0),
                                          0.0,
                                          0,
                                          FontInformation::new(game_data.get_font(FontID::JpFude1),
                                                               numeric::Vector2f::new(20.0, 20.0),
                                                               ggraphics::Color::from_rgba_u32(0x000000ff)));
	self
    }
}

impl DrawableComponent for BorrowingRecordBookPage {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.canvas.begin_drawing(ctx);

            self.paper_texture.draw(ctx)?;
            self.book_head.draw(ctx)?;
            self.borrower.draw(ctx)?;
            self.borrow_date.draw(ctx)?;
            self.return_date.draw(ctx)?;

            for d in &mut self.borrow_book {
                d.draw(ctx)?;
            }

            self.canvas.end_drawing(ctx);
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

impl DrawableObject for BorrowingRecordBookPage {

    #[inline(always)]
    fn set_position(&mut self, pos: numeric::Point2f) {
        self.canvas.set_position(pos);
    }

    #[inline(always)]
    fn get_position(&self) -> numeric::Point2f {
        self.canvas.get_position()
    }

    #[inline(always)]
    fn move_diff(&mut self, offset: numeric::Vector2f) {
        self.canvas.move_diff(offset)
    }
}

impl Clickable for BorrowingRecordBookPage {
    fn button_down(&mut self,
                   _ctx: &mut ggez::Context,
		   _: &GameData,
		   _: Clock,
                   _button: ggez::input::mouse::MouseButton,
                   _point: numeric::Point2f) {
    }
    
    fn button_up(&mut self,
                 _ctx: &mut ggez::Context,
		 _: &GameData,
		 _: Clock,
                 _button: ggez::input::mouse::MouseButton,
                 point: numeric::Point2f) {
	let rpoint = self.canvas.relative_point(point);
	println!("{:?}", rpoint);
    }
}

impl TextureObject for BorrowingRecordBookPage {
    impl_texture_object_for_wrapped!{canvas}
}

pub struct BorrowingRecordBook {
    pages: Vec<BorrowingRecordBookPage>,
    rect: numeric::Rect,
    current_page: usize,
    drwob_essential: DrawableObjectEssential,
}

impl BorrowingRecordBook {
    pub fn new(rect: ggraphics::Rect) -> Self {
        BorrowingRecordBook {
            pages: Vec::new(),
            rect: rect,
            current_page: 0,
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }

    pub fn add_page(&mut self, ctx: &mut ggez::Context,
                    info: &BorrowingInformation,
                    game_data: &GameData,
                    t: Clock) -> &Self {
	let page_rect = if let Some(page) = self.get_current_page() {
	    page.get_drawing_area(ctx)
	} else {
	    self.rect
	};
        self.pages.push(
            BorrowingRecordBookPage::new(ctx,
                                         page_rect,
                                         TextureID::Paper1,
                                         info,
                                         game_data, t));
        self
    }

    pub fn add_empty_page(&mut self, ctx: &mut ggez::Context,
			  game_data: &GameData,
			  t: Clock) -> &Self {
	let page_rect = if let Some(page) = self.get_current_page() {
	    page.get_drawing_area(ctx)
	} else {
	    self.rect
	};
        self.pages.push(
            BorrowingRecordBookPage::new_empty(ctx, page_rect, TextureID::Paper1, game_data, t));
        self
    }
    
    fn get_current_page(&self) -> Option<&BorrowingRecordBookPage> {
        self.pages.get(self.current_page)
    }

    fn get_current_page_mut(&mut self) -> Option<&mut BorrowingRecordBookPage> {
        self.pages.get_mut(self.current_page)
    }

    fn next_page(&mut self) {
	if self.current_page < self.pages.len() {
	    self.current_page += 1;
	}
    }

    fn prev_page(&mut self) {
	if self.current_page > 0 {
	    self.current_page -= 1;
	}
    }

    fn borrow_date_insert_check(ctx: &mut ggez::Context,
				rpoint: numeric::Point2f,
				page: &mut BorrowingRecordBookPage, data: &HoldData) -> bool {
	if page.borrow_date.get_drawing_area(ctx).contains(rpoint) {
	    match data {
		HoldData::Date(date_data) => {
		    page.borrow_date.replace_text(format!("貸出日 {}", date_data.to_string()));
		    return true;
		}
		_ => (),
	    }
	}

	return false;
    }

    fn borrower_customer_insert_check(ctx: &mut ggez::Context,
				      rpoint: numeric::Point2f,
				      page: &mut BorrowingRecordBookPage, data: &HoldData) -> bool {
	if page.borrower.get_drawing_area(ctx).contains(rpoint) {
	    match data {
		HoldData::CustomerName(customer_name) => {
		    page.borrower.replace_text(format!("借りる人 {}", customer_name.to_string()));
		    return true;
		}
		_ => (),
	    }
	}

	return false;
    }
}

impl DrawableComponent for BorrowingRecordBook {
    #[inline(always)]
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            if self.pages.len() > 0 {
                self.pages.get_mut(self.current_page).unwrap().draw(ctx)?;
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

impl DrawableObject for BorrowingRecordBook {

    #[inline(always)]
    fn set_position(&mut self, pos: numeric::Point2f) {
        self.rect.x = pos.x;
        self.rect.y = pos.y;

        for page in &mut self.pages {
            page.set_position(pos);
        }
    }

    #[inline(always)]
    fn get_position(&self) -> numeric::Point2f {
        self.rect.point().into()
    }

    #[inline(always)]
    fn move_diff(&mut self, offset: numeric::Vector2f) {
        self.rect.x += offset.x;
        self.rect.y += offset.y;

        for page in &mut self.pages {
            page.move_diff(offset);
        }
    }
}

impl TextureObject for BorrowingRecordBook {
    #[inline(always)]
    fn set_scale(&mut self, scale: numeric::Vector2f) {
        self.get_current_page_mut().unwrap().set_scale(scale);
    }
    
    #[inline(always)]
    fn get_scale(&self) -> numeric::Vector2f {
        self.get_current_page().unwrap().get_scale()
    }
    
    #[inline(always)]
    fn set_rotation(&mut self, rad: f32) {
        self.get_current_page_mut().unwrap().set_rotation(rad);
    }
    
    #[inline(always)]
    fn get_rotation(&self) -> f32 {
        self.get_current_page().unwrap().get_rotation()
    }

    #[inline(always)]
    fn set_crop(&mut self, crop: ggraphics::Rect) {
        self.get_current_page_mut().unwrap().set_crop(crop)
    }

    #[inline(always)]
    fn get_crop(&self) -> ggraphics::Rect {
        self.get_current_page().unwrap().get_crop()
    }

    #[inline(always)]
    fn set_drawing_color(&mut self, color: ggraphics::Color) {
        self.get_current_page_mut().unwrap().set_drawing_color(color)
    }

    #[inline(always)]
    fn get_drawing_color(&self) -> ggraphics::Color {
        self.get_current_page().unwrap().get_drawing_color()
    }

    #[inline(always)]
    fn set_alpha(&mut self, alpha: f32) {
        self.get_current_page_mut().unwrap().set_alpha(alpha)
    }

    #[inline(always)]
    fn get_alpha(&self) -> f32 {
        self.get_current_page().unwrap().get_alpha()
    }

    #[inline(always)]
    fn set_transform_offset(&mut self, offset: numeric::Point2f) {
        self.get_current_page_mut().unwrap().set_transform_offset(offset)
    }

    #[inline(always)]
    fn get_transform_offset(&self) -> numeric::Point2f {
        self.get_current_page().unwrap().get_transform_offset()
    }

    #[inline(always)]
    fn get_texture_size(&self, ctx: &mut ggez::Context) -> numeric::Vector2f {
        self.get_current_page().unwrap().get_texture_size(ctx)
    }

    #[inline(always)]
    fn replace_texture(&mut self, _: Rc<ggraphics::Image>) {
    }

    #[inline(always)]
    fn set_color(&mut self, color: ggraphics::Color) {
        self.get_current_page_mut().unwrap().set_color(color)
    }

    #[inline(always)]
    fn get_color(&mut self) -> ggraphics::Color {
        self.get_current_page_mut().unwrap().get_color()
    }
}

impl Clickable for BorrowingRecordBook {
    fn button_down(&mut self,
                   _ctx: &mut ggez::Context,
		   _: &GameData,
		   _: Clock,
                   _button: ggez::input::mouse::MouseButton,
                   _point: numeric::Point2f) {
    }
    
    fn button_up(&mut self,
                 ctx: &mut ggez::Context,
		 game_data: &GameData,
		 t: Clock,
                 button: ggez::input::mouse::MouseButton,
                 point: numeric::Point2f) {
	if let Some(page) = self.get_current_page_mut() {
	    let rpoint = page.relative_point(point);
	    
	    if rpoint.x < 20.0 {
		println!("next page!!");
		self.add_empty_page(ctx, game_data, t);
		self.next_page();
	    } else if rpoint.x > page.get_drawing_size(ctx).x - 20.0 {
		println!("prev page!!");
		self.prev_page();
	    } else {
		page.button_up(ctx, game_data, t, button, point);
	    }
	}
    }
}

impl OnDesk for BorrowingRecordBook {
    fn ondesk_whose(&self) -> i32 {
	0
    }

    fn click_data(&self, _: &mut ggez::Context, _: numeric::Point2f) -> HoldData {
	HoldData::None
    }

    fn insert_data(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f, data: &HoldData) -> bool {
	let mut insert_done_flag = false;
	
	if let Some(page) = self.get_current_page_mut() {
	    let rpoint = page.relative_point(point);
	    let mut hit_book_index = None;
	    
	    for (index, book) in page.borrow_book.iter().enumerate() {
		if book.get_drawing_area(ctx).contains(rpoint) {
		    hit_book_index = Some(index);
		}
	    }

	    if let Some(hit_book_index) = hit_book_index {
		match data {
		    HoldData::BookName(name) => {
			page.get_borrowing_info_mut().borrowing.push(BookInformation {
			    name: name.to_string(),
			    pages: 0,
			    size: "".to_string(),
			});
			page.borrow_book.get_mut(hit_book_index).unwrap().replace_text(name.to_string());
			insert_done_flag = true;
		    }
		    _ => (),
		}
	    }

	    if Self::borrow_date_insert_check(ctx, rpoint, page, data) ||
		Self::borrower_customer_insert_check(ctx, rpoint, page, data) {
		return true;
	    }
	}

	insert_done_flag
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeskObjectType {
    CustomerObject = 0,
    BorrowRecordBook,
    SuzunaObject,
}

pub struct DeskObject {
    small: Box<EffectableWrap<MovableWrap<dyn OnDesk>>>,
    large: Box<EffectableWrap<MovableWrap<dyn OnDesk>>>,
    switch: u8,
    object_type: DeskObjectType,
}

impl DeskObject {
    pub fn new(small: Box<dyn OnDesk>, large: Box<dyn OnDesk>,
	       switch: u8, obj_type: DeskObjectType, t: Clock) -> Self {
        DeskObject {
            small: Box::new(EffectableWrap::new(MovableWrap::new(small, None, t), Vec::new())),
            large: Box::new(EffectableWrap::new(MovableWrap::new(large, None, t), Vec::new())),
            switch: switch,
	    object_type: obj_type,
        }
    }

    pub fn enable_small(&mut self) {
        self.switch = 0;
    }

    pub fn enable_large(&mut self) {
        self.switch = 1;
    }

    pub fn get_object(&self) -> &Box<EffectableWrap<MovableWrap<dyn OnDesk>>> {
        match self.switch {
            0 => &self.small,
            1 => &self.large,
            _ => panic!("Failed to object selecting. select = {}", self.switch),
        }
    }

    pub fn get_object_mut(&mut self) -> &mut Box<EffectableWrap<MovableWrap<dyn OnDesk>>> {
        match self.switch {
            0 => &mut self.small,
            1 => &mut self.large,
            _ => panic!("Failed to object selecting. select = {}", self.switch),
        }
    }

    pub fn get_object_type(&self) -> DeskObjectType {
	self.object_type
    }
}

pub struct DeskObjectContainer {
    container: Vec<DeskObject>,
}

impl DeskObjectContainer {
    fn new() -> Self {
        DeskObjectContainer {
            container: Vec::new(),
        }
    }

    fn add(&mut self, obj: DeskObject) {
        self.container.push(obj);
    }

    fn sort_with_depth(&mut self) {
        self.container.sort_by(|a: &DeskObject, b: &DeskObject| {
            let (ad, bd) = (a.get_object().get_drawing_depth(), b.get_object().get_drawing_depth());
            if ad > bd {
                Ordering::Less
            } else if ad < bd {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });
    }

    fn get_raw_container(&self) -> &Vec<DeskObject> {
        &self.container
    }

    fn get_raw_container_mut(&mut self) -> &mut Vec<DeskObject> {
        &mut self.container
    }

    fn get_minimum_depth(&mut self) -> i8 {
        self.sort_with_depth();
        if let Some(depth) = self.container.last() {
            depth.get_object().get_drawing_depth()
        } else {
            127
        }
    }
    
    fn len(&self) -> usize {
        self.container.len()
    }

    fn change_depth_equally(&mut self, offset: i8)  {
        for obj in &mut self.container {
            let current_depth = obj.get_object().get_drawing_depth();
            let next_depth: i16 = (current_depth as i16) + (offset as i16);

            if next_depth <= 127 && next_depth >= -128 {
                // 範囲内
                obj.get_object_mut().set_drawing_depth(next_depth as i8);
            } else if next_depth > 0 {
                // 範囲外（上限）
                obj.get_object_mut().set_drawing_depth(127);
            } else {
                // 範囲外（下限）
                obj.get_object_mut().set_drawing_depth(-128);
            }
        }
    }
}

pub struct ObjectContainer<T> {
    container: Vec<T>,
}

impl<T> ObjectContainer<T> {
    pub fn new() -> Self {
        ObjectContainer {
            container: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn add(&mut self, obj: T) {
        self.container.push(obj);
    }

    #[inline(always)]
    pub fn remove_if<F>(&mut self, f: F)
    where F: Fn(&T) -> bool {
        self.container.retain(|e| !f(e));
    }

    pub fn len(&self) -> usize {
        self.container.len()
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        self.container.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        self.container.iter_mut()
    }
}

pub struct DeskObjects {
    canvas: SubScreen,
    desk_objects: DeskObjectContainer,
    dragging: Option<DeskObject>,
    table_texture: tobj::SimpleObject,
}

impl DeskObjects {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData,
               rect: ggraphics::Rect) -> DeskObjects {

        let mut dparam = ggraphics::DrawParam::default();
        dparam.dest = numeric::Point2f::new(rect.x, rect.y).into();
        
        let desk_objects = DeskObjectContainer::new();
        
        DeskObjects {
            canvas: SubScreen::new(ctx, rect, 0, ggraphics::Color::new(0.0, 0.0, 0.0, 0.0)),
            desk_objects: desk_objects,
            dragging: None,
            table_texture: tobj::SimpleObject::new(
                tobj::MovableUniTexture::new(game_data.ref_texture(TextureID::Wood1),
                                             numeric::Point2f::new(0.0, 0.0),
                                             numeric::Vector2f::new(1.0, 1.0),
                                             0.0, 0, move_fn::stop(), 0), Vec::new()),
        }
    }

    pub fn check_data_click(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) -> HoldData {
        let rpoint = self.canvas.relative_point(point);
	let mut clicked_data = HoldData::None;
        
        // オブジェクトは深度が深い順にソートされているので、
        // 逆順から検索していくことで、最も手前に表示されているオブジェクトを
        // 取り出すことができる
        for obj in self.desk_objects.get_raw_container_mut().iter_mut().rev() {
	    let contains = obj.get_object().get_drawing_area(ctx).contains(rpoint);
            if contains {
		clicked_data = obj.get_object_mut().ref_wrapped_object_mut().ref_wrapped_object_mut().click_data(ctx, rpoint);
                break;
            }
        }

	clicked_data
    }

    pub fn check_insert_data(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f, data: &HoldData) -> bool {
        let rpoint = self.canvas.relative_point(point);
        
        // オブジェクトは深度が深い順にソートされているので、
        // 逆順から検索していくことで、最も手前に表示されているオブジェクトを
        // 取り出すことができる
        for obj in self.desk_objects.get_raw_container_mut().iter_mut().rev() {
	    let contains = obj.get_object().get_drawing_area(ctx).contains(rpoint);
            if contains {
		return obj.get_object_mut().ref_wrapped_object_mut().ref_wrapped_object_mut().insert_data(ctx, rpoint, data);
            }
        }

	false
    }
    
    pub fn dragging_handler(&mut self,
                        point: numeric::Point2f,
                        last: numeric::Point2f) {
        if let Some(obj) = &mut self.dragging {
            obj.get_object_mut().move_diff(numeric::Vector2f::new(point.x - last.x, point.y - last.y));
        }
    }

    pub fn select_dragging_object(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {

        let mut dragging_object_index = 0;
        let mut drag_start = false;

        let rpoint = self.canvas.relative_point(point);
        
        // オブジェクトは深度が深い順にソートされているので、
        // 逆順から検索していくことで、最も手前に表示されているオブジェクトを
        // 取り出すことができる
        for (index, obj) in self.desk_objects.get_raw_container_mut().iter_mut().rev().enumerate() {
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
                    self.desk_objects.get_raw_container_mut().swap_remove(dragging_object_index)
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
            None =>  (),
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

    pub fn double_click_handler(&mut self,
                            ctx: &mut ggez::Context,
                            point: numeric::Point2f,
                            _game_data: &GameData) {
        let rpoint = self.canvas.relative_point(point);
        let mut click_flag = false;
        
        // オブジェクトは深度が深い順にソートされているので、
        // 逆順から検索していくことで、最も手前に表示されているオブジェクトを
        // 取り出すことができる
        for (_, obj) in self.desk_objects.get_raw_container_mut().iter_mut().rev().enumerate() {
            if obj.get_object().get_drawing_area(ctx).contains(rpoint) {
                click_flag = true;
                break;
            }
        }

        if click_flag {
            /*
            self.desk_objects.add(
                Box::new(CopyingRequestPaper::new(ctx, ggraphics::Rect::new(rpoint.x, rpoint.y, 420.0, 350.0), TextureID::Paper2,
                                                  &CopyingRequestInformation::new("テスト本1".to_string(),
                                                                                  "霧雨魔里沙".to_string(),
                                                                                  GensoDate::new(128, 12, 8),
                                                                                  GensoDate::new(128, 12, 8),
                                                                                  212),
                                                  game_data, 0))
            );
            */
            self.desk_objects.sort_with_depth();
        }
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
	let count = self.desk_objects.get_raw_container().iter().fold(0, |sum, obj| {
	    sum + if obj.get_object_type() == object_type { 1 } else { 0 }
	});
	count + if self.dragging.is_some() { 1 } else { 0 }
    }

    fn button_up_handler(&mut self,
			 ctx: &mut ggez::Context,
			 game_data: &GameData,
			 t: Clock,
			 button: ggez::input::mouse::MouseButton,
			 point: numeric::Point2f) {
	let rpoint = self.canvas.relative_point(point);
	
	for dobj in self.desk_objects.get_raw_container_mut() {
	    if dobj.get_object_mut().get_drawing_area(ctx).contains(rpoint) {
		dobj.get_object_mut()
		    .ref_wrapped_object_mut()
		    .ref_wrapped_object_mut()
		    .button_up(ctx, game_data, t, button, rpoint);
	    }
	}
    }

    pub fn check_mouse_cursor_status(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) -> MouseCursor {
	if self.canvas.get_drawing_area(ctx).contains(point) {
	    let rpoint = self.canvas.relative_point(point);

	    // オブジェクトは深度が深い順にソートされているので、
            // 逆順から検索していくことで、最も手前に表示されているオブジェクトを
            // 取り出すことができる
            for obj in self.desk_objects.get_raw_container_mut().iter_mut().rev() {
		if obj.get_object().get_drawing_area(ctx).contains(rpoint) {
		    return MouseCursor::Grab
		}
            }
	}

	MouseCursor::Default
    }
}

impl DrawableComponent for DeskObjects {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.canvas.begin_drawing(ctx);

            self.table_texture.draw(ctx)?;
            
            for obj in self.desk_objects.get_raw_container_mut() {
                obj.get_object_mut().draw(ctx)?;
            }
            
            if let Some(ref mut d) = self.dragging {
                d.get_object_mut().draw(ctx)?;
            }
            
            self.canvas.end_drawing(ctx);
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

struct TaskSilhouette {
    character: Option<SimpleObject>,
    name: Option<String>,
    canvas: SubScreen,
}

impl TaskSilhouette {
    pub fn new(ctx: &mut ggez::Context, pos_rect: numeric::Rect, char_obj: SimpleObject, name: &str) -> Self {
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
            self.canvas.begin_drawing(ctx);

	    if let Some(character) = &mut self.character {
		character.draw(ctx)?;
	    }
            
            self.canvas.end_drawing(ctx);
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
    impl_drawable_object_for_wrapped!{canvas}
}

impl TextureObject for TaskSilhouette {
    impl_texture_object_for_wrapped!{canvas}
}

impl Clickable for TaskSilhouette {
    fn clickable_status(&mut self,
			ctx: &mut ggez::Context,
			point: numeric::Point2f) -> ggez::input::mouse::MouseCursor {
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
}

pub struct SuzuMiniSightSilhouette {
    background: MovableUniTexture,
    silhouette: TaskSilhouette,
    canvas: SubScreen,
}

impl SuzuMiniSightSilhouette {
    pub fn new(ctx: &mut ggez::Context, rect: numeric::Rect, background: MovableUniTexture) -> Self {
	SuzuMiniSightSilhouette {
	    background: background,
	    silhouette: TaskSilhouette::new_empty(ctx, rect),
	    canvas: SubScreen::new(ctx, rect, 0, ggraphics::Color::from_rgba_u32(0x00000000)),
	}
    }

    pub fn replace_character(&mut self, chara: SimpleObject, name: String) {
	self.silhouette
	    .change_character(chara)
	    .update_name(name);
	
    }

    fn run_effect(&mut self, ctx: &mut ggez::Context, t: Clock) {
	if self.silhouette.is_some() {
	    self.silhouette.get_object_mut().unwrap().move_with_func(t);
	    self.silhouette.get_object_mut().unwrap().effect(ctx, t);
	}
    }
}

impl DrawableComponent for SuzuMiniSightSilhouette {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.canvas.begin_drawing(ctx);

	    self.background.draw(ctx)?;
	    if self.silhouette.is_some() {
		self.silhouette.draw(ctx)?;
	    }
            
            self.canvas.end_drawing(ctx);
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
    impl_drawable_object_for_wrapped!{canvas}
}

impl TextureObject for SuzuMiniSightSilhouette {
    impl_texture_object_for_wrapped!{canvas}
}

impl Clickable for SuzuMiniSightSilhouette {
    fn clickable_status(&mut self,
			ctx: &mut ggez::Context,
			point: numeric::Point2f) -> ggez::input::mouse::MouseCursor {
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
}


pub struct SuzuMiniSight {
    canvas: SubScreen,
    dragging: Option<DeskObject>,
    dropping: Vec<DeskObject>,
    dropping_to_desk: Vec<DeskObject>,
    silhouette: SuzuMiniSightSilhouette,
}

impl SuzuMiniSight {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData,
               rect: ggraphics::Rect) -> Self {
        
        SuzuMiniSight {
            canvas: SubScreen::new(ctx, rect, 0, ggraphics::Color::new(0.0, 0.0, 0.0, 0.0)),
            dragging: None,
	    dropping: Vec::new(),
	    dropping_to_desk: Vec::new(),
	    silhouette: SuzuMiniSightSilhouette::new(ctx,
						     rect,
						     MovableUniTexture::new(game_data.ref_texture(TextureID::Paper1),
									    numeric::Point2f::new(-100.0, 0.0),
									    numeric::Vector2f::new(1.0, 1.0),
									    0.0,
									    0,
									    move_fn::stop(),
									    0)),
        }
    }

    pub fn replace_character_silhouette(&mut self, chara: SimpleObject, name: String) {
	self.silhouette.replace_character(chara, name);
    }
    
    pub fn dragging_handler(&mut self,
                        point: numeric::Point2f,
                        last: numeric::Point2f) {
        if let Some(obj) = &mut self.dragging {
            obj.get_object_mut().move_diff(numeric::Vector2f::new(point.x - last.x, point.y - last.y));
        }
    }

    fn check_object_drop(&self, ctx: &mut ggez::Context, desk_obj: &DeskObject) -> bool {
	let area = desk_obj.get_object().get_drawing_area(ctx);
	area.y + area.h < self.canvas.get_drawing_area(ctx).h
    }

    fn check_object_drop_to_desk(&self, ctx: &mut ggez::Context, desk_obj: &DeskObject) -> bool {
	let area = desk_obj.get_object().get_drawing_area(ctx);
	area.y + area.h < self.canvas.get_drawing_area(ctx).h / 1.5
    }
    
    pub fn update(&mut self, ctx: &mut ggez::Context, t: Clock) {
	self.dropping.retain(|d| !d.get_object().is_stop());

	for d in &mut self.dropping {
            d.get_object_mut().move_with_func(t);
	    d.get_object_mut().effect(ctx, t);
        }

	for d in &mut self.dropping_to_desk {
            d.get_object_mut().move_with_func(t);
	    d.get_object_mut().effect(ctx, t);
        }

	self.silhouette.run_effect(ctx, t);
    }

    pub fn check_drop_desk(&mut self) -> Vec<DeskObject> {
	let mut drop_to_desk = Vec::new();

	let mut index = 0;
	while index < self.dropping_to_desk.len() {
	    let stop = self.dropping_to_desk.get(index).unwrap().get_object().is_stop();
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
		dragged.get_object_mut().override_move_func(move_fn::gravity_move(1.0, 10.0, 310.0, 0.3), t);
		dragged.get_object_mut().add_effect(vec![
		    Box::new(|obj: &mut dyn MovableObject, _: &ggez::Context, t: Clock| {
			if obj.get_position().y > 350.0 { obj.override_move_func(None, t); EffectFnStatus::EffectFinish }
			else { EffectFnStatus::EffectContinue }
		    })
		]);
		self.dropping.push(dragged);
	    } else {
		dragged.get_object_mut().override_move_func(move_fn::gravity_move(1.0, 10.0, 310.0, 0.3), t);
		dragged.get_object_mut().add_effect(vec![
		    Box::new(|obj: &mut dyn MovableObject, _: &ggez::Context, t: Clock| {
			if obj.get_position().y > 300.0 { obj.override_move_func(None, t); EffectFnStatus::EffectFinish }
			else { EffectFnStatus::EffectContinue }
		    })
		]);
		self.dropping_to_desk.push(dragged);
	    }
        }
    }

    pub fn check_data_click(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) -> HoldData {
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

    pub fn check_mouse_cursor_status(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) -> MouseCursor {
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
            self.canvas.begin_drawing(ctx);

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
            
            self.canvas.end_drawing(ctx);
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
	    calendar: DrawableCalendar::new(ctx, game_data, numeric::Rect::new(0.0, 0.0, 100.0, 100.0),
					    GensoDate::new(12, 12, 12), TextureID::Paper1),
	    canvas: SubScreen::new(ctx, pos_rect, 0, ggraphics::Color::from_rgba_u32(0x00000000)),
	}
    }

    pub fn check_data_click(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) -> HoldData {
        let rpoint = self.canvas.relative_point(point);
	
        if self.calendar.get_drawing_area(ctx).contains(rpoint) {
	    return self.calendar.click_data(ctx, rpoint);
        }

	HoldData::None
    }
    
    pub fn check_mouse_cursor_status(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) -> MouseCursor {
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
            self.canvas.begin_drawing(ctx);
	    
	    self.calendar.draw(ctx)?;

            self.canvas.end_drawing(ctx);
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

#[derive(Clone)]
pub enum CustomerRequest {
    Borrowing(BorrowingInformation),
    Copying(CopyingRequestInformation),
}

pub struct TaskTable {
    canvas: SubScreen,
    sight: SuzuMiniSight,
    desk: DeskObjects,
    goods: Goods,
    hold_data: HoldData,
}

impl TaskTable {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData,
               pos: numeric::Rect,
               sight_rect: ggraphics::Rect,
	       goods_rect: ggraphics::Rect,
	       desk_rect: ggraphics::Rect, t: Clock) -> Self {
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
        
        TaskTable {
            canvas: SubScreen::new(ctx, pos, 0, ggraphics::Color::from_rgba_u32(0x00000000)),
            sight: sight,
            desk: desk,
	    goods: Goods::new(ctx, game_data, goods_rect),
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
        for _ in info.borrowing {
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
	    vec![effect::appear_bale_from_bottom(50, t), effect::fade_in(50, t)]);
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
            self.canvas.begin_drawing(ctx);

            self.sight.draw(ctx).unwrap();
            self.desk.draw(ctx).unwrap();
	    self.goods.draw(ctx)?;
            
            self.canvas.end_drawing(ctx);
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
