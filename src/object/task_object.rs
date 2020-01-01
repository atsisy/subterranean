use ggez::graphics as ggraphics;

use torifune::graphics::object::*;
use torifune::graphics::*;
use torifune::numeric;

use std::str::FromStr;
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
    fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.canvas.begin_drawing(ctx);

            self.paper_texture.draw(ctx)?;
            self.title.draw(ctx)?;
            self.book_head.draw(ctx)?;
            self.borrower.draw(ctx)?;
            self.borrow_date.draw(ctx)?;
            self.return_date.draw(ctx)?;

            for d in &self.borrowing {
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
