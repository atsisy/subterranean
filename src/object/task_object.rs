use ggez::graphics as ggraphics;

use torifune::graphics::object::*;
use torifune::graphics::*;
use torifune::numeric;

use crate::core::{TextureID, FontID, GameData};
use super::*;

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
        format!("第{}季 {}月 {}日", self.season, self.month, self.day)
    }
}

pub struct BorrowingInformation {
    pub borrowing: Vec<String>,
    pub borrower: String,
    pub borrow_date: GensoDate,
    pub return_date: GensoDate,
}

impl BorrowingInformation {
    pub fn new(borrowing: Vec<String>,
               borrower: String,
               borrow_date: GensoDate,
               return_date: GensoDate) -> Self {
        BorrowingInformation {
            borrowing: borrowing,
            borrower: borrower,
            borrow_date,
            return_date,
        }
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
            .map(|s| {
                pos += numeric::Vector2f::new(0.0, 30.0);
                SimpleText::new(MovableText::new(s.to_string(),
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
    pub book_title: String,
    pub customer: String,
    pub request_date: GensoDate,
    pub return_date: GensoDate,
    pub pages: u32,
}

impl CopyingRequestInformation {
    pub fn new(book_title: String,
               customer: String,
               request_date: GensoDate,
               return_date: GensoDate,
               pages: u32) -> Self {
        CopyingRequestInformation {
            book_title: book_title,
            customer: customer,
            request_date: request_date,
            return_date: return_date,
            pages: pages,
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

        let customer = SimpleText::new(MovableText::new(format!("依頼者   {}", info.customer),
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

        let request_date = SimpleText::new(MovableText::new(format!("依頼日     {}", info.request_date.to_string()),
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

        let return_date = SimpleText::new(MovableText::new(format!("完了予定   {}", info.return_date.to_string()),
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
        
        let pages = SimpleText::new(MovableText::new(format!("頁数   {}", info.pages),
                                                     numeric::Point2f::new(50.0, 500.0),
                                                     numeric::Vector2f::new(1.0, 1.0),
                                                     0.0,
                                                     0,
                                                     move_fn::halt(numeric::Point2f::new(50.0, 300.0)),
                                                     FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                                          numeric::Vector2f::new(28.0, 28.0),
                                                                          ggraphics::Color::from_rgba_u32(0x000000ff)),
                                                     t),
                                    Vec::new());
        
        let book_type = SimpleText::new(MovableText::new(format!("寸法   美濃判"),
                                                         numeric::Point2f::new(50.0, 450.0),
                                                         numeric::Vector2f::new(1.0, 1.0),
                                                         0.0,
                                                         0,
                                                         move_fn::halt(numeric::Point2f::new(50.0, 400.0)),
                                                         FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                                              numeric::Vector2f::new(28.0, 28.0),
                                                                              ggraphics::Color::from_rgba_u32(0x000000ff)),
                                                         t),
                                        Vec::new());

        let request_book = SimpleText::new(MovableText::new(format!("転写本    {}", info.book_title),
                                                            numeric::Point2f::new(50.0, 400.0),
                                                            numeric::Vector2f::new(1.0, 1.0),
                                                            0.0,
                                                            0,
                                                            move_fn::halt(numeric::Point2f::new(50.0, 550.0)),
                                                            FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                                                 numeric::Vector2f::new(28.0, 28.0),
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
