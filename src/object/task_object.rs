use std::collections::HashMap;

use ggez::graphics as ggraphics;
use ggez::input as ginput;
use ginput::mouse::MouseButton;

use torifune::graphics::object::*;
use torifune::graphics::*;
use torifune::numeric;

use torifune::hash;

use super::*;

use crate::core::{TextureID, FontID, GameData};

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

    pub fn new_random(ctx: &mut ggez::Context, rect: ggraphics::Rect, paper_tid: TextureID,
                      borrow_date: GensoDate, return_date: GensoDate,
                      game_data: &GameData, t: Clock) -> Self {

        let mut borrowing = Vec::new();

        for _ in 0..(rand::random::<u32>() % 7) {
            borrowing.push(game_data.book_random_select().get_name().to_string());
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

    pub fn new_random(game_data: &GameData,
                      request_date: GensoDate,
                      return_date: GensoDate) -> Self {
        let book_info = game_data.book_random_select();
        CopyingRequestInformation {
            book_title: book_info.get_name().to_string(),
            customer: game_data.customer_random_select().to_string(),
            request_date: request_date,
            return_date: return_date,
            pages: book_info.get_pages() as u32,
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

pub struct DrawableComponentContainer {
    container: Vec<Box<dyn DrawableComponent>>,
}

impl DrawableComponentContainer {
    pub fn new() -> Self {
        DrawableComponentContainer {
            container: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn add(&mut self, obj: Box<dyn DrawableComponent>) {
        self.container.push(obj);
    }

    #[inline(always)]
    pub fn remove_if<F>(&mut self, f: F)
    where F: Fn(&Box<dyn DrawableComponent>) -> bool {
        self.container.retain(|e| !f(e));
    }

    pub fn len(&self) -> usize {
        self.container.len()
    }

    pub fn iter(&self) -> std::slice::Iter<Box<dyn DrawableComponent>> {
        self.container.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<Box<dyn DrawableComponent>> {
        self.container.iter_mut()
    }
}

pub struct DeskObjects {
    canvas: SubScreen,
    desk_objects: SimpleObjectContainer,
    dragging: Option<tobj::SimpleObject>,
    dobj_container: DrawableComponentContainer,
    table_texture: tobj::SimpleObject,
}

impl DeskObjects {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData,
               rect: ggraphics::Rect) -> DeskObjects {

        let mut dparam = ggraphics::DrawParam::default();
        dparam.dest = numeric::Point2f::new(rect.x, rect.y).into();
        
        let mut desk_objects = SimpleObjectContainer::new();
        
        desk_objects.add(tobj::SimpleObject::new(
            tobj::MovableUniTexture::new(
                game_data.ref_texture(TextureID::Ghost1),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(0.1, 0.1),
                0.0, 0, move_fn::stop(),
                0), vec![]));
        desk_objects.add(tobj::SimpleObject::new(
            tobj::MovableUniTexture::new(
                game_data.ref_texture(TextureID::LotusPink),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(0.1, 0.1),
                0.0, -1, move_fn::stop(),
                0), vec![]));
        desk_objects.sort_with_depth();

        
        
        DeskObjects {
            canvas: SubScreen::new(ctx, rect, 0, ggraphics::Color::new(0.0, 0.0, 0.0, 0.0)),
            desk_objects: desk_objects,
            dragging: None,
            dobj_container: DrawableComponentContainer::new(),
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
            obj.move_diff(numeric::Vector2f::new(point.x - last.x, point.y - last.y));
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
            if obj.get_drawing_area(ctx).contains(rpoint) {
                dragging_object_index = self.desk_objects.len() - index - 1;
                drag_start = true;
                break;
            }
        }
        if drag_start {
            // 元々、最前面に表示されていたオブジェクトのdepthに設定する
            self.dragging = Some(self.desk_objects.get_raw_container_mut()
                                 .swap_remove(dragging_object_index));
        }
    }

    pub fn unselect_dragging_object(&mut self) {
        if let Some(obj) = &mut self.dragging {
            let min = self.desk_objects.get_minimum_depth();
            obj.set_drawing_depth(min);
            self.desk_objects.change_depth_equally(1);
        }
        match self.dragging {
            None =>  (),
            _ => {
                self.desk_objects.add(std::mem::replace(&mut self.dragging, None).unwrap());
                self.desk_objects.sort_with_depth();
            }
        }
    }

    pub fn update(&mut self, _ctx: &mut ggez::Context, t: Clock) {
        for p in self.desk_objects.get_raw_container_mut() {
            p.move_with_func(t);
        }
    }    

    pub fn double_click_handler(&mut self,
                            ctx: &mut ggez::Context,
                            point: numeric::Point2f,
                            game_data: &GameData) {
        let rpoint = self.canvas.relative_point(point);
        
        // オブジェクトは深度が深い順にソートされているので、
        // 逆順から検索していくことで、最も手前に表示されているオブジェクトを
        // 取り出すことができる
        for (_, obj) in self.desk_objects.get_raw_container_mut().iter_mut().rev().enumerate() {
            if obj.get_drawing_area(ctx).contains(rpoint) {
                println!("sassss");
                self.dobj_container.add(
                    Box::new(CopyingRequestPaper::new(ctx, ggraphics::Rect::new(0.0, 0.0, 700.0, 700.0), TextureID::Paper2,
                                                      &CopyingRequestInformation::new("テスト本1".to_string(),
                                                                                      "霧雨魔里沙".to_string(),
                                                                                      GensoDate::new(128, 12, 8),
                                                                                      GensoDate::new(128, 12, 8),
                                                                                      212),
                                                      game_data, 0))
                )
            }
        }
    }
    
}

impl DrawableComponent for DeskObjects {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.canvas.begin_drawing(ctx);

            self.table_texture.draw(ctx)?;
            
            for obj in self.desk_objects.get_raw_container_mut() {
                obj.draw(ctx)?;
            }
            
            if let Some(ref mut d) = self.dragging {
                d.draw(ctx)?;
            }

            for obj in self.dobj_container.iter_mut() {
                obj.draw(ctx)?;
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

pub enum CustomerRequest {
    Borrowing(BorrowingInformation),
    Copying(CopyingRequestInformation),
}
