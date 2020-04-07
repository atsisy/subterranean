use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

use ggez::graphics as ggraphics;

use torifune::core::Clock;
use torifune::debug;
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::graphics::object::*;
use torifune::graphics::*;
use torifune::hash;
use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;
use torifune::numeric;
use torifune::roundup2f;

use crate::core::BookInformation;
use crate::object::move_fn;
use crate::object::util_object::*;
use crate::set_table_frame_cell_center;

use super::Clickable;
use crate::core::{FontID, GameData, GensoDate, TextureID};

use super::tt_menu_component::*;

use number_to_jk::number_to_jk;

#[derive(Clone, Copy, PartialEq)]
pub enum BookStatus {
    Good = 0,
    Ok,
    Bad,
}

impl ToString for BookStatus {
    fn to_string(&self) -> String {
        match self {
            &BookStatus::Good => "良".to_string(),
            &BookStatus::Ok => "可".to_string(),
            &BookStatus::Bad => "悪".to_string(),
        }
    }
}

impl From<i32> for BookStatus {
    fn from(integer: i32) -> Self {
        match integer {
            0 => BookStatus::Good,
            1 => BookStatus::Ok,
            2 => BookStatus::Bad,
            _ => panic!("Not reserved number"),
        }
    }
}

pub struct HoldDataVText {
    pub data: HoldData,
    pub vtext: VerticalText,
}

impl HoldDataVText {
    pub fn new(
        hold_data: HoldData,
        position: numeric::Point2f,
        scale: numeric::Vector2f,
        drawing_depth: i8,
        font_info: FontInformation,
    ) -> Self {
        HoldDataVText {
            vtext: VerticalText::new(
                hold_data.to_string(),
                position,
                scale,
                0.0,
                drawing_depth,
                font_info,
            ),
            data: hold_data,
        }
    }

    pub fn reset(&mut self, hold_data: HoldData) {
        self.data = hold_data;
        self.vtext.replace_text(self.data.to_string());
    }

    pub fn copy_hold_data(&self) -> HoldData {
        self.data.clone()
    }
}

impl DrawableComponent for HoldDataVText {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.vtext.is_visible() {
            self.vtext.draw(ctx).unwrap();
        }

        Ok(())
    }

    fn hide(&mut self) {
        self.vtext.hide();
    }
    fn appear(&mut self) {
        self.vtext.appear();
    }

    fn is_visible(&self) -> bool {
        self.vtext.is_visible()
    }

    fn set_drawing_depth(&mut self, depth: i8) {
        self.vtext.set_drawing_depth(depth);
    }

    fn get_drawing_depth(&self) -> i8 {
        self.vtext.get_drawing_depth()
    }
}

impl DrawableObject for HoldDataVText {
    impl_drawable_object_for_wrapped! {vtext}
}

impl TextureObject for HoldDataVText {
    impl_texture_object_for_wrapped! {vtext}
}

#[derive(PartialEq, Clone)]
pub enum RentalLimit {
    ShortTerm = 0,
    LongTerm,
}

impl RentalLimit {
    pub fn random() -> RentalLimit {
        match rand::random::<u32>() % 2 {
            0 => RentalLimit::ShortTerm,
            1 => RentalLimit::LongTerm,
            _ => panic!("Exception"),
        }
    }
}

#[derive(Clone)]
pub struct BorrowingInformation {
    pub borrowing: Vec<BookInformation>,
    pub borrower: String,
    pub borrow_date: GensoDate,
    pub return_date: GensoDate,
    pub rental_limit: RentalLimit,
}

impl BorrowingInformation {
    pub fn new(
        borrowing: Vec<BookInformation>,
        borrower: &str,
        borrow_date: GensoDate,
        rental_limit: RentalLimit,
    ) -> Self {
        let mut return_date = borrow_date.clone();

        match rental_limit {
            RentalLimit::ShortTerm => return_date.add_day(7),
            RentalLimit::LongTerm => return_date.add_day(14),
        }

        BorrowingInformation {
            borrowing: borrowing,
            borrower: borrower.to_string(),
            borrow_date: borrow_date,
            return_date: return_date,
            rental_limit: rental_limit,
        }
    }
}

#[derive(Clone)]
pub struct ReturnBookInformation {
    pub returning: Vec<BookInformation>,
    pub borrower: String,
    pub borrow_date: GensoDate,
    pub return_date: GensoDate,
}

impl ReturnBookInformation {
    pub fn new(
        returning: Vec<BookInformation>,
        borrower: &str,
        borrow_date: GensoDate,
        return_date: GensoDate,
    ) -> Self {
        ReturnBookInformation {
            returning: returning,
            borrower: borrower.to_string(),
            borrow_date,
            return_date,
        }
    }

    pub fn new_random(
        game_data: &GameData,
        borrow_date: GensoDate,
        return_date: GensoDate,
    ) -> Self {
        let borrowing_num = rand::random::<u32>() % 5;
        let mut borrow_books = Vec::new();

        for _ in 0..borrowing_num {
            borrow_books.push(game_data.book_random_select().clone());
        }

        Self::new(
            borrow_books,
            game_data.customer_random_select(),
            borrow_date,
            return_date,
        )
    }
}

///
/// TaskSceneでクリックしたときに取得できるデータ
///
#[derive(Clone)]
pub enum HoldData {
    BookName(BookInformation),
    CustomerName(String),
    Date(GensoDate),
    BookStatus(BookStatus),
    None,
}

impl HoldData {
    pub fn to_each_type_string(&self) -> String {
        match self {
            HoldData::BookName(_) => "題目".to_string(),
            HoldData::CustomerName(_) => "御客氏名".to_string(),
            HoldData::Date(_) => "日付".to_string(),
            HoldData::BookStatus(_) => "状態".to_string(),
            HoldData::None => "".to_string(),
        }
    }
}

impl ToString for HoldData {
    fn to_string(&self) -> String {
        match self {
            HoldData::BookName(book_info) => book_info.name.to_string(),
            HoldData::CustomerName(name) => name.to_string(),
            HoldData::Date(date) => date.to_string(),
            HoldData::BookStatus(status) => status.to_string(),
            HoldData::None => "".to_string(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum OnDeskType {
    Book = 0,
    BorrowingRecordBook,
    CopyingPaper,
    Silhouette,
    Goods,
    Texture,
}

pub trait OnDesk: TextureObject + Clickable {
    fn ondesk_whose(&self) -> i32;

    fn click_data(&self, ctx: &mut ggez::Context, point: numeric::Point2f) -> HoldData;

    fn insert_data(
        &mut self,
        _: &mut ggez::Context,
        _: numeric::Point2f,
        _: &HoldData,
        _: &KosuzuMemory,
    ) -> bool {
        false
    }

    fn get_type(&self) -> OnDeskType;
}

pub struct OnDeskTexture {
    texture: UniTexture,
    on_desk_type: OnDeskType,
}

impl OnDeskTexture {
    pub fn new(obj: UniTexture, on_desk_type: OnDeskType) -> Self {
        OnDeskTexture {
            texture: obj,
            on_desk_type: on_desk_type,
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
    impl_texture_object_for_wrapped! {texture}
}

impl Clickable for OnDeskTexture {}

impl OnDesk for OnDeskTexture {
    fn ondesk_whose(&self) -> i32 {
        0
    }

    fn click_data(&self, _: &mut ggez::Context, _: numeric::Point2f) -> HoldData {
        HoldData::None
    }

    fn get_type(&self) -> OnDeskType {
        self.on_desk_type
    }
}

pub struct OnDeskBook {
    info: BookInformation,
    book_texture: UniTexture,
    title: VerticalText,
    canvas: SubScreen,
}

impl OnDeskBook {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        texture_id: TextureID,
        info: BookInformation,
    ) -> Self {
        let texture = game_data.ref_texture(texture_id);
        let book_texture = UniTexture::new(
            texture,
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(0.2, 0.2),
            0.0,
            0,
        );
        let book_area = book_texture.get_drawing_area(ctx);
        let book_title = info.get_name().to_string();

        OnDeskBook {
            info: info,
            book_texture: book_texture,
            title: VerticalText::new(
                book_title,
                numeric::Point2f::new(40.0, 30.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                FontInformation::new(
                    game_data.get_font(FontID::JpFude1),
                    numeric::Vector2f::new(18.0, 18.0),
                    ggraphics::Color::from_rgba_u32(0x000000ff),
                ),
            ),
            canvas: SubScreen::new(
                ctx,
                book_area,
                0,
                ggraphics::Color::from_rgba_u32(0x00000000),
            ),
        }
    }

    pub fn get_book_info(&self) -> &BookInformation {
        &self.info
    }
}

impl DrawableComponent for OnDeskBook {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.book_texture.draw(ctx)?;
            self.title.draw(ctx)?;

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
    impl_texture_object_for_wrapped! {canvas}
}

impl Clickable for OnDeskBook {}

impl OnDesk for OnDeskBook {
    fn ondesk_whose(&self) -> i32 {
        0
    }
    fn click_data(&self, _: &mut ggez::Context, _: numeric::Point2f) -> HoldData {
        return HoldData::BookName(self.info.clone());
    }

    fn get_type(&self) -> OnDeskType {
        OnDeskType::Book
    }
}

#[derive(Debug, Clone)]
pub struct CopyingRequestInformation {
    pub book_info: BookInformation,
    pub customer: String,
    pub request_date: GensoDate,
    pub return_date: GensoDate,
}

impl CopyingRequestInformation {
    pub fn new(
        book_info: BookInformation,
        customer: String,
        request_date: GensoDate,
        return_date: GensoDate,
    ) -> Self {
        CopyingRequestInformation {
            book_info: book_info,
            customer: customer,
            request_date: request_date,
            return_date: return_date,
        }
    }

    pub fn new_random(
        game_data: &GameData,
        request_date: GensoDate,
        return_date: GensoDate,
    ) -> Self {
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
    pub fn new(
        ctx: &mut ggez::Context,
        rect: ggraphics::Rect,
        paper_tid: TextureID,
        info: CopyingRequestInformation,
        game_data: &GameData,
        t: Clock,
    ) -> Self {
        let default_scale = numeric::Vector2f::new(1.0, 1.0);
        let mut font_info = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(16.0, 16.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );
        let mut pos = numeric::Point2f::new(rect.w - 50.0, 0.0);

        let paper_texture = SimpleObject::new(
            MovableUniTexture::new(
                game_data.ref_texture(paper_tid),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                t,
            ),
            Vec::new(),
        );

        let title_text = VerticalText::new(
            "鈴奈庵 転写依頼票".to_string(),
            numeric::Point2f::new(pos.x, 50.0),
            default_scale,
            0.0,
            0,
            font_info,
        );
        pos.x -= 60.0;

        font_info.scale = numeric::Vector2f::new(20.0, 20.0);

        let request_book = VerticalText::new(
            format!("転写本    {}", info.book_info.name),
            numeric::Point2f::new(pos.x, 50.0),
            default_scale,
            0.0,
            0,
            font_info,
        );
        pos.x -= 24.0;

        let pages = VerticalText::new(
            format!("頁数   {}", number_to_jk(info.book_info.pages as u64)),
            numeric::Point2f::new(pos.x, 50.0),
            default_scale,
            0.0,
            0,
            font_info,
        );
        pos.x -= 24.0;

        let book_type = VerticalText::new(
            format!("寸法   {}", info.book_info.size),
            numeric::Point2f::new(pos.x, 50.0),
            default_scale,
            0.0,
            0,
            font_info,
        );
        pos.x -= 180.0;

        font_info.scale = numeric::Vector2f::new(16.0, 16.0);

        let customer = VerticalText::new(
            format!("依頼者 {}", info.customer),
            numeric::Point2f::new(pos.x, 50.0),
            default_scale,
            0.0,
            0,
            font_info,
        );
        pos.x -= 24.0;

        let request_date = VerticalText::new(
            format!("依頼日 {}", info.request_date.to_string()),
            numeric::Point2f::new(pos.x, 50.0),
            default_scale,
            0.0,
            0,
            font_info,
        );
        pos.x -= 24.0;

        let return_date = VerticalText::new(
            format!("完了予定 {}", info.return_date.to_string()),
            numeric::Point2f::new(pos.x, 50.0),
            default_scale,
            0.0,
            0,
            font_info,
        );
        pos.x -= 24.0;

        let mut canvas = SubScreen::new(ctx, rect, 0, ggraphics::BLACK);
        canvas.set_filter(ggraphics::FilterMode::Nearest);

        CopyingRequestPaper {
            title: title_text,
            request_book: request_book,
            customer: customer,
            paper_texture: paper_texture,
            request_date: request_date,
            return_date: return_date,
            pages: pages,
            canvas: canvas,
            book_type: book_type,
            raw_info: info,
        }
    }
}

impl DrawableComponent for CopyingRequestPaper {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.paper_texture.draw(ctx)?;
            self.title.draw(ctx)?;
            self.customer.draw(ctx)?;
            self.request_date.draw(ctx)?;
            self.return_date.draw(ctx)?;
            self.pages.draw(ctx)?;
            self.book_type.draw(ctx)?;
            self.request_book.draw(ctx)?;

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

impl DrawableObject for CopyingRequestPaper {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for CopyingRequestPaper {
    impl_texture_object_for_wrapped! {canvas}
}

impl Clickable for CopyingRequestPaper {}

impl OnDesk for CopyingRequestPaper {
    fn ondesk_whose(&self) -> i32 {
        0
    }

    fn click_data(&self, ctx: &mut ggez::Context, point: numeric::Point2f) -> HoldData {
        let rpoint = self.canvas.relative_point(point);

        if self.request_book.get_drawing_area(ctx).contains(rpoint) {
            return HoldData::BookName(self.raw_info.book_info.clone());
        }

        if self.customer.get_drawing_area(ctx).contains(rpoint) {
            return HoldData::CustomerName(self.raw_info.customer.to_string());
        }

        HoldData::None
    }

    fn get_type(&self) -> OnDeskType {
        OnDeskType::CopyingPaper
    }
}

#[derive(Clone)]
pub struct BorrowingRecordBookPageData {
    pub borrow_book: HashMap<numeric::Vector2u, HoldData>,
    pub request_information: HashMap<numeric::Vector2u, HoldData>,
}

pub struct BorrowingRecordBookData {
    pub pages_data: Vec<BorrowingRecordBookPageData>,
}

pub struct BorrowingRecordBookPage {
    customer_info_table: TableFrame,
    books_table: TableFrame,
    borrow_book: HashMap<numeric::Vector2u, HoldDataVText>,
    request_information: HashMap<numeric::Vector2u, HoldDataVText>,
    book_head: VerticalText,
    book_status: VerticalText,
    borrower: VerticalText,
    borrow_date: VerticalText,
    return_date: VerticalText,
    paper_texture: SimpleObject,
    drwob_essential: DrawableObjectEssential,
}

impl BorrowingRecordBookPage {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        rect: ggraphics::Rect,
        paper_tid: TextureID,
        page_data: BorrowingRecordBookPageData,
        t: Clock,
    ) -> Self {
        let mut page = Self::new_empty(ctx, rect, paper_tid, game_data, t);

        for (position, hold_data) in page_data.borrow_book.iter() {
            if hold_data.is_none() {
                continue;
            }

            let info = page.borrow_book.get_mut(position).unwrap();
            info.reset(hold_data.clone());
            info.vtext.make_center(
                ctx,
                page.books_table
                    .get_center_of(*position, page.books_table.get_position()),
            );
        }

        for (position, hold_data) in page_data.request_information.iter() {
            if hold_data.is_none() {
                continue;
            }

            let info = page.request_information.get_mut(position).unwrap();
            info.reset(hold_data.clone());
            info.vtext.make_center(
                ctx,
                page.customer_info_table
                    .get_center_of(*position, page.customer_info_table.get_position()),
            );
        }

        page
    }

    pub fn new_empty(
        ctx: &mut ggez::Context,
        rect: ggraphics::Rect,
        paper_tid: TextureID,
        game_data: &GameData,
        t: Clock,
    ) -> Self {
        let table_frame = TableFrame::new(
            game_data,
            numeric::Point2f::new(rect.w - 200.0, 40.0),
            FrameData::new(vec![150.0, 300.0], vec![40.0; 3]),
            numeric::Vector2f::new(0.3, 0.3),
            0,
        );

        let mut borrower = VerticalText::new(
            "借りた人".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                game_data.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );

        set_table_frame_cell_center!(ctx, table_frame, borrower, numeric::Vector2u::new(2, 0));

        let mut borrow_date = VerticalText::new(
            "貸出日".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                game_data.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );

        set_table_frame_cell_center!(ctx, table_frame, borrow_date, numeric::Vector2u::new(1, 0));

        let mut return_date = VerticalText::new(
            "返却期限".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                game_data.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );
 
        set_table_frame_cell_center!(ctx, table_frame, return_date, numeric::Vector2u::new(0, 0));

        let books_table = TableFrame::new(
            game_data,
            numeric::Point2f::new(rect.w - 550.0, 30.0),
            FrameData::new(vec![380.0, 70.0], vec![40.0; 6]),
            numeric::Vector2f::new(0.3, 0.3),
            0,
        );

        let mut book_head = VerticalText::new(
            "貸出本名称".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                game_data.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );

        set_table_frame_cell_center!(ctx, books_table, book_head, numeric::Vector2u::new(5, 0));

        let mut book_status = VerticalText::new(
            "状態".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                game_data.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );

        set_table_frame_cell_center!(ctx, books_table, book_status, numeric::Vector2u::new(5, 1));

        let paper_texture = SimpleObject::new(
            MovableUniTexture::new(
                game_data.ref_texture(paper_tid),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                t,
            ),
            Vec::new(),
        );

        let info_font = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(24.0, 24.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );
        let request_info_text = hash![
            (
                numeric::Vector2u::new(0, 1),
                HoldDataVText::new(
                    HoldData::None,
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0,
                    info_font.clone()
                )
            ),
            (
                numeric::Vector2u::new(1, 1),
                HoldDataVText::new(
                    HoldData::None,
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0,
                    info_font.clone()
                )
            ),
            (
                numeric::Vector2u::new(2, 1),
                HoldDataVText::new(
                    HoldData::None,
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0,
                    info_font.clone()
                )
            )
        ];

        let borrow_text = hash![
            (
                numeric::Vector2u::new(0, 0),
                HoldDataVText::new(
                    HoldData::None,
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0,
                    info_font.clone()
                )
            ),
            (
                numeric::Vector2u::new(1, 0),
                HoldDataVText::new(
                    HoldData::None,
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0,
                    info_font.clone()
                )
            ),
            (
                numeric::Vector2u::new(2, 0),
                HoldDataVText::new(
                    HoldData::None,
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0,
                    info_font.clone()
                )
            ),
            (
                numeric::Vector2u::new(3, 0),
                HoldDataVText::new(
                    HoldData::None,
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0,
                    info_font.clone()
                )
            ),
            (
                numeric::Vector2u::new(4, 0),
                HoldDataVText::new(
                    HoldData::None,
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0,
                    info_font.clone()
                )
            ),
            (
                numeric::Vector2u::new(5, 0),
                HoldDataVText::new(
                    HoldData::None,
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0,
                    info_font.clone()
                )
            ),
            (
                numeric::Vector2u::new(0, 1),
                HoldDataVText::new(
                    HoldData::None,
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0,
                    info_font.clone()
                )
            ),
            (
                numeric::Vector2u::new(1, 1),
                HoldDataVText::new(
                    HoldData::None,
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0,
                    info_font.clone()
                )
            ),
            (
                numeric::Vector2u::new(2, 1),
                HoldDataVText::new(
                    HoldData::None,
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0,
                    info_font.clone()
                )
            ),
            (
                numeric::Vector2u::new(3, 1),
                HoldDataVText::new(
                    HoldData::None,
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0,
                    info_font.clone()
                )
            ),
            (
                numeric::Vector2u::new(4, 1),
                HoldDataVText::new(
                    HoldData::None,
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0,
                    info_font.clone()
                )
            ),
            (
                numeric::Vector2u::new(5, 1),
                HoldDataVText::new(
                    HoldData::None,
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0,
                    info_font.clone()
                )
            )
        ];

        BorrowingRecordBookPage {
            customer_info_table: table_frame,
            books_table: books_table,
            borrow_book: borrow_text,
            borrower: borrower,
            request_information: request_info_text,
            book_head: book_head,
            book_status: book_status,
            paper_texture: paper_texture,
            borrow_date: borrow_date,
            return_date: return_date,
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }

    pub fn try_insert_data_in_info_frame(
        &mut self,
        ctx: &mut ggez::Context,
        position: numeric::Vector2u,
        hold_data: &HoldData,
    ) {
        if position == numeric::Vector2u::new(2, 1) {
            match hold_data {
                HoldData::CustomerName(_) => {
                    let info = self.request_information.get_mut(&position).unwrap();
                    info.reset(hold_data.clone());
                    info.vtext.make_center(
                        ctx,
                        self.customer_info_table
                            .get_center_of(position, self.customer_info_table.get_position()),
                    );
                }
                _ => (),
            }
        } else if position == numeric::Vector2u::new(1, 1) {
            match hold_data {
                HoldData::Date(_) => {
                    let info = self.request_information.get_mut(&position).unwrap();
                    info.reset(hold_data.clone());
                    info.vtext.make_center(
                        ctx,
                        self.customer_info_table
                            .get_center_of(position, self.customer_info_table.get_position()),
                    );
                }
                _ => (),
            }
        } else if position == numeric::Vector2u::new(0, 1) {
            match hold_data {
                HoldData::Date(_) => {
                    let info = self.request_information.get_mut(&position).unwrap();
                    info.reset(hold_data.clone());
                    info.vtext.make_center(
                        ctx,
                        self.customer_info_table
                            .get_center_of(position, self.customer_info_table.get_position()),
                    );
                }
                _ => (),
            }
        }
    }

    ///
    /// Dataを格納できればtrue, できなければfalse
    ///
    pub fn try_insert_data_in_borrowing_books_frame(
        &mut self,
        ctx: &mut ggez::Context,
        menu_position: numeric::Point2f,
        book_info: BookInformation,
    ) {
        let grid_pos = self
            .books_table
            .get_grid_position(ctx, menu_position)
            .unwrap();

        // 本の状態は、このメソッドからは設定できない
        if grid_pos.y == 1 {
            return;
        }

        let info = self.borrow_book.get_mut(&grid_pos).unwrap();
        info.reset(HoldData::BookName(book_info));
        info.vtext.make_center(
            ctx,
            self.books_table
                .get_center_of(grid_pos, self.books_table.get_position()),
        );
    }

    ///
    /// Dataを格納できればtrue, できなければfalse
    ///
    pub fn try_insert_date_data_in_cutomer_info_frame(
        &mut self,
        ctx: &mut ggez::Context,
        menu_position: numeric::Point2f,
        date: GensoDate,
    ) {
        let grid_pos = self
            .customer_info_table
            .get_grid_position(ctx, menu_position)
            .unwrap();

        // 挿入先が日時のエントリではない
        if !(grid_pos == numeric::Vector2u::new(0, 1) || grid_pos == numeric::Vector2u::new(1, 1)) {
            return;
        }

        let info = self.request_information.get_mut(&grid_pos).unwrap();
        info.reset(HoldData::Date(date));
        info.vtext.make_center(
            ctx,
            self.customer_info_table
                .get_center_of(grid_pos, self.customer_info_table.get_position()),
        );
    }

    ///
    /// CustomerNameを明示的に設定するメソッド, 格納できればtrue, できなければfalse
    ///
    pub fn try_insert_customer_name_in_cutomer_info_frame(
        &mut self,
        ctx: &mut ggez::Context,
        menu_position: numeric::Point2f,
        customer_name: String,
    ) {
        let grid_pos = self
            .customer_info_table
            .get_grid_position(ctx, menu_position)
            .unwrap();

        // 挿入先が日時のエントリではない
        if grid_pos != numeric::Vector2u::new(2, 1) {
            return;
        }

        let info = self.request_information.get_mut(&grid_pos).unwrap();
        info.reset(HoldData::CustomerName(customer_name));
        info.vtext.make_center(
            ctx,
            self.customer_info_table
                .get_center_of(grid_pos, self.customer_info_table.get_position()),
        );
    }

    pub fn replace_borrower_name(&mut self, game_data: &GameData, name: &str) -> &mut Self {
        let pos = self.borrower.get_position();
        self.borrower = VerticalText::new(
            format!("借りた人   {}", name),
            pos,
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                game_data.get_font(FontID::JpFude1),
                numeric::Vector2f::new(20.0, 20.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );
        self
    }

    fn insert_book_status_data(
        &mut self,
        ctx: &mut ggez::Context,
        status_index: i32,
        menu_position: numeric::Point2f,
    ) {
        println!("book status insert!! data -> {}", status_index);
        let grid_position = self
            .books_table
            .get_grid_position(ctx, menu_position)
            .unwrap();
        let info = self.borrow_book.get_mut(&grid_position).unwrap();
        info.reset(HoldData::BookStatus(BookStatus::from(status_index)));
        info.vtext.make_center(
            ctx,
            self.books_table
                .get_center_of(grid_position, self.books_table.get_position()),
        );
    }

    pub fn export_page_data(&self) -> BorrowingRecordBookPageData {
        let mut borrow_book = HashMap::new();
        let mut request_information = HashMap::new();

        for (p, hold_data_vtext) in &self.borrow_book {
            borrow_book.insert(p.clone(), hold_data_vtext.copy_hold_data());
        }

        for (p, hold_data_vtext) in &self.request_information {
            request_information.insert(p.clone(), hold_data_vtext.copy_hold_data());
        }

        BorrowingRecordBookPageData {
            borrow_book: borrow_book,
            request_information: request_information,
        }
    }
}

impl DrawableComponent for BorrowingRecordBookPage {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.paper_texture.draw(ctx)?;
            self.customer_info_table.draw(ctx)?;
            self.books_table.draw(ctx)?;
            self.book_head.draw(ctx)?;
            self.book_status.draw(ctx)?;
            self.borrower.draw(ctx)?;
            self.borrow_date.draw(ctx)?;
            self.return_date.draw(ctx)?;

            for (_, d) in &mut self.borrow_book {
                d.draw(ctx)?;
            }

            for (_, d) in &mut self.request_information {
                d.draw(ctx)?;
            }
        }

        Ok(())
    }

    fn hide(&mut self) {
        self.drwob_essential.visible = false;
    }

    fn appear(&mut self) {
        self.drwob_essential.visible = true;
    }

    fn is_visible(&self) -> bool {
        self.drwob_essential.visible
    }

    fn set_drawing_depth(&mut self, depth: i8) {
        self.drwob_essential.drawing_depth = depth;
    }

    fn get_drawing_depth(&self) -> i8 {
        self.drwob_essential.drawing_depth
    }
}

pub struct BorrowingRecordBook {
    pages: Vec<BorrowingRecordBookPage>,
    rect: numeric::Rect,
    current_page: usize,
    canvas: MovableWrap<SubScreen>,
}

impl BorrowingRecordBook {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        rect: ggraphics::Rect,
        drawing_depth: i8,
        mut maybe_book_data: Option<BorrowingRecordBookData>,
        t: Clock,
    ) -> Self {
        let pages = if let Some(book_data) = maybe_book_data.as_mut() {
            let mut pages = Vec::new();

            while !book_data.pages_data.is_empty() {
                let page_data = book_data.pages_data.remove(0);
                pages.push(BorrowingRecordBookPage::new(
                    ctx,
                    game_data,
                    rect,
                    TextureID::Paper1,
                    page_data,
                    t,
                ));
            }

            pages
        } else {
            Vec::new()
        };

        BorrowingRecordBook {
            pages: pages,
            rect: rect,
            current_page: 0,
            canvas: MovableWrap::new(
                Box::new(SubScreen::new(
                    ctx,
                    rect,
                    drawing_depth,
                    ggraphics::Color::from_rgba_u32(0xffffffff),
                )),
                None,
                0,
            ),
        }
    }

    pub fn add_empty_page(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        t: Clock,
    ) -> &Self {
        self.pages.push(BorrowingRecordBookPage::new_empty(
            ctx,
            self.rect,
            TextureID::Paper1,
            game_data,
            t,
        ));
        self
    }

    fn get_current_page(&self) -> Option<&BorrowingRecordBookPage> {
        self.pages.get(self.current_page)
    }

    fn get_current_page_mut(&mut self) -> Option<&mut BorrowingRecordBookPage> {
        self.pages.get_mut(self.current_page)
    }

    pub fn relative_point(&self, point: numeric::Point2f) -> numeric::Point2f {
        self.canvas.ref_wrapped_object().relative_point(point)
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

    fn try_insert_data_customer_info_frame(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
        hold_data: &HoldData,
    ) -> bool {
        let rpoint = self.relative_point(point);
        if let Some(page) = self.get_current_page_mut() {
            let maybe_position = page.customer_info_table.get_grid_position(ctx, rpoint);
            if let Some(position) = maybe_position {
                page.try_insert_data_in_info_frame(ctx, position, hold_data);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn insert_book_title_to_books_frame(
        &mut self,
        ctx: &mut ggez::Context,
        menu_position: numeric::Point2f,
        book_info: BookInformation,
    ) {
        let rpoint = self.relative_point(menu_position);
        if let Some(page) = self.get_current_page_mut() {
            page.try_insert_data_in_borrowing_books_frame(ctx, rpoint, book_info);
        }
    }

    pub fn insert_date_data_to_customer_info(
        &mut self,
        ctx: &mut ggez::Context,
        menu_position: numeric::Point2f,
        date: GensoDate,
    ) {
        let rpoint = self.relative_point(menu_position);
        if let Some(page) = self.get_current_page_mut() {
            page.try_insert_date_data_in_cutomer_info_frame(ctx, rpoint, date);
        }
    }

    pub fn insert_customer_name_data_to_customer_info(
        &mut self,
        ctx: &mut ggez::Context,
        menu_position: numeric::Point2f,
        customer_name: String,
    ) {
        let rpoint = self.relative_point(menu_position);
        if let Some(page) = self.get_current_page_mut() {
            page.try_insert_customer_name_in_cutomer_info_frame(ctx, rpoint, customer_name);
        }
    }

    pub fn export_book_data(&self) -> BorrowingRecordBookData {
        BorrowingRecordBookData {
            pages_data: self
                .pages
                .iter()
                .map(|page| page.export_page_data())
                .collect(),
        }
    }

    pub fn click_handler(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        t: Clock,
        point: numeric::Point2f,
    ) {
        let rpoint = self.relative_point(point);
        let width = self.get_drawing_size(ctx).x;

        debug::debug_screen_push_text("book click!!");
        if rpoint.x < 20.0 && rpoint.x >= 0.0 {
            println!("next page!!");
            self.add_empty_page(ctx, game_data, t);
            self.next_page();
        } else if rpoint.x > width - 20.0 && rpoint.x <= width {
            println!("prev page!!");
            self.prev_page();
        }
    }

    pub fn pages_length(&self) -> usize {
        self.pages.len()
    }

    pub fn update(&mut self, t: Clock) {
        self.move_with_func(t);
    }

    pub fn get_book_info_frame_grid_position(
        &self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> Option<numeric::Vector2u> {
        if let Some(page) = self.get_current_page().as_ref() {
            let page_point = self.relative_point(point);
            page.books_table.get_grid_position(ctx, page_point)
        } else {
            None
        }
    }

    pub fn get_customer_info_frame_grid_position(
        &self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> Option<numeric::Vector2u> {
        if let Some(page) = self.get_current_page().as_ref() {
            let page_point = self.relative_point(point);
            page.customer_info_table.get_grid_position(ctx, page_point)
        } else {
            None
        }
    }

    pub fn insert_book_status_via_choice(
        &mut self,
        ctx: &mut ggez::Context,
        status_index: usize,
        menu_position: numeric::Point2f,
    ) {
        let rpoint = self.relative_point(menu_position);
        if let Some(page) = self.get_current_page_mut() {
            page.insert_book_status_data(ctx, status_index as i32, rpoint);
        }
    }
}

impl DrawableComponent for BorrowingRecordBook {
    #[inline(always)]
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, self.canvas.ref_wrapped_object());

            if self.pages.len() > 0 {
                self.pages.get_mut(self.current_page).unwrap().draw(ctx)?;
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
    fn virtual_key_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _event_type: torifune::device::KeyboardEvent,
        _vkey: torifune::device::VirtualKey,
    ) {
        // Nothing
    }
    fn mouse_button_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _event_type: torifune::device::MouseButtonEvent,
        _button: ggez::event::MouseButton,
        _point: numeric::Point2f,
    ) {
        // Nothing
    }
}

impl DrawableObject for BorrowingRecordBook {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for BorrowingRecordBook {
    impl_texture_object_for_wrapped! {canvas}
}

impl HasBirthTime for BorrowingRecordBook {
    fn get_birth_time(&self) -> Clock {
        self.canvas.get_birth_time()
    }
}

impl MovableObject for BorrowingRecordBook {
    fn move_with_func(&mut self, t: Clock) {
        self.canvas.move_with_func(t);
    }

    fn override_move_func(
        &mut self,
        move_fn: Option<Box<dyn Fn(&dyn MovableObject, Clock) -> numeric::Point2f>>,
        now: Clock,
    ) {
        self.canvas.override_move_func(move_fn, now);
    }

    fn mf_start_timing(&self) -> Clock {
        self.canvas.mf_start_timing()
    }

    fn is_stop(&self) -> bool {
        self.canvas.is_stop()
    }
}

impl Clickable for BorrowingRecordBook {}

impl OnDesk for BorrowingRecordBook {
    fn ondesk_whose(&self) -> i32 {
        0
    }

    fn click_data(&self, _: &mut ggez::Context, _: numeric::Point2f) -> HoldData {
        HoldData::None
    }

    fn insert_data(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
        data: &HoldData,
        _: &KosuzuMemory,
    ) -> bool {
        // いずれかのTableFrameにデータを挿入できた場合trueが返る
        self.try_insert_data_customer_info_frame(ctx, point, data)
    }

    fn get_type(&self) -> OnDeskType {
        OnDeskType::BorrowingRecordBook
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
    pub fn new(
        small: Box<dyn OnDesk>,
        large: Box<dyn OnDesk>,
        switch: u8,
        obj_type: DeskObjectType,
        t: Clock,
    ) -> Self {
        DeskObject {
            small: Box::new(EffectableWrap::new(
                MovableWrap::new(small, None, t),
                Vec::new(),
            )),
            large: Box::new(EffectableWrap::new(
                MovableWrap::new(large, None, t),
                Vec::new(),
            )),
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
    pub fn new() -> Self {
        DeskObjectContainer {
            container: Vec::new(),
        }
    }

    pub fn add(&mut self, obj: DeskObject) {
        self.container.push(obj);
    }

    pub fn sort_with_depth(&mut self) {
        self.container.sort_by(|a: &DeskObject, b: &DeskObject| {
            let (ad, bd) = (
                a.get_object().get_drawing_depth(),
                b.get_object().get_drawing_depth(),
            );
            if ad > bd {
                Ordering::Less
            } else if ad < bd {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });
    }

    pub fn get_raw_container(&self) -> &Vec<DeskObject> {
        &self.container
    }

    pub fn get_raw_container_mut(&mut self) -> &mut Vec<DeskObject> {
        &mut self.container
    }

    pub fn get_minimum_depth(&mut self) -> i8 {
        self.sort_with_depth();
        if let Some(depth) = self.container.last() {
            depth.get_object().get_drawing_depth()
        } else {
            127
        }
    }

    pub fn len(&self) -> usize {
        self.container.len()
    }

    pub fn change_depth_equally(&mut self, offset: i8) {
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

pub struct DeskBookMenu {
    book_info: BookInformation,
    select_table_frame: TableFrame,
    choice_element_text: Vec<VerticalText>,
    drwob_essential: DrawableObjectEssential,
    last_clicked: Option<usize>,
}

impl DeskBookMenu {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        book_info: BookInformation,
        mut choice_text_str: Vec<String>,
        drawing_depth: i8,
    ) -> Self {
        let mut choice_vtext = Vec::new();

        let font_info = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(32.0, 32.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let select_table_frame = TableFrame::new(
            game_data,
            numeric::Point2f::new(10.0, 10.0),
            FrameData::new(vec![200.0], vec![64.0; 2]),
            numeric::Vector2f::new(0.3, 0.3),
            0,
        );

        while choice_text_str.len() > 0 {
            let choice_str_element = choice_text_str.swap_remove(0);
            let mut vtext = VerticalText::new(
                choice_str_element,
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                drawing_depth,
                font_info,
            );

            set_table_frame_cell_center!(
                ctx,
                select_table_frame,
                vtext,
                numeric::Vector2u::new(choice_text_str.len() as u32, 0)
            );

            choice_vtext.push(vtext);
        }

        DeskBookMenu {
            book_info: book_info,
            select_table_frame: select_table_frame,
            choice_element_text: choice_vtext,
            drwob_essential: DrawableObjectEssential::new(true, drawing_depth),
            last_clicked: None,
        }
    }

    pub fn click_handler(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        let maybe_grid_position = self.select_table_frame.get_grid_position(ctx, point);
        if let Some(grid_position) = maybe_grid_position {
            self.last_clicked = Some(grid_position.x as usize);
        }
    }

    pub fn get_last_clicked(&self) -> Option<usize> {
        self.last_clicked
    }

    pub fn get_target_book_info(&self) -> BookInformation {
        self.book_info.clone()
    }
}

impl DrawableComponent for DeskBookMenu {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.select_table_frame.draw(ctx)?;

            for vtext in &mut self.choice_element_text {
                vtext.draw(ctx)?;
            }
        }
        Ok(())
    }

    fn hide(&mut self) {
        self.drwob_essential.visible = false;
    }

    fn appear(&mut self) {
        self.drwob_essential.visible = true;
    }

    fn is_visible(&self) -> bool {
        self.drwob_essential.visible
    }

    fn set_drawing_depth(&mut self, depth: i8) {
        self.drwob_essential.drawing_depth = depth;
    }

    fn get_drawing_depth(&self) -> i8 {
        self.drwob_essential.drawing_depth
    }
}

impl Clickable for DeskBookMenu {
    fn on_click(
        &mut self,
        ctx: &mut ggez::Context,
        _game_data: &GameData,
        _t: Clock,
        _button: ggez::event::MouseButton,
        point: numeric::Point2f,
    ) {
        self.click_handler(ctx, point);
        if let Some(menu_id) = self.last_clicked.as_ref() {
            println!("clicked menu, {}", menu_id);
        }
    }
}

pub type DeskBookDropMenu = DropDownArea<DeskBookMenu>;
