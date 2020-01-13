pub mod factory;

use std::collections::HashMap;

use ggez::graphics as ggraphics;
use ggez::input as ginput;
use ginput::mouse::MouseButton;

use torifune::graphics::object::*;
use torifune::graphics::*;
use torifune::numeric;

use crate::core::BookInformation;

use torifune::hash;

use super::*;

use crate::core::{TextureID, FontID, GameData};

use number_to_jk::number_to_jk;

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

    pub fn to_string(&self) -> String {
        format!("第{}季 {}月 {}日",
		number_to_jk(self.season as u64),
		number_to_jk(self.month as u64),
		number_to_jk(self.day as u64))
    }
}

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
                      game_data: &GameData, t: Clock) -> Self {

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

impl Clickable for BorrowingPaper {    
    fn button_up(&mut self,
                 ctx: &mut ggez::Context,
                 _button: ggez::input::mouse::MouseButton,
                 point: numeric::Point2f) {
        let rp = self.canvas.relative_point(point);
        
        if self.title.get_drawing_area(ctx).contains(rp) {
            println!("aaaaaaaaa");
        }
    }
}

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
    title: SimpleText,
    request_book: SimpleText,
    customer: SimpleText,
    request_date: SimpleText,
    return_date: SimpleText,
    book_type: SimpleText,
    pages: SimpleText,
    canvas: SubScreen,
    paper_texture: SimpleObject,
}

impl CopyingRequestPaper {
    pub fn new(ctx: &mut ggez::Context, rect: ggraphics::Rect, paper_tid: TextureID,
               info: &CopyingRequestInformation, game_data: &GameData, t: Clock) -> Self {
        
        let paper_texture = SimpleObject::new(MovableUniTexture::new(game_data.ref_texture(paper_tid),
                                                                     numeric::Point2f::new(0.0, 0.0),
                                                                     numeric::Vector2f::new(1.0, 1.0),
                                                                     0.0,
                                                                     0,
                                                                     move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                                                                     t),
                                              Vec::new());
        
        let title_text = SimpleText::new(MovableText::new("鈴奈庵 転写依頼票".to_string(),
                                                          numeric::Point2f::new(120.0, 50.0),
                                                          numeric::Vector2f::new(1.0, 1.0),
                                                          0.0,
                                                          0,
                                                          move_fn::stop(),
                                                          FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                                               numeric::Vector2f::new(20.0, 20.0),
                                                                               ggraphics::Color::from_rgba_u32(0x000000ff)),
                                                          t),
                                         Vec::new());

        let customer = SimpleText::new(MovableText::new(format!("依頼者   {}", info.customer),
                                                        numeric::Point2f::new(50.0, 100.0),
                                                        numeric::Vector2f::new(1.0, 1.0),
                                                        0.0,
                                                        0,
                                                        move_fn::stop(),
                                                        FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                                             numeric::Vector2f::new(19.0, 19.0),
                                                                             ggraphics::Color::from_rgba_u32(0x000000ff)),
                                                        t),
                                       Vec::new());

        let request_date = SimpleText::new(MovableText::new(format!("依頼日     {}", info.request_date.to_string()),
                                                        numeric::Point2f::new(50.0, 135.0),
                                                        numeric::Vector2f::new(1.0, 1.0),
                                                        0.0,
                                                        0,
                                                        move_fn::stop(),
                                                        FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                                             numeric::Vector2f::new(19.0, 19.0),
                                                                             ggraphics::Color::from_rgba_u32(0x000000ff)),
                                                        t),
                                          Vec::new());

        let return_date = SimpleText::new(MovableText::new(format!("完了予定   {}", info.return_date.to_string()),
                                                           numeric::Point2f::new(50.0, 170.0),
                                                           numeric::Vector2f::new(1.0, 1.0),
                                                           0.0,
                                                           0,
                                                           move_fn::stop(),
                                                           FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                                                numeric::Vector2f::new(19.0, 19.0),
                                                                                ggraphics::Color::from_rgba_u32(0x000000ff)),
                                                           t),
                                          Vec::new());
        
        let pages = SimpleText::new(MovableText::new(format!("頁数   {}", info.book_info.pages),
                                                     numeric::Point2f::new(50.0, 275.0),
                                                     numeric::Vector2f::new(1.0, 1.0),
                                                     0.0,
                                                     0,
                                                     move_fn::stop(),
                                                     FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                                          numeric::Vector2f::new(19.0, 19.0),
                                                                          ggraphics::Color::from_rgba_u32(0x000000ff)),
                                                     t),
                                    Vec::new());
        
        let book_type = SimpleText::new(MovableText::new(format!("寸法   {}", info.book_info.size),
                                                         numeric::Point2f::new(50.0, 240.0),
                                                         numeric::Vector2f::new(1.0, 1.0),
                                                         0.0,
                                                         0,
                                                         move_fn::stop(),
                                                         FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                                              numeric::Vector2f::new(19.0, 19.0),
                                                                              ggraphics::Color::from_rgba_u32(0x000000ff)),
                                                         t),
                                        Vec::new());

        let request_book = SimpleText::new(MovableText::new(format!("転写本    {}", info.book_info.name),
                                                            numeric::Point2f::new(50.0, 205.0),
                                                            numeric::Vector2f::new(1.0, 1.0),
                                                            0.0,
                                                            0,
                                                            move_fn::stop(),
                                                            FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                                                 numeric::Vector2f::new(19.0, 19.0),
                                                                                 ggraphics::Color::from_rgba_u32(0x000000ff)),
                                                            t),
                                           Vec::new());
        
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

impl TextureObject for CopyingRequestPaper {
    #[inline(always)]
    fn set_scale(&mut self, scale: numeric::Vector2f) {
        self.canvas.set_scale(scale)
    }

    #[inline(always)]
    fn get_scale(&self) -> numeric::Vector2f {
        self.canvas.get_scale()
    }

    #[inline(always)]
    fn set_rotation(&mut self, rad: f32) {
        self.canvas.set_rotation(rad)
    }

    #[inline(always)]
    fn get_rotation(&self) -> f32 {
        self.canvas.get_rotation()
    }

    #[inline(always)]
    fn set_crop(&mut self, crop: ggraphics::Rect) {
        self.canvas.set_crop(crop);
    }

    #[inline(always)]
    fn get_crop(&self) -> ggraphics::Rect {
        self.canvas.get_crop()
    }

    #[inline(always)]
    fn set_drawing_color(&mut self, color: ggraphics::Color) {
        self.canvas.set_drawing_color(color)
    }

    #[inline(always)]
    fn get_drawing_color(&self) -> ggraphics::Color {
        self.canvas.get_drawing_color()
    }

    #[inline(always)]
    fn set_alpha(&mut self, alpha: f32) {
        self.canvas.set_alpha(alpha);
    }

    #[inline(always)]
    fn get_alpha(&self) -> f32 {
        self.canvas.get_alpha()
    }

    #[inline(always)]
    fn set_transform_offset(&mut self, offset: numeric::Point2f) {
        self.canvas.set_transform_offset(offset);
    }

    #[inline(always)]
    fn get_transform_offset(&self) -> numeric::Point2f {
        self.canvas.get_transform_offset()
    }

    #[inline(always)]
    fn get_drawing_area(&self, ctx: &mut ggez::Context) -> ggraphics::Rect {
        self.canvas.get_drawing_area(ctx)
    }

    #[inline(always)]
    fn get_drawing_size(&self, ctx: &mut ggez::Context) -> numeric::Vector2f {
        self.canvas.get_drawing_size(ctx)
    }

    #[inline(always)]
    fn get_texture_size(&self, ctx: &mut ggez::Context) -> numeric::Vector2f {
        self.canvas.get_texture_size(ctx)
    }

    #[inline(always)]
    fn replace_texture(&mut self, texture: Rc<ggraphics::Image>) {
        self.canvas.replace_texture(texture);
    }

    #[inline(always)]
    fn set_color(&mut self, color: ggraphics::Color) {
        self.canvas.set_color(color);
    }

    #[inline(always)]
    fn get_color(&mut self) -> ggraphics::Color {
        self.canvas.get_color()
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
    borrow_book: Vec<VerticalText>,
    book_head: VerticalText,
    borrower: VerticalText,
    borrow_date: VerticalText,
    return_date: VerticalText,
    paper_texture: SimpleObject,
    canvas: SubScreen,
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

impl BorrowingRecordBookPage {
    pub fn new(ctx: &mut ggez::Context, rect: ggraphics::Rect, paper_tid: TextureID,
               info: &BorrowingInformation, game_data: &GameData, t: Clock) -> Self {
        let mut pos = numeric::Point2f::new(210.0, 370.0);
        let borrowing = info.borrowing.iter()
            .map(|book_info| {
                pos += numeric::Vector2f::new(0.0, 30.0);
                VerticalText::new(book_info.name.to_string(),
                                  pos,
                                  numeric::Vector2f::new(1.0, 1.0),
                                  0.0,
                                  0,
                                  FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                       numeric::Vector2f::new(24.0, 24.0),
                                                       ggraphics::Color::from_rgba_u32(0x000000ff))) }).collect();
        
        let borrower = VerticalText::new(format!("借りた人   {}", info.borrower),
                                         numeric::Point2f::new(rect.x + rect.w - 28.0, 20.0),
                                         numeric::Vector2f::new(1.0, 1.0),
                                         0.0,
                                         0,
                                         FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                              numeric::Vector2f::new(20.0, 20.0),
                                                              ggraphics::Color::from_rgba_u32(0x000000ff)));

        let book_head = VerticalText::new("貸出本".to_string(),
                                          numeric::Point2f::new(rect.x + rect.w - 56.0, 80.0),
                                          numeric::Vector2f::new(1.0, 1.0),
                                          0.0,
                                          0,
                                          FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                               numeric::Vector2f::new(22.0, 22.0),
                                                               ggraphics::Color::from_rgba_u32(0x000000ff)));

        let paper_texture = SimpleObject::new(MovableUniTexture::new(game_data.ref_texture(paper_tid),
                                                                     numeric::Point2f::new(0.0, 0.0),
                                                                     numeric::Vector2f::new(1.0, 1.0),
                                                                     0.0,
                                                                     0,
                                                                     move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                                                                     t),
                                              Vec::new());

        let borrow_date = VerticalText::new(format!("貸出日 {}", info.borrow_date.to_string()),
                                            numeric::Point2f::new(rect.x + rect.w - 80.0, 80.0),
                                            numeric::Vector2f::new(1.0, 1.0),
                                            0.0,
                                            0,
                                            FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                                 numeric::Vector2f::new(18.0, 18.0),
                                                                 ggraphics::Color::from_rgba_u32(0x000000ff)));
        
        let return_date = VerticalText::new(format!("返却期限 {}", info.return_date.to_string()),
                                            numeric::Point2f::new(rect.x + rect.w - 108.0, 80.0),
                                            numeric::Vector2f::new(1.0, 1.0),
                                            0.0,
                                            0,
                                            FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                                 numeric::Vector2f::new(18.0, 18.0),
                                                                 ggraphics::Color::from_rgba_u32(0x000000ff)));
        
        BorrowingRecordBookPage {
            borrow_book: borrowing,
            borrower: borrower,
            book_head: book_head,
            paper_texture: paper_texture,
            borrow_date: borrow_date,
            return_date: return_date,
            canvas: SubScreen::new(ctx, rect, 0, ggraphics::BLACK),
        }
    }
}

impl TextureObject for BorrowingRecordBookPage {
    #[inline(always)]
    fn set_scale(&mut self, scale: numeric::Vector2f) {
        self.canvas.set_scale(scale);
    }

    #[inline(always)]
    fn get_scale(&self) -> numeric::Vector2f {
        self.canvas.get_scale()
    }

    #[inline(always)]
    fn set_rotation(&mut self, rad: f32) {
        self.canvas.set_rotation(rad);
    }

    #[inline(always)]
    fn get_rotation(&self) -> f32 {
        self.canvas.get_rotation()
    }

    #[inline(always)]
    fn set_crop(&mut self, crop: ggraphics::Rect) {
        self.canvas.set_crop(crop)
    }

    #[inline(always)]
    fn get_crop(&self) -> ggraphics::Rect {
        self.canvas.get_crop()
    }

    #[inline(always)]
    fn set_drawing_color(&mut self, color: ggraphics::Color) {
        self.canvas.set_drawing_color(color)
    }

    #[inline(always)]
    fn get_drawing_color(&self) -> ggraphics::Color {
        self.canvas.get_drawing_color()
    }

    #[inline(always)]
    fn set_alpha(&mut self, alpha: f32) {
        self.canvas.set_alpha(alpha)
    }

    #[inline(always)]
    fn get_alpha(&self) -> f32 {
        self.canvas.get_alpha()
    }

    #[inline(always)]
    fn set_transform_offset(&mut self, offset: numeric::Point2f) {
        self.set_transform_offset(offset)
    }

    #[inline(always)]
    fn get_transform_offset(&self) -> numeric::Point2f {
        self.canvas.get_transform_offset()
    }

    #[inline(always)]
    fn get_texture_size(&self, ctx: &mut ggez::Context) -> numeric::Vector2f {
        self.canvas.get_texture_size(ctx)
    }

    #[inline(always)]
    fn replace_texture(&mut self, texture: Rc<ggraphics::Image>) {
    }

    #[inline(always)]
    fn set_color(&mut self, color: ggraphics::Color) {
        self.canvas.set_color(color)
    }

    #[inline(always)]
    fn get_color(&mut self) -> ggraphics::Color {
        self.canvas.get_color()
    }
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
        self.pages.push(
            BorrowingRecordBookPage::new(ctx,
                                         self.rect,
                                         TextureID::Paper1,
                                         info,
                                         game_data, t));
        self
    }
    
    fn get_current_page(&self) -> Option<&BorrowingRecordBookPage> {
        self.pages.get(self.current_page)
    }

    fn get_current_page_mut(&mut self) -> Option<&mut BorrowingRecordBookPage> {
        self.pages.get_mut(self.current_page)
    }
}

impl DrawableComponent for BorrowingRecordBook {
    #[inline(always)]
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            if self.pages.len() > 0 {
                self.pages.get_mut(self.current_page).unwrap().draw(ctx);
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
    fn replace_texture(&mut self, texture: Rc<ggraphics::Image>) {
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

pub struct DeskObject {
    small: Box<dyn TextureObject>,
    large: Box<dyn TextureObject>,
    switch: u8,
}

impl DeskObject {
    pub fn new(small: Box<dyn TextureObject>, large: Box<dyn TextureObject>, switch: u8) -> Self {
        DeskObject {
            small: small,
            large: large,
            switch: switch,
        }
    }

    pub fn enable_small(&mut self) {
        self.switch = 0;
    }

    pub fn enable_large(&mut self) {
        self.switch = 1;
    }

    pub fn get_object(&self) -> &Box<dyn TextureObject> {
        match self.switch {
            0 => &self.small,
            1 => &self.large,
            _ => panic!("Failed to object selecting. select = {}", self.switch),
        }
    }

    pub fn get_object_mut(&mut self) -> &mut Box<dyn TextureObject> {
        match self.switch {
            0 => &mut self.small,
            1 => &mut self.large,
            _ => panic!("Failed to object selecting. select = {}", self.switch),
        }
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
               rect: ggraphics::Rect, desk_size: u8) -> DeskObjects {

        let mut dparam = ggraphics::DrawParam::default();
        dparam.dest = numeric::Point2f::new(rect.x, rect.y).into();
        
        let mut desk_objects = DeskObjectContainer::new();
        
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

    pub fn update(&mut self, _ctx: &mut ggez::Context, t: Clock) {
        /*
        for p in self.desk_objects.get_raw_container_mut() {
            p.move_with_func(t);
        }
        */
    }    

    pub fn double_click_handler(&mut self,
                            ctx: &mut ggez::Context,
                            point: numeric::Point2f,
                            game_data: &GameData) {
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

pub struct DeskSight {
    canvas: SubScreen,
    desk: DeskObjects,
}

impl DeskSight {
    pub fn new(ctx: &mut ggez::Context,
               game_data: &GameData,
               rect: ggraphics::Rect) -> Self {
        
        DeskSight {
            canvas: SubScreen::new(ctx, rect, 0, ggraphics::Color::from_rgba_u32(0x00000000)),
            desk: DeskObjects::new(ctx, game_data,
                                   numeric::Rect::new(rect.x, rect.y + (rect.h / 2.0), rect.w, rect.h / 2.0), 0),
        }
    }

    pub fn get_desk(&mut self) -> &DeskObjects {
        &self.desk
    }

    pub fn get_desk_mut(&mut self) -> &mut DeskObjects {
        &mut self.desk
    }
}

impl DrawableComponent for DeskSight {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.canvas.begin_drawing(ctx);

            self.get_desk_mut().draw(ctx).unwrap();
            
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

impl DrawableObject for DeskSight {

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


pub struct TaskTable {
    canvas: SubScreen,
    left: DeskSight,
    right: DeskObjects,
}

impl TaskTable {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData,
               pos: numeric::Rect,
               left_rect: ggraphics::Rect, right_rect: ggraphics::Rect) -> Self {
        let mut left = DeskSight::new(ctx, game_data, left_rect);
        let mut right = DeskObjects::new(ctx, game_data, right_rect, 1);
        
        right.add_object(DeskObject::new(
                Box::new(tobj::SimpleObject::new(
                    tobj::MovableUniTexture::new(
                        game_data.ref_texture(TextureID::LotusPink),
                        numeric::Point2f::new(0.0, 0.0),
                        numeric::Vector2f::new(0.1, 0.1),
                        0.0, -1, move_fn::stop(),
                        0), vec![])),
                Box::new(tobj::SimpleObject::new(
                    tobj::MovableUniTexture::new(
                        game_data.ref_texture(TextureID::LotusPink),
                        numeric::Point2f::new(0.0, 0.0),
                        numeric::Vector2f::new(0.1, 0.1),
                        0.0, -1, move_fn::stop(),
                        0), vec![])), 1));

        let mut book = Box::new(BorrowingRecordBook::new(ggraphics::Rect::new(0.0, 0.0, 400.0, 400.0)));
        book.add_page(ctx,
                      &BorrowingInformation::new_random(game_data, GensoDate::new(12, 12, 12), GensoDate::new(12, 12, 12)),
                      game_data, 0);
        left.get_desk_mut().add_object(DeskObject::new(
            Box::new(tobj::SimpleObject::new(
                tobj::MovableUniTexture::new(
                    game_data.ref_texture(TextureID::LotusPink),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(0.1, 0.1),
                    0.0, -1, move_fn::stop(),
                    0), vec![])),
            book, 0));
        
        TaskTable {
            canvas: SubScreen::new(ctx, pos, 0, ggraphics::Color::from_rgba_u32(0x00000000)),
            left: left,
            right: right,
        }
    }

    pub fn select_dragging_object(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        let rpoint = self.canvas.relative_point(point);
        self.left.get_desk_mut().select_dragging_object(ctx, rpoint);
        self.right.select_dragging_object(ctx, rpoint);
    }

    pub fn double_click_handler(&mut self,
                                ctx: &mut ggez::Context,
                                point: numeric::Point2f,
                                game_data: &GameData) {
        let rpoint = self.canvas.relative_point(point);
        self.left.get_desk_mut().double_click_handler(ctx, rpoint, game_data);
        self.right.double_click_handler(ctx, rpoint, game_data);
    }

    pub fn dragging_handler(&mut self,
                            point: numeric::Point2f,
                            last: numeric::Point2f) {
        let rpoint = self.canvas.relative_point(point);
        let rlast = self.canvas.relative_point(last);
        
        self.left.get_desk_mut().dragging_handler(rpoint, rlast);
        self.right.dragging_handler(rpoint, rlast);
    }

    pub fn unselect_dragging_object(&mut self) {
        self.left.get_desk_mut().unselect_dragging_object();
        self.right.unselect_dragging_object();
    }

    pub fn hand_over_check(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        let rpoint = self.canvas.relative_point(point);
        self.hand_over_check_r2l(ctx, rpoint);
        self.hand_over_check_l2r(ctx, rpoint);
    }

    fn hand_over_check_r2l(&mut self, ctx: &mut ggez::Context, rpoint: numeric::Point2f) {
        let border = self.desk_border(ctx);
        
        if self.right.has_dragging() && border > rpoint.x {
            if let Some(mut dragging) = self.right.release_dragging() {
                let p = self.right_edge_to_left_edge(ctx, dragging.get_object_mut().get_position());
                dragging.enable_small();
                dragging.get_object_mut().set_position(p);
                self.left.get_desk_mut().insert_dragging(dragging);
            }
        }
    }

    /// point: 絶対座標
    fn hand_over_check_l2r(&mut self, ctx: &mut ggez::Context, rpoint: numeric::Point2f) {
        let border = self.desk_border(ctx);
        
        if self.left.get_desk().has_dragging() && border < rpoint.x {
            if let Some(mut dragging) = self.left.get_desk_mut().release_dragging() {
                let p = self.left_edge_to_right_edge(ctx, dragging.get_object().get_position());
                dragging.enable_large();
                dragging.get_object_mut().set_position(p);
                self.right.insert_dragging(dragging);
            }
        }
    }

    fn desk_border(&mut self, ctx: &mut ggez::Context) -> f32 {
        let left_edge = self.left.canvas.get_position().x + self.left.canvas.get_texture_size(ctx).x;
        let diff = (left_edge - self.right.canvas.get_position().x).abs();
        left_edge + diff
    }

    fn right_edge_to_left_edge(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) -> numeric::Point2f {
        let from_bottom = self.right.canvas.get_texture_size(ctx).y - point.y;

        numeric::Point2f::new(self.left.get_desk().canvas.get_texture_size(ctx).x,
                              self.left.get_desk().canvas.get_texture_size(ctx).y - from_bottom)
    }

    fn left_edge_to_right_edge(&mut self, ctx: &mut ggez::Context, rpoint: numeric::Point2f) -> numeric::Point2f {
        let from_bottom = self.left.get_desk().canvas.get_texture_size(ctx).y - rpoint.y;

        numeric::Point2f::new(0.0,
                              self.right.canvas.get_texture_size(ctx).y - from_bottom)
    }

    pub fn update(&mut self, ctx: &mut ggez::Context, game_data: &GameData, t: Clock) {
        if t == 500 {
            self.left.get_desk_mut().desk_objects.add(factory::create_dobj_book_random(ctx, game_data));
        }
    }

    pub fn start_customer_event(&mut self,
                                ctx: &mut ggez::Context,
                                game_data: &GameData,
                                info: BorrowingInformation, t: Clock) {
        for _ in info.borrowing {
            self.left.get_desk_mut().add_object(factory::create_dobj_book_random(ctx, game_data));
        }
    }
}

impl DrawableComponent for TaskTable {
    
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.canvas.begin_drawing(ctx);

            self.left.draw(ctx).unwrap();
            self.right.draw(ctx).unwrap();
            
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

pub enum CustomerRequest {
    Borrowing(BorrowingInformation),
    Copying(CopyingRequestInformation),
}
