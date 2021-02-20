use std::cmp::Ordering;
use std::collections::HashMap;

use ggez::graphics as ggraphics;

use torifune::core::Clock;
use torifune::graphics::drawable::*;
use torifune::graphics::object::shadow::*;
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::graphics::object::*;
use torifune::hash;
use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;
use torifune::numeric;
use torifune::{mintp_new, mintp, roundup2f};

use crate::{core::{BookInformation, RentalLimit, TileBatchTextureID}, flush_delay_event_and_redraw_check, flush_delay_event, scene::DelayEventList};
use crate::object::move_fn;
use crate::object::util_object::*;
use crate::set_table_frame_cell_center;

use serde::{Deserialize, Serialize};

use super::Clickable;
use crate::core::*;
use crate::scene::DrawRequest;

use number_to_jk::number_to_jk;

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

    pub fn ref_hold_data(&self) -> &HoldData {
        &self.data
    }

    pub fn is_none(&self) -> bool {
        self.data == HoldData::None
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

///
/// TaskSceneでクリックしたときに取得できるデータ
///
#[derive(Clone, PartialEq)]
pub enum HoldData {
    BookName(BookInformation),
    CustomerName(String),
    Date(GensoDate),
    BookCondition(BookCondition),
    None,
}

impl HoldData {
    pub fn to_each_type_string(&self) -> String {
        match self {
            HoldData::BookName(_) => "題目".to_string(),
            HoldData::CustomerName(_) => "御客氏名".to_string(),
            HoldData::Date(_) => "日付".to_string(),
            HoldData::BookCondition(_) => "状態".to_string(),
            HoldData::None => "".to_string(),
        }
    }

    pub fn to_book_name(&self) -> Option<BookInformation> {
        match self {
            HoldData::BookName(info) => Some(info.clone()),
            _ => None,
        }
    }

    pub fn to_customer_name(&self) -> Option<String> {
        match self {
            HoldData::CustomerName(name) => Some(name.clone()),
            _ => None,
        }
    }

    pub fn to_date(&self) -> Option<GensoDate> {
        match self {
            HoldData::Date(date) => Some(date.clone()),
            _ => None,
        }
    }

    pub fn to_book_status(&self) -> Option<BookCondition> {
        match self {
            HoldData::BookCondition(status) => Some(status.clone()),
            _ => None,
        }
    }
}

impl ToString for HoldData {
    fn to_string(&self) -> String {
        match self {
            HoldData::BookName(book_info) => book_info.name.to_string(),
            HoldData::CustomerName(name) => name.to_string(),
            HoldData::Date(date) => date.to_string(),
            HoldData::BookCondition(status) => status.to_string(),
            HoldData::None => "".to_string(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum OnDeskType {
    Book = 0,
    BorrowingRecordBook,
    ManualBook,
    CopyingPaper,
    Silhouette,
    Goods,
    Texture,
    Coin,
}

pub trait OnDesk: TextureObject + Clickable {
    fn ondesk_whose(&self) -> i32;

    fn click_hold_data(&self, ctx: &mut ggez::Context, point: numeric::Point2f) -> HoldData;

    fn get_hold_data(&self) -> HoldData {
        HoldData::None
    }

    fn get_type(&self) -> OnDeskType;

    fn start_dragging<'a>(&mut self, _: &mut SuzuContext<'a>) {}

    fn finish_dragging<'a>(&mut self, _: &mut SuzuContext<'a>) {}
}

pub struct OnDeskTexture {
    texture: UniTexture,
    shadow: ShadowShape,
    on_desk_type: OnDeskType,
    canvas: SubScreen,
}

impl OnDeskTexture {
    pub fn new(ctx: &mut ggez::Context, mut obj: UniTexture, on_desk_type: OnDeskType) -> Self {
        let area = obj.get_drawing_area(ctx);
        obj.set_position(numeric::Point2f::new(6.0, 6.0));
        let shadow_bounds = numeric::Rect::new(area.x, area.y, area.w + 12.0, area.h + 12.0);
        let mut shadow = ShadowShape::new(
            ctx,
            12.0,
            shadow_bounds,
            ggraphics::Color::from_rgba_u32(0xbb),
            0,
        );

        shadow.hide();

        let canvas = SubScreen::new(ctx, shadow_bounds, 0, ggraphics::Color::from_rgba_u32(0));

        OnDeskTexture {
            texture: obj,
            shadow: shadow,
            on_desk_type: on_desk_type,
            canvas: canvas,
        }
    }

    pub fn disable_shadow(&mut self) {
        self.shadow.hide();
    }

    pub fn enable_shadow(&mut self) {
        self.shadow.appear();
    }
}

impl DrawableComponent for OnDeskTexture {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.shadow.draw(ctx)?;
            self.texture.draw(ctx)?;

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

impl DrawableObject for OnDeskTexture {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for OnDeskTexture {
    impl_texture_object_for_wrapped! {canvas}
}

impl Clickable for OnDeskTexture {}

impl OnDesk for OnDeskTexture {
    fn ondesk_whose(&self) -> i32 {
        0
    }

    fn click_hold_data(&self, _: &mut ggez::Context, _: numeric::Point2f) -> HoldData {
        HoldData::None
    }

    fn get_type(&self) -> OnDeskType {
        self.on_desk_type
    }

    fn start_dragging<'a>(&mut self, _: &mut SuzuContext<'a>) {
        self.enable_shadow();
    }

    fn finish_dragging<'a>(&mut self, _: &mut SuzuContext<'a>) {
        self.disable_shadow();
    }
}

pub struct OnDeskBook {
    info: BookInformation,
    book_texture: UniTexture,
    scratch_texture: Option<UniTexture>,
    shadow: ShadowShape,
    title: VerticalText,
    canvas: SubScreen,
}

impl OnDeskBook {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        position: numeric::Point2f,
        texture_id: TextureID,
        info: BookInformation,
    ) -> Self {
        let texture_scale = match info.size.as_str() {
            "大判本" => numeric::Vector2f::new(0.16, 0.16),
            "中判本" => numeric::Vector2f::new(0.2, 0.2),
            _ => panic!("invalid book size info"),
        };
        let texture = ctx.ref_texture(texture_id);
        let book_texture = UniTexture::new(
            texture,
            numeric::Point2f::new(6.0, 6.0),
            texture_scale,
            0.0,
            0,
        );
        let book_size = book_texture.get_drawing_size(ctx.context);
        let book_title = info.get_name().to_string();

        let scratch_texture = match info.get_condition() {
            BookCondition::Good => None,
            BookCondition::Fair => Some(UniTexture::new(
                ctx.ref_texture(TextureID::random_large_book_scratch_fair()),
                numeric::Point2f::new(6.0, 6.0),
                numeric::Vector2f::new(0.16, 0.16),
                0.0,
                0,
            )),
            BookCondition::Bad => Some(UniTexture::new(
                ctx.ref_texture(TextureID::random_large_book_scratch_bad()),
                numeric::Point2f::new(6.0, 6.0),
                numeric::Vector2f::new(0.16, 0.16),
                0.0,
                0,
            )),
        };

        let shadow_bounds = numeric::Rect::new(0.0, 0.0, book_size.x + 12.0, book_size.y + 12.0);

        let mut shadow = ShadowShape::new(
            ctx.context,
            12.0,
            shadow_bounds,
            ggraphics::Color::from_rgba_u32(0xbb),
            0,
        );
        shadow.hide();

        let canvas = SubScreen::new(
            ctx.context,
            numeric::Rect::new(position.x, position.y, shadow_bounds.w, shadow_bounds.h),
            0,
            ggraphics::Color::from_rgba_u32(0x00000000),
        );

        let (title_center, title_size) = match info.size.as_str() {
            "大判本" => (
                numeric::Point2f::new(43.0, 66.0),
                numeric::Vector2f::new(12.0, 12.0),
            ),
            "中判本" => (
                numeric::Point2f::new(38.0, 64.0),
                numeric::Vector2f::new(12.0, 12.0),
            ),
            _ => panic!("invalid book size info"),
        };

	let mut title_vtext = VerticalText::new(
            book_title,
            title_center,
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                ctx.resource.get_font(FontID::BitMap1),
                title_size,
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );

	title_vtext.make_center(ctx.context, title_center);
	
        OnDeskBook {
            info: info,
            book_texture: book_texture,
            scratch_texture: scratch_texture,
            shadow: shadow,
            title: title_vtext,
            canvas: canvas,
        }
    }

    pub fn get_book_info(&self) -> &BookInformation {
        &self.info
    }

    pub fn disable_shadow(&mut self) {
        self.shadow.hide();
    }

    pub fn enable_shadow(&mut self) {
        self.shadow.appear();
    }
}

impl DrawableComponent for OnDeskBook {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.shadow.draw(ctx)?;

            self.book_texture.draw(ctx)?;
            self.title.draw(ctx)?;

            if let Some(scratch_texture) = self.scratch_texture.as_mut() {
                scratch_texture.draw(ctx)?;
            }

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

    fn click_hold_data(&self, _: &mut ggez::Context, _: numeric::Point2f) -> HoldData {
        return HoldData::BookName(self.info.clone());
    }

    fn get_hold_data(&self) -> HoldData {
        HoldData::BookName(self.info.clone())
    }

    fn get_type(&self) -> OnDeskType {
        OnDeskType::Book
    }

    fn start_dragging<'a>(&mut self, _: &mut SuzuContext<'a>) {
        self.enable_shadow();
    }

    fn finish_dragging<'a>(&mut self, _: &mut SuzuContext<'a>) {
        self.disable_shadow();
    }
}

#[derive(Clone)]
pub struct BookConditionEvalReport {
    originals: Vec<BookInformation>,
    each_evaluation: Vec<BookCondition>,
}

impl BookConditionEvalReport {
    pub fn new(originals: Vec<BookInformation>, each_evaluation: Vec<BookCondition>) -> Self {
        BookConditionEvalReport {
            originals: originals,
            each_evaluation: each_evaluation,
        }
    }

    pub fn count_mistake(&self) -> usize {
        let mut count: usize = 0;

        for index in 0..self.originals.len() {
            let original = self.originals.get(index).as_ref().unwrap().get_condition();
            let eval = self.each_evaluation.get(index).as_ref().unwrap().clone();

            if original != *eval {
                count += 1;
            }
        }

        count
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowingRecordBookPageData {
    pub borrowing_book_title: Vec<(numeric::Vector2u, BookInformation)>,
    pub borrowing_book_status: Vec<(numeric::Vector2u, BookCondition)>,
    pub customer_name: Option<String>,
    pub return_date: Option<GensoDate>,
    pub rental_date: Option<GensoDate>,
    pub rental_limit: Option<RentalLimit>,
    pub borrowing_is_signed: bool,
    pub returning_is_signed: bool,
}

impl BorrowingRecordBookPageData {
    pub fn is_maybe_waiting_returning(&self) -> bool {
        !self.returning_is_signed
            && self.borrowing_is_signed
            && !self.borrowing_book_title.is_empty()
            && self.borrowing_book_status.is_empty()
            && self.customer_name.is_some()
            && self.return_date.is_some()
            && self.rental_date.is_some()
            && self.rental_limit.is_some()
    }

    pub fn generate_return_book_information(&self) -> Option<ReturnBookInformation> {
        if !self.is_maybe_waiting_returning() {
            return None;
        }

        return Some(ReturnBookInformation::new(
            self.borrowing_book_title
                .iter()
                .map(|elem| elem.1.clone())
                .collect(),
            self.customer_name.as_ref().unwrap(),
            self.rental_date.as_ref().unwrap().clone(),
            self.return_date.as_ref().unwrap().clone(),
        ));
    }
}

impl From<&ReturnBookInformation> for BorrowingRecordBookPageData {
    fn from(info: &ReturnBookInformation) -> Self {
        let mut borrowing_book_title = Vec::new();

        for (index, book_info) in info.returning.iter().enumerate() {
            borrowing_book_title.push((
                numeric::Vector2u::new((4 - index) as u32, 0),
                book_info.clone(),
            ));
        }

        BorrowingRecordBookPageData {
            borrowing_book_title: borrowing_book_title,
            borrowing_book_status: Vec::new(),
            customer_name: Some(info.borrower.clone()),
            rental_limit: info.borrow_date.rental_limit_type(&info.return_date),
            return_date: Some(info.return_date),
            rental_date: Some(info.borrow_date),
            borrowing_is_signed: true,
            returning_is_signed: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowingRecordBookData {
    pub pages_data: Vec<BorrowingRecordBookPageData>,
}

impl BorrowingRecordBookData {
    pub fn from_returning_request_pool(pool: ReturningRequestPool) -> Self {
        BorrowingRecordBookData {
            pages_data: pool
                .iter()
                .map(|request| BorrowingRecordBookPageData::from(request))
                .collect(),
        }
    }

    pub fn has_returning_request(&self) -> bool {
        let count = self
            .pages_data
            .iter()
            .map(|data| {
                if data.is_maybe_waiting_returning() {
                    1
                } else {
                    0
                }
            })
            .fold(0, |m, v| m + v);

        count > 0
    }

    pub fn pick_returning_request_up(&self) -> Option<ReturnBookInformation> {
        let count = self
            .pages_data
            .iter()
            .map(|data| {
                if data.is_maybe_waiting_returning() {
                    1
                } else {
                    0
                }
            })
            .fold(0, |m, v| m + v);
        if count == 0 {
            return None;
        }

        let mut picked_data = rand::random::<usize>() % count;

        for data in self.pages_data.iter() {
            if data.is_maybe_waiting_returning() {
                if picked_data == 0 {
                    return data.generate_return_book_information();
                }

                picked_data -= 1;
            }
        }

        None
    }
}

#[derive(Clone)]
pub enum SignFrameEntry {
    BorrowingSign,
    ReturningSign,
}

pub struct SignFrame {
    sign_frame: TableFrame,
    borrowing_is_done: bool,
    returning_is_done: bool,
    borrowing_sign: Option<UniTexture>,
    returning_sign: Option<UniTexture>,
    drwob_essential: DrawableObjectEssential,
}

impl SignFrame {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, position: numeric::Point2f, depth: i8) -> Self {
        let sign_frame = TableFrame::new(
            ctx.resource,
            position,
            TileBatchTextureID::RedOldStyleFrame,
            FrameData::new(vec![80.0; 2], vec![80.0]),
            numeric::Vector2f::new(0.3, 0.3),
            0,
        );

        SignFrame {
            sign_frame: sign_frame,
            borrowing_is_done: false,
            returning_is_done: false,
            borrowing_sign: None,
            returning_sign: None,
            drwob_essential: DrawableObjectEssential::new(true, depth),
        }
    }

    pub fn contains_sign_frame(
        &self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> Option<SignFrameEntry> {
        let maybe_grid_position = self.sign_frame.get_grid_position(ctx, point);
        if let Some(grid_position) = maybe_grid_position {
            match grid_position.y {
                0 => Some(SignFrameEntry::BorrowingSign),
                1 => Some(SignFrameEntry::ReturningSign),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn sign_borrowing_frame(&mut self, ctx: &mut SuzuContext) {
        let mut sign_texture = UniTexture::new(
            ctx.ref_texture(TextureID::Hanko),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(0.5, 0.5),
            0.0,
            0,
        );

        set_table_frame_cell_center!(
            ctx.context,
            self.sign_frame,
            sign_texture,
            numeric::Vector2u::new(0, 0)
        );

        self.borrowing_is_done = true;

        self.borrowing_sign = Some(sign_texture);
    }

    pub fn sign_returning_frame(&mut self, ctx: &mut SuzuContext) {
        let mut sign_texture = UniTexture::new(
            ctx.ref_texture(TextureID::Hanko),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(0.5, 0.5),
            0.0,
            0,
        );

        set_table_frame_cell_center!(
            ctx.context,
            self.sign_frame,
            sign_texture,
            numeric::Vector2u::new(0, 1)
        );

        self.returning_is_done = true;

        self.returning_sign = Some(sign_texture);
    }

    pub fn borrowing_signing_is_done(&self) -> bool {
        self.borrowing_sign.is_some()
    }

    pub fn retuning_signing_is_done(&self) -> bool {
        self.returning_sign.is_some()
    }
}

impl DrawableComponent for SignFrame {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.sign_frame.draw(ctx)?;

            if let Some(texture) = self.borrowing_sign.as_mut() {
                texture.draw(ctx)?;
            }

            if let Some(texture) = self.returning_sign.as_mut() {
                texture.draw(ctx)?;
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

pub struct PayFrame {
    pay_frame: TableFrame,
    cell_desc_text: Vec<VerticalText>,
    rental_limit_text: Option<VerticalText>,
    borrowing_number_text: Option<VerticalText>,
    pay_money_text: Option<VerticalText>,
    rental_limit_data: Option<RentalLimit>,
    listed_books_number: usize,
    drwob_essential: DrawableObjectEssential,
    calculated_price: Option<u32>,
}

impl PayFrame {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, position: numeric::Point2f, depth: i8) -> Self {
        let pay_frame = TableFrame::new(
            ctx.resource,
            position,
            TileBatchTextureID::RedOldStyleFrame,
            FrameData::new(vec![160.0, 300.0], vec![42.0; 3]),
            numeric::Vector2f::new(0.3, 0.3),
            0,
        );

        let mut borrowing_number = VerticalText::new(
            "貸出冊数".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                ctx.resource.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );

        set_table_frame_cell_center!(
            ctx.context,
            pay_frame,
            borrowing_number,
            numeric::Vector2u::new(2, 0)
        );

        let mut rental_limit = VerticalText::new(
            "貸出期限".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                ctx.resource.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );

        set_table_frame_cell_center!(
            ctx.context,
            pay_frame,
            rental_limit,
            numeric::Vector2u::new(1, 0)
        );

        let mut total = VerticalText::new(
            "合計".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                ctx.resource.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );

        set_table_frame_cell_center!(ctx.context, pay_frame, total, numeric::Vector2u::new(0, 0));

        let mut pay_frame = PayFrame {
            pay_frame: pay_frame,
            cell_desc_text: vec![borrowing_number, rental_limit, total],
            rental_limit_text: None,
            borrowing_number_text: None,
            pay_money_text: None,
            rental_limit_data: None,
            listed_books_number: 0,
            drwob_essential: DrawableObjectEssential::new(true, depth),
            calculated_price: None,
        };

        pay_frame.update_book_count(ctx, 0, 0);

        pay_frame
    }

    pub fn update_rental_limit_text<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        rental_limit: RentalLimit,
        base_price: u32,
    ) {
        let text = match rental_limit {
            RentalLimit::ShortTerm => "短期",
            RentalLimit::LongTerm => "長期",
            RentalLimit::Today => "",
        }
        .to_string();

        let mut vtext = VerticalText::new(
            text,
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                ctx.resource.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );

        set_table_frame_cell_center!(
            ctx.context,
            self.pay_frame,
            vtext,
            numeric::Vector2u::new(1, 1)
        );

        self.rental_limit_text = Some(vtext);
        self.rental_limit_data = Some(rental_limit);

        self.calc_payment_money(ctx, base_price);
    }

    pub fn update_book_count<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        count: usize,
        base_price: u32,
    ) {
        let mut vtext = VerticalText::new(
            format!("{}冊", number_to_jk(count as u64)),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                ctx.resource.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );

        set_table_frame_cell_center!(
            ctx.context,
            self.pay_frame,
            vtext,
            numeric::Vector2u::new(2, 1)
        );

        self.borrowing_number_text = Some(vtext);
        self.listed_books_number = count;

        self.calc_payment_money(ctx, base_price);
    }

    fn calc_payment_money<'a>(&mut self, ctx: &mut SuzuContext<'a>, base_price: u32) {
        if let Some(rental_limit) = self.rental_limit_data.as_ref() {
            // 返却期限がTodayの場合は計算を行わず、終了
            if rental_limit == &RentalLimit::Today {
                return;
            }

            self.calculated_price = Some((rental_limit.fee_rate() * base_price as f32) as u32);

            let mut vtext = VerticalText::new(
                format!("{}円", number_to_jk(self.calculated_price.unwrap() as u64)),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                FontInformation::new(
                    ctx.resource.get_font(FontID::JpFude1),
                    numeric::Vector2f::new(24.0, 24.0),
                    ggraphics::Color::from_rgba_u32(0x000000ff),
                ),
            );
            set_table_frame_cell_center!(
                ctx.context,
                self.pay_frame,
                vtext,
                numeric::Vector2u::new(0, 1)
            );

            self.pay_money_text = Some(vtext);
        }
    }

    pub fn get_pay_frame(&self) -> &TableFrame {
        &self.pay_frame
    }

    pub fn get_calculated_price(&self) -> Option<u32> {
        self.calculated_price.clone()
    }
}

impl DrawableComponent for PayFrame {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.pay_frame.draw(ctx)?;

            for vtext in &mut self.cell_desc_text {
                vtext.draw(ctx)?;
            }

            if let Some(vtext) = self.borrowing_number_text.as_mut() {
                vtext.draw(ctx)?;
            }

            if let Some(vtext) = self.rental_limit_text.as_mut() {
                vtext.draw(ctx)?;
            }

            if let Some(vtext) = self.pay_money_text.as_mut() {
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
    pay_frame: PayFrame,
    sign_frame: SignFrame,
    paper_texture: SimpleObject,
    drwob_essential: DrawableObjectEssential,
}

impl BorrowingRecordBookPage {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        rect: ggraphics::Rect,
        paper_tid: TextureID,
        page_data: BorrowingRecordBookPageData,
        t: Clock,
    ) -> Self {
        let mut page = Self::new_empty(ctx, rect, paper_tid, t);

        for (position, book_info) in page_data.borrowing_book_title.iter() {
            let info = page.borrow_book.get_mut(&position).unwrap();
            info.reset(HoldData::BookName(book_info.clone()));
            info.vtext.make_center(
                ctx.context,
                page.books_table
                    .get_center_of(*position, page.books_table.get_position()),
            );
        }

        for (position, book_status) in page_data.borrowing_book_status.iter() {
            let info = page.borrow_book.get_mut(&position).unwrap();
            info.reset(HoldData::BookCondition(book_status.clone()));
            info.vtext.make_center(
                ctx.context,
                page.books_table
                    .get_center_of(*position, page.books_table.get_position()),
            );
        }

        if let Some(customer_name) = page_data.customer_name {
            let position = numeric::Vector2u::new(2, 1);
            let info = page.request_information.get_mut(&position).unwrap();
            info.reset(HoldData::CustomerName(customer_name.clone()));
            info.vtext.make_center(
                ctx.context,
                page.customer_info_table
                    .get_center_of(position, page.customer_info_table.get_position()),
            );
        }

        if let Some(rental_date) = page_data.rental_date {
            let position = numeric::Vector2u::new(1, 1);
            let info = page.request_information.get_mut(&position).unwrap();
            info.reset(HoldData::Date(rental_date.clone()));
            info.vtext.make_center(
                ctx.context,
                page.customer_info_table
                    .get_center_of(position, page.customer_info_table.get_position()),
            );
        }

        if let Some(return_date) = page_data.return_date {
            let position = numeric::Vector2u::new(0, 1);
            let info = page.request_information.get_mut(&position).unwrap();
            info.reset(HoldData::Date(return_date.clone()));
            info.vtext.make_center(
                ctx.context,
                page.customer_info_table
                    .get_center_of(position, page.customer_info_table.get_position()),
            );
        }

        let base_price = page.base_price_of_written_book();

        if let Some(rental_limit) = page_data.rental_limit {
            page.pay_frame
                .update_rental_limit_text(ctx, rental_limit, base_price);
        }

        page.pay_frame
            .update_book_count(ctx, page_data.borrowing_book_title.len(), base_price);

        if page_data.borrowing_is_signed {
            page.sign_frame.sign_borrowing_frame(ctx);
        }

        if page_data.returning_is_signed {
            page.sign_frame.sign_returning_frame(ctx);
        }

        page
    }

    pub fn new_empty<'a>(
        ctx: &mut SuzuContext<'a>,
        rect: ggraphics::Rect,
        paper_tid: TextureID,
        t: Clock,
    ) -> Self {
        let table_frame = TableFrame::new(
            ctx.resource,
            numeric::Point2f::new(rect.right() - 200.0, 40.0),
            TileBatchTextureID::OldStyleFrame,
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
                ctx.resource.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );

        set_table_frame_cell_center!(
            ctx.context,
            table_frame,
            borrower,
            numeric::Vector2u::new(2, 0)
        );

        let mut borrow_date = VerticalText::new(
            "貸出日".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                ctx.resource.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );

        set_table_frame_cell_center!(
            ctx.context,
            table_frame,
            borrow_date,
            numeric::Vector2u::new(1, 0)
        );

        let mut return_date = VerticalText::new(
            "返却期限".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                ctx.resource.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );

        set_table_frame_cell_center!(
            ctx.context,
            table_frame,
            return_date,
            numeric::Vector2u::new(0, 0)
        );

        let books_table = TableFrame::new(
            ctx.resource,
            numeric::Point2f::new(rect.right() - 550.0, 30.0),
            TileBatchTextureID::OldStyleFrame,
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
                ctx.resource.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );

        set_table_frame_cell_center!(
            ctx.context,
            books_table,
            book_head,
            numeric::Vector2u::new(5, 0)
        );

        let mut book_status = VerticalText::new(
            "状態".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                ctx.resource.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );

        set_table_frame_cell_center!(
            ctx.context,
            books_table,
            book_status,
            numeric::Vector2u::new(5, 1)
        );

        let paper_texture = SimpleObject::new(
            MovableUniTexture::new(
                Box::new(UniTexture::new(
                    ctx.ref_texture(paper_tid),
                    numeric::Point2f::new(rect.x, rect.y),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                )),
                move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                t,
            ),
            Vec::new(),
        );

        let info_font = FontInformation::new(
            ctx.resource.get_font(FontID::JpFude1),
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
            pay_frame: PayFrame::new(ctx, numeric::Point2f::new(220.0, 40.0), 0),
            sign_frame: SignFrame::new(ctx, numeric::Point2f::new(rect.left() + 30.0, rect.bottom() - 190.0), 0),
            return_date: return_date,
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }

    pub fn get_books_table_rows(&self) -> usize {
        self.books_table.get_rows()
    }

    pub fn get_calculated_price(&self) -> Option<u32> {
        self.pay_frame.get_calculated_price()
    }

    pub fn get_written_books(&self) -> Vec<BookInformation> {
        let mut book_info = Vec::new();

        for (_, hold_data) in self.borrow_book.iter() {
            match hold_data.ref_hold_data() {
                HoldData::BookName(info) => book_info.push(info.clone()),
                _ => (),
            }
        }

        book_info
    }

    pub fn create_current_book_condition_report(&self) -> BookConditionEvalReport {
        let mut originals = Vec::new();
        let mut evals = Vec::new();

        let size = self.books_table.get_rows();
        for index in 0..size {
            let key = numeric::Vector2u::new(index as u32, 0);
            let book_info = match self.borrow_book.get(&key).as_ref().unwrap().ref_hold_data() {
                HoldData::BookName(info) => info,
                HoldData::None => continue,
                _ => panic!("BUG"),
            };

            let key = numeric::Vector2u::new(index as u32, 1);
            let eval = match self.borrow_book.get(&key).unwrap().ref_hold_data() {
                HoldData::BookCondition(status) => status,
                HoldData::None => continue,
                _ => panic!("BUG"),
            };

            originals.push(book_info.clone());
            evals.push(eval.clone());
        }

        BookConditionEvalReport::new(originals, evals)
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

    fn count_written_book_title(&self) -> usize {
        self.borrow_book
            .iter()
            .map(|(_, data)| if data.is_none() { 0 } else { 1 })
            .fold(0, |sum, c| sum + c)
    }

    fn base_price_of_written_book(&self) -> u32 {
        self.borrow_book
            .iter()
            .map(|(_, data)| match data.ref_hold_data() {
                HoldData::BookName(info) => info.base_price,
                _ => 0,
            })
            .fold(0, |sum, c| sum + c)
    }

    fn borrowing_signing_is_available(&self) -> bool {
        // 何らかの値段が設定されていればOK
        self.pay_frame.calculated_price.is_some() && !self.sign_frame.borrowing_signing_is_done()
    }

    fn returning_signing_is_available(&self) -> bool {
        let rows_size = self.books_table.get_rows();
        for index in 0..rows_size {
            // ある列の先頭の要素が存在し、かつ本の情報である場合、下の欄に状態が記載されてるか確認する
            let data = self
                .borrow_book
                .get(&numeric::Vector2u::new(index as u32, 0));
            if let Some(data) = data.as_ref() {
                match data.ref_hold_data() {
                    HoldData::BookName(_) => {
                        match self
                            .borrow_book
                            .get(&numeric::Vector2u::new(index as u32, 1))
                            .unwrap()
                            .data
                        {
                            HoldData::BookCondition(_) => (),
                            _ => return false, // 本の状態の情報が無い場合、書き漏れがあるとして、falseを返す
                        }
                    }
                    _ => (),
                }
            }
        }

        true
    }

    ///
    /// Dataを格納できればtrue, できなければfalse
    ///
    pub fn try_insert_data_in_borrowing_books_frame<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        menu_position: numeric::Point2f,
        book_info: BookInformation,
    ) -> DrawRequest {
        let grid_pos = self
            .books_table
            .get_grid_position(ctx.context, menu_position)
            .unwrap();

        // 本の状態は、このメソッドからは設定できない
        if grid_pos.y == 1 {
            return DrawRequest::Skip;
        }

        let info = self.borrow_book.get_mut(&grid_pos).unwrap();
        info.reset(HoldData::BookName(book_info));
        info.vtext.make_center(
            ctx.context,
            self.books_table
                .get_center_of(grid_pos, self.books_table.get_position()),
        );

        self.pay_frame.update_book_count(
            ctx,
            self.count_written_book_title(),
            self.base_price_of_written_book(),
        );

        DrawRequest::Draw
    }

    ///
    /// Dataを格納できればtrue, できなければfalse
    ///
    pub fn try_insert_date_data_in_cutomer_info_frame<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        menu_position: numeric::Point2f,
        date: GensoDate,
        rental_limit: RentalLimit,
    ) -> DrawRequest {
        let grid_pos = self
            .customer_info_table
            .get_grid_position(ctx.context, menu_position)
            .unwrap();

        // 挿入先が日時のエントリではない
        if !(grid_pos == numeric::Vector2u::new(0, 1) || grid_pos == numeric::Vector2u::new(1, 1)) {
            return DrawRequest::Skip;
        }

        let info = self.request_information.get_mut(&grid_pos).unwrap();
        info.reset(HoldData::Date(date));
        info.vtext.make_center(
            ctx.context,
            self.customer_info_table
                .get_center_of(grid_pos, self.customer_info_table.get_position()),
        );

        if grid_pos.x == 0 {
            self.pay_frame.update_rental_limit_text(
                ctx,
                rental_limit,
                self.base_price_of_written_book(),
            );
        }

        DrawRequest::Draw
    }

    ///
    /// CustomerNameを明示的に設定するメソッド, 格納できればtrue, できなければfalse
    ///
    pub fn try_insert_customer_name_in_cutomer_info_frame(
        &mut self,
        ctx: &mut ggez::Context,
        menu_position: numeric::Point2f,
        customer_name: String,
    ) -> DrawRequest {
        let grid_pos = self
            .customer_info_table
            .get_grid_position(ctx, menu_position)
            .unwrap();

        // 挿入先が日時のエントリではない
        if grid_pos != numeric::Vector2u::new(2, 1) {
            return DrawRequest::Skip;
        }

        let info = self.request_information.get_mut(&grid_pos).unwrap();
        info.reset(HoldData::CustomerName(customer_name));
        info.vtext.make_center(
            ctx,
            self.customer_info_table
                .get_center_of(grid_pos, self.customer_info_table.get_position()),
        );

        DrawRequest::Draw
    }

    pub fn replace_borrower_name(&mut self, game_data: &GameResource, name: &str) -> &mut Self {
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
        let grid_position = self
            .books_table
            .get_grid_position(ctx, menu_position)
            .unwrap();
        let info = self.borrow_book.get_mut(&grid_position).unwrap();
        info.reset(HoldData::BookCondition(BookCondition::from(status_index)));
        info.vtext.make_center(
            ctx,
            self.books_table
                .get_center_of(grid_position, self.books_table.get_position()),
        );
    }

    fn remove_book_status_data(
        &mut self,
        ctx: &mut ggez::Context,
        menu_position: numeric::Point2f,
    ) {
        let grid_position = self
            .books_table
            .get_grid_position(ctx, menu_position)
            .unwrap();
        let info = self.borrow_book.get_mut(&grid_position).unwrap();
        info.reset(HoldData::None);
    }
    
    pub fn get_return_date(&self) -> Option<GensoDate> {
        if let Some(data) = self.request_information.get(&numeric::Vector2u::new(0, 1)) {
            data.ref_hold_data().to_date()
        } else {
            None
        }
    }
    
    pub fn export_page_data(&self) -> BorrowingRecordBookPageData {
        let mut borrow_book_title = Vec::new();
        let mut borrow_book_status = Vec::new();

        for index in 0..self.books_table.get_rows() {
            let position = numeric::Vector2u::new(index as u32, 0);
            let hold_data_vtext = self.borrow_book.get(&position);

            if let Some(hold_data_vtext) = hold_data_vtext {
                if !hold_data_vtext.is_none() {
                    match hold_data_vtext.copy_hold_data() {
                        HoldData::BookName(info) => {
                            borrow_book_title.push((position, info));
                        }
                        _ => (),
                    }
                }
            }
        }

        for index in 0..self.books_table.get_rows() {
            let position = numeric::Vector2u::new(index as u32, 1);
            let hold_data_vtext = self.borrow_book.get(&position);

            if let Some(hold_data_vtext) = hold_data_vtext {
                if !hold_data_vtext.is_none() {
                    match hold_data_vtext.copy_hold_data() {
                        HoldData::BookCondition(status) => {
                            borrow_book_status.push((position, status));
                        }
                        _ => (),
                    }
                }
            }
        }

        let customer_name =
            if let Some(data) = self.request_information.get(&numeric::Vector2u::new(2, 1)) {
                data.ref_hold_data().to_customer_name()
            } else {
                None
            };

        let rental_date =
            if let Some(data) = self.request_information.get(&numeric::Vector2u::new(1, 1)) {
                data.ref_hold_data().to_date()
            } else {
                None
            };

        let return_date = self.get_return_date();

        let rental_limit = if let Some(data) = self.pay_frame.rental_limit_data.as_ref() {
            Some(data.clone())
        } else {
            None
        };

        BorrowingRecordBookPageData {
            borrowing_book_title: borrow_book_title,
            borrowing_book_status: borrow_book_status,
            customer_name: customer_name,
            return_date: return_date,
            rental_date: rental_date,
            rental_limit: rental_limit,
            borrowing_is_signed: self.sign_frame.borrowing_is_done,
            returning_is_signed: self.sign_frame.returning_is_done,
        }
    }

    pub fn sign_with_mouse_click<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
    ) -> Option<SignFrameEntry> {
        let maybe_sign_entry = self.sign_frame.contains_sign_frame(ctx.context, point);

        if let Some(sign_entry) = maybe_sign_entry.as_ref() {
            match sign_entry {
                SignFrameEntry::BorrowingSign => {
                    if self.borrowing_signing_is_available() {
                        self.sign_frame.sign_borrowing_frame(ctx);
                        return maybe_sign_entry;
                    }
                }
                SignFrameEntry::ReturningSign => {
                    if self.returning_signing_is_available() {
                        self.sign_frame.sign_returning_frame(ctx);
                        return maybe_sign_entry;
                    }
                }
            }
        }

        None
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

            self.pay_frame.draw(ctx)?;
            self.sign_frame.draw(ctx)?;

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

pub struct AlphaScope {
    scope: ggraphics::Mesh,
    draw_param: ggraphics::DrawParam,
    drwob_essential: DrawableObjectEssential,
}

impl AlphaScope {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        radius: u32,
        max_sub_alpha: u8,
        pos: numeric::Point2f,
        depth: i8,
    ) -> Self {
        let mut builder = ggraphics::MeshBuilder::new();
        let p_alpha = max_sub_alpha as f32 / radius as f32;
        let mut alpha = max_sub_alpha as f32;

        for r in 0..radius {
            builder.circle(
                ggraphics::DrawMode::stroke(1.0),
                mintp_new!(0.0, 0.0),
                r as f32,
                0.0001,
                ggraphics::Color::from_rgba_u32(alpha as u32),
            ).expect("Failed to create circle");

            alpha -= p_alpha;
        }

	let draw_param = ggraphics::DrawParam::default()
	    .dest(mintp!(pos))
	    .color(ggraphics::Color::from_rgba_u32(0xffffff00));

        AlphaScope {
            scope: builder.build(ctx.context).unwrap(),
            draw_param: draw_param,
            drwob_essential: DrawableObjectEssential::new(true, depth),
        }
    }
}

impl DrawableComponent for AlphaScope {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            ggraphics::set_blend_mode(ctx, ggraphics::BlendMode::Subtract).unwrap();
            ggraphics::draw(ctx, &self.scope, self.draw_param).unwrap();
            ggraphics::set_blend_mode(ctx, ggraphics::BlendMode::Alpha).unwrap();
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
    redraw_request: DrawRequest,
    pages: Vec<BorrowingRecordBookPage>,
    host_rect: numeric::Rect,
    page_rect: numeric::Rect,
    current_page: usize,
    next_page_ope_mesh: UniTexture,
    prev_page_ope_mesh: UniTexture,
    page_scroll_event_list: DelayEventList<Self>,
    //scope: AlphaScope,
    canvas: MovableWrap<SubScreen>,
    page_data_backup: BorrowingRecordBookData,
    next10_button: SelectButton,
    prev10_button: SelectButton,
}

impl BorrowingRecordBook {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        rect: ggraphics::Rect,
        drawing_depth: i8,
        mut book_data: BorrowingRecordBookData,
        t: Clock,
    ) -> Self {
        let backup = book_data.clone();
	let page_rect = numeric::Rect::new(60.0, 0.0, rect.w, rect.h);
	let host_rect = numeric::Rect::new(rect.x, rect.y, rect.w + 60.0, rect.h);
	
        let pages = {
            let mut pages = Vec::new();
	    
            while !book_data.pages_data.is_empty() {
                let page_data = book_data.pages_data.remove(0);
                pages.push(BorrowingRecordBookPage::new(
                    ctx,
                    page_rect,
                    TextureID::Paper1,
                    page_data,
                    t,
                ));
            }

            pages
        };

        let next = UniTexture::new(
            ctx.ref_texture(TextureID::GoNextPageLeft),
            numeric::Point2f::new(page_rect.x, page_rect.h - 32.0),
            numeric::Vector2f::new(0.5, 0.5),
            0.0,
            0,
        );

        let mut prev = UniTexture::new(
            ctx.ref_texture(TextureID::GoNextPageRight),
            numeric::Point2f::new(page_rect.right() - 32.0, page_rect.h - 32.0),
            numeric::Vector2f::new(0.5, 0.5),
            0.0,
            0,
        );
        prev.hide();

	let button_texture = Box::new(TextButtonTexture::new(
            ctx,
            numeric::Point2f::new(0.0, 0.0),
            "次10".to_string(),
            FontInformation::new(
                ctx.resource.get_font(FontID::Cinema),
                numeric::Vector2f::new(22.0, 22.0),
                ggraphics::Color::from_rgba_u32(0xff),
            ),
            10.0,
            ggraphics::Color::from_rgba_u32(0xfcf0eaff),
            0,
        ));

        let next10_button = SelectButton::new(
            ctx,
            numeric::Rect::new(0.0, 40.0, 60.0, 60.0),
            button_texture,
        );

	let button_texture = Box::new(TextButtonTexture::new(
            ctx,
            numeric::Point2f::new(0.0, 0.0),
            "前10".to_string(),
            FontInformation::new(
                ctx.resource.get_font(FontID::Cinema),
                numeric::Vector2f::new(22.0, 22.0),
                ggraphics::Color::from_rgba_u32(0xff),
            ),
            10.0,
            ggraphics::Color::from_rgba_u32(0xfff8edff),
            0,
        ));

        let prev10_button = SelectButton::new(
            ctx,
            numeric::Rect::new(0.0, 0.0, 60.0, 60.0),
            button_texture,
        );
	
        BorrowingRecordBook {
            redraw_request: DrawRequest::InitDraw,
            pages: pages,
	    host_rect: host_rect,
	    page_rect: page_rect,
            current_page: 0,
            next_page_ope_mesh: next,
            prev_page_ope_mesh: prev,
            canvas: MovableWrap::new(
                Box::new(SubScreen::new(
                    ctx.context,
                    host_rect,
                    drawing_depth,
                    ggraphics::Color::from_rgba_u32(0),
                )),
                None,
                0,
            ),
            //scope: AlphaScope::new(ctx, 50, 230, numeric::Point2f::new(100.0, 100.0), 0),
            page_data_backup: backup,
	    next10_button: next10_button,
	    prev10_button: prev10_button,
	    page_scroll_event_list: DelayEventList::new(),
        }
    }

    pub fn reset_pages_data<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        let pages_len = self.pages.len();
        self.pages = {
            let mut pages = Vec::new();

            for page_data in self.page_data_backup.pages_data.iter() {
                pages.push(BorrowingRecordBookPage::new(
                    ctx,
                    self.page_rect,
                    TextureID::Paper1,
                    page_data.clone(),
                    t,
                ));
            }

            pages
        };

        while self.pages.len() < pages_len {
            self.add_empty_page(ctx, t);
        }

        ctx.process_utility.redraw();
        self.redraw_request = DrawRequest::Draw;
    }

    pub fn add_empty_page<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) -> &Self {
        self.redraw_request = DrawRequest::Draw;

        self.pages.push(BorrowingRecordBookPage::new_empty(
            ctx,
            self.page_rect,
            TextureID::Paper1,
            t,
        ));
        self
    }

    fn get_current_page(&self) -> Option<&BorrowingRecordBookPage> {
        self.pages.get(self.current_page)
    }

    /// このメソッドをpubにしてはいけない
    /// 理由: mut参照を返すメソッドを外に公開すると、変更がどこで行われているのか
    ///       追跡できなくなるから
    fn get_current_page_mut(&mut self) -> Option<&mut BorrowingRecordBookPage> {
        self.pages.get_mut(self.current_page)
    }

    pub fn relative_point(&self, point: numeric::Point2f) -> numeric::Point2f {
        self.canvas.relative_point(point)
    }

    ///
    /// 次のページが存在する場合は、ページを繰って、trueを返す
    /// 存在しない場合は、falseを返す
    ///
    fn next_page<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        self.redraw_request = DrawRequest::Draw;
        self.current_page += 1;
        if self.current_page >= self.pages.len() {
            self.add_empty_page(ctx, t);
        }
        self.prev_page_ope_mesh.appear();
        ctx.play_sound_as_se(SoundID::SeTurnThePage, None);
    }

    fn prev_page<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        self.redraw_request = DrawRequest::Draw;
        self.prev_page_ope_mesh.appear();

        if self.current_page > 0 {
            self.current_page -= 1;

            if self.current_page == 0 {
                self.prev_page_ope_mesh.hide();
            }

            ctx.play_sound_as_se(SoundID::SeTurnThePage, None);
        }
    }

    pub fn insert_book_title_to_books_frame<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        menu_position: numeric::Point2f,
        book_info: BookInformation,
    ) {
        let rpoint = self.relative_point(menu_position);
        if let Some(page) = self.get_current_page_mut() {
            self.redraw_request =
                page.try_insert_data_in_borrowing_books_frame(ctx, rpoint, book_info);
        }
    }

    pub fn get_current_page_written_books<'a>(&self) -> Option<Vec<BookInformation>> {
        if let Some(page) = self.get_current_page() {
            Some(page.get_written_books())
        } else {
            None
        }
    }

    pub fn get_current_page_return_date(&self) -> Option<GensoDate> {	
        if let Some(page) = self.get_current_page() {
            page.get_return_date()
        } else {
            None
        }
    }

    pub fn insert_date_data_to_customer_info<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        menu_position: numeric::Point2f,
        date: GensoDate,
        rental_limit: RentalLimit,
    ) {
        let rpoint = self.relative_point(menu_position);
        if let Some(page) = self.get_current_page_mut() {
            self.redraw_request =
                page.try_insert_date_data_in_cutomer_info_frame(ctx, rpoint, date, rental_limit);
        }
    }

    pub fn get_calculated_price(&self) -> Option<u32> {
        if let Some(page) = self.get_current_page() {
            page.get_calculated_price()
        } else {
            None
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
            self.redraw_request =
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

    pub fn get_current_page_condition_eval_report(&self) -> Option<BookConditionEvalReport> {
        match self.get_current_page() {
            Some(page) => Some(page.create_current_book_condition_report()),
            None => None,
        }
    }

    pub fn click_handler<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        t: Clock,
        point: numeric::Point2f,
    ) -> bool {
        let rpoint = self.relative_point(point);

        if self.next_page_ope_mesh.contains(ctx.context, rpoint) {
            self.next_page(ctx, t);
            self.check_move_page_icon_visibility();
            return true;
        } else if self.prev_page_ope_mesh.contains(ctx.context, rpoint) {
            self.prev_page(ctx);
            self.check_move_page_icon_visibility();
            return true;
        } else if self.next10_button.contains(ctx.context, rpoint) {
	    self.redraw_request = DrawRequest::Draw;
	    self.page_scroll_event_list.clear();
	    
	    for i in 0..10 {
		if self.current_page + i + 1 < self.pages.len() {
		    self.page_scroll_event_list.add_event(
			Box::new(|slf, _, _| {
			    slf.current_page += 1;
			    slf.redraw_request = DrawRequest::Draw;
			    slf.check_move_page_icon_visibility();
			}),
			t + (i * 2) as Clock,
		    );
		} else {
		    break;
		}
	    }
	    ctx.play_sound_as_se(SoundID::SeTurnThePage, None);
	    self.check_move_page_icon_visibility();
	} else if self.prev10_button.contains(ctx.context, rpoint) {
	    self.page_scroll_event_list.clear();
	    for i in 0..10 {
		if self.current_page as i32 - i > 0 {
		    self.page_scroll_event_list.add_event(
			Box::new(|slf, _, _| {
			    slf.current_page -= 1;
			    slf.redraw_request = DrawRequest::Draw;
			    slf.check_move_page_icon_visibility();
			}),
			t + (i * 2) as Clock,
		    );
		} else {
		    break;
		}
	    }
	    ctx.play_sound_as_se(SoundID::SeTurnThePage, None);
	}

        false
    }

    pub fn sign_with_mouse_click<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
    ) -> Option<SignFrameEntry> {
        let rpoint = self.relative_point(point);

        if let Some(page) = self.get_current_page_mut() {
            let ret = page.sign_with_mouse_click(ctx, rpoint);
            if ret.is_some() {
                self.redraw_request = DrawRequest::Draw;
            }
            ret
        } else {
            None
        }
    }

    pub fn pages_length(&self) -> usize {
        self.pages.len()
    }

    pub fn update<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        if !self.is_stop() {
            ctx.process_utility.redraw();
            self.move_with_func(t);
        }

	flush_delay_event_and_redraw_check!(self, self.page_scroll_event_list, ctx, t, {})
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

    pub fn get_payment_frame_grid_position(
        &self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> Option<numeric::Vector2u> {
        if let Some(page) = self.get_current_page().as_ref() {
            let page_point = self.relative_point(point);
            page.pay_frame
                .get_pay_frame()
                .get_grid_position(ctx, page_point)
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
            self.redraw_request = DrawRequest::Draw;
        }
    }

    pub fn remove_book_status_at(
        &mut self,
        ctx: &mut ggez::Context,
        menu_position: numeric::Point2f,
    ) {
        let rpoint = self.relative_point(menu_position);
        if let Some(page) = self.get_current_page_mut() {
            page.remove_book_status_data(ctx, rpoint);
            self.redraw_request = DrawRequest::Draw;
        }
    }

    pub fn mouse_motion_handler(&mut self, point: numeric::Point2f) {
        let _rpoint = self.canvas.relative_point(point);
    }

    pub fn get_books_table_rows(&self) -> Option<usize> {
        if let Some(page) = self.get_current_page() {
            Some(page.get_books_table_rows())
        } else {
            None
        }
    }

    fn check_move_page_icon_visibility(&mut self) {
        self.next_page_ope_mesh.appear();
        self.prev_page_ope_mesh.appear();

        if self.current_page == 0 {
            self.prev_page_ope_mesh.hide();
        }

        self.redraw_request = DrawRequest::Draw;
    }
}

impl DrawableComponent for BorrowingRecordBook {
    #[inline(always)]
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            if self.redraw_request != DrawRequest::Skip {
                self.redraw_request = DrawRequest::Skip;
                sub_screen::stack_screen(ctx, &self.canvas);

		self.next10_button.draw(ctx)?;
		self.prev10_button.draw(ctx)?;
		
                if self.pages.len() > 0 {
                    self.pages.get_mut(self.current_page).unwrap().draw(ctx)?;
                }
		
                self.prev_page_ope_mesh.draw(ctx)?;
                self.next_page_ope_mesh.draw(ctx)?;

                //self.scope.draw(ctx)?;

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

    fn override_move_func(&mut self, move_fn: Option<GenericMoveFn>, now: Clock) {
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

    fn click_hold_data(&self, _: &mut ggez::Context, _: numeric::Point2f) -> HoldData {
        HoldData::None
    }

    fn get_type(&self) -> OnDeskType {
        OnDeskType::BorrowingRecordBook
    }
}

pub struct TaskItemStruct<S, L>
where
    S: OnDesk,
    L: OnDesk,
{
    small: Box<EffectableWrap<MovableWrap<S>>>,
    large: Box<EffectableWrap<MovableWrap<L>>>,
    switch: u8,
    handover_locked: bool,
    shelving_box_locked: bool,
    object_type: DeskObjectType,
    drag_point: numeric::Vector2f,
}

impl<S, L> TaskItemStruct<S, L>
where
    S: OnDesk,
    L: OnDesk,
{
    pub fn new(
        small: S,
        large: L,
        switch: u8,
        handover_locked: bool,
        shelving_box_locked: bool,
        obj_type: DeskObjectType,
        t: Clock,
    ) -> TaskItemStruct<S, L> {
        TaskItemStruct {
            small: Box::new(EffectableWrap::new(
                MovableWrap::new(Box::new(small), None, t),
                Vec::new(),
            )),
            large: Box::new(EffectableWrap::new(
                MovableWrap::new(Box::new(large), None, t),
                Vec::new(),
            )),
            switch: switch,
            handover_locked: handover_locked,
            shelving_box_locked: shelving_box_locked,
            object_type: obj_type,
            drag_point: numeric::Vector2f::new(0.0, 0.0),
        }
    }

    pub fn enable_small(&mut self) {
        self.switch = 0;
    }

    pub fn enable_large(&mut self) {
        self.switch = 1;
    }

    pub fn get_object_type(&self) -> DeskObjectType {
        self.object_type
    }

    pub fn get_object(&self) -> &dyn OnDesk {
        match self.switch {
            0 => self
                .small
                .ref_wrapped_object()
                .ref_wrapped_object()
                .as_ref(),
            1 => self
                .large
                .ref_wrapped_object()
                .ref_wrapped_object()
                .as_ref(),
            _ => panic!("Failed to object selecting. select = {}", self.switch),
        }
    }

    pub fn get_object_mut(&mut self) -> &mut dyn OnDesk {
        match self.switch {
            0 => self
                .small
                .ref_wrapped_object_mut()
                .ref_wrapped_object_mut()
                .as_mut(),
            1 => self
                .large
                .ref_wrapped_object_mut()
                .ref_wrapped_object_mut()
                .as_mut(),
            _ => panic!("Failed to object selecting. select = {}", self.switch),
        }
    }

    pub fn get_movable_object(&self) -> &dyn MovableObject {
        match self.switch {
            0 => self.small.ref_wrapped_object(),
            1 => self.large.ref_wrapped_object(),
            _ => panic!("Failed to object selecting. select = {}", self.switch),
        }
    }

    pub fn get_movable_object_mut(&mut self) -> &mut dyn MovableObject {
        match self.switch {
            0 => self.small.ref_wrapped_object_mut(),
            1 => self.large.ref_wrapped_object_mut(),
            _ => panic!("Failed to object selecting. select = {}", self.switch),
        }
    }

    pub fn get_effectable_object(&mut self) -> &mut dyn HasGenericEffect {
        match self.switch {
            0 => self.small.as_mut(),
            1 => self.large.as_mut(),
            _ => panic!("Failed to object selecting. select = {}", self.switch),
        }
    }

    pub fn is_handover_locked(&self) -> bool {
        self.handover_locked
    }

    pub fn lock_handover(&mut self) {
        self.handover_locked = true;
    }

    pub fn unlock_handover(&mut self) {
        self.handover_locked = false;
    }

    pub fn is_shelving_box_handover_locked(&self) -> bool {
        self.shelving_box_locked
    }

    pub fn lock_shelving_box_handover(&mut self) {
        self.shelving_box_locked = true;
    }

    pub fn unlock_shelving_box_handover(&mut self) {
        self.shelving_box_locked = false;
    }

    pub fn get_small_object(&self) -> &S {
        self.small.ref_wrapped_object().ref_wrapped_object()
    }

    pub fn get_small_object_mut(&mut self) -> &mut S {
        self.small.ref_wrapped_object_mut().ref_wrapped_object_mut()
    }

    pub fn get_large_object(&self) -> &L {
        self.large.ref_wrapped_object().ref_wrapped_object()
    }

    pub fn get_large_object_mut(&mut self) -> &mut L {
        self.large.ref_wrapped_object_mut().ref_wrapped_object_mut()
    }

    pub fn get_drag_point(&self) -> numeric::Vector2f {
        self.drag_point
    }

    pub fn set_drag_point(&mut self, drag_point: numeric::Vector2f) {
        self.drag_point = drag_point;
    }
}

pub type TaskBook = TaskItemStruct<OnDeskTexture, OnDeskBook>;
pub type TaskTexture = TaskItemStruct<OnDeskTexture, OnDeskTexture>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeskObjectType {
    CustomerObject = 0,
    BorrowRecordBook,
    ManualBook,
    SuzunaObject,
    Coin,
}

pub enum TaskItem {
    Book(TaskBook),
    Texture(TaskTexture),
    Coin(TaskTexture),
}

impl TaskItem {
    pub fn enable_small(&mut self) {
        match self {
            TaskItem::Book(item) => item.enable_small(),
            TaskItem::Texture(item) => item.enable_small(),
            TaskItem::Coin(item) => item.enable_small(),
        }
    }

    pub fn enable_large(&mut self) {
        match self {
            TaskItem::Book(item) => item.enable_large(),
            TaskItem::Texture(item) => item.enable_large(),
            TaskItem::Coin(item) => item.enable_large(),
        }
    }

    pub fn get_object_type(&self) -> DeskObjectType {
        match self {
            TaskItem::Book(item) => item.get_object_type(),
            TaskItem::Texture(item) => item.get_object_type(),
            TaskItem::Coin(item) => item.get_object_type(),
        }
    }

    pub fn get_object(&self) -> &dyn OnDesk {
        match self {
            TaskItem::Book(item) => item.get_object(),
            TaskItem::Texture(item) => item.get_object(),
            TaskItem::Coin(item) => item.get_object(),
        }
    }

    pub fn get_object_mut(&mut self) -> &mut dyn OnDesk {
        match self {
            TaskItem::Book(item) => item.get_object_mut(),
            TaskItem::Texture(item) => item.get_object_mut(),
            TaskItem::Coin(item) => item.get_object_mut(),
        }
    }

    pub fn as_movable_object(&self) -> &dyn MovableObject {
        match self {
            TaskItem::Book(item) => item.get_movable_object(),
            TaskItem::Texture(item) => item.get_movable_object(),
            TaskItem::Coin(item) => item.get_movable_object(),
        }
    }

    pub fn as_movable_object_mut(&mut self) -> &mut dyn MovableObject {
        match self {
            TaskItem::Book(item) => item.get_movable_object_mut(),
            TaskItem::Texture(item) => item.get_movable_object_mut(),
            TaskItem::Coin(item) => item.get_movable_object_mut(),
        }
    }

    pub fn as_effectable_object(&mut self) -> &mut dyn HasGenericEffect {
        match self {
            TaskItem::Book(item) => item.get_effectable_object(),
            TaskItem::Texture(item) => item.get_effectable_object(),
            TaskItem::Coin(item) => item.get_effectable_object(),
        }
    }

    pub fn is_handover_locked(&self) -> bool {
        match self {
            TaskItem::Book(item) => item.is_handover_locked(),
            TaskItem::Texture(item) => item.is_handover_locked(),
            TaskItem::Coin(item) => item.is_handover_locked(),
        }
    }

    pub fn lock_handover(&mut self) {
        match self {
            TaskItem::Book(item) => item.lock_handover(),
            TaskItem::Texture(item) => item.lock_handover(),
            TaskItem::Coin(item) => item.lock_handover(),
        }
    }

    pub fn unlock_handover(&mut self) {
        match self {
            TaskItem::Book(item) => item.unlock_handover(),
            TaskItem::Texture(item) => item.unlock_handover(),
            TaskItem::Coin(item) => item.unlock_handover(),
        }
    }

    pub fn is_shelving_box_handover_locked(&self) -> bool {
        match self {
            TaskItem::Book(item) => item.is_shelving_box_handover_locked(),
            TaskItem::Texture(item) => item.is_shelving_box_handover_locked(),
            TaskItem::Coin(item) => item.is_shelving_box_handover_locked(),
        }
    }

    pub fn lock_shelving_box_handover(&mut self) {
        match self {
            TaskItem::Book(item) => item.lock_shelving_box_handover(),
            TaskItem::Texture(item) => item.lock_shelving_box_handover(),
            TaskItem::Coin(item) => item.lock_shelving_box_handover(),
        }
    }

    pub fn unlock_shelving_box_handover(&mut self) {
        match self {
            TaskItem::Book(item) => item.unlock_shelving_box_handover(),
            TaskItem::Texture(item) => item.unlock_shelving_box_handover(),
            TaskItem::Coin(item) => item.unlock_shelving_box_handover(),
        }
    }

    pub fn get_drag_point(&self) -> numeric::Vector2f {
        match self {
            TaskItem::Book(item) => item.get_drag_point(),
            TaskItem::Texture(item) => item.get_drag_point(),
            TaskItem::Coin(item) => item.get_drag_point(),
        }
    }

    pub fn set_drag_point(&mut self, drag_point: numeric::Vector2f) {
        match self {
            TaskItem::Book(item) => item.set_drag_point(drag_point),
            TaskItem::Texture(item) => item.set_drag_point(drag_point),
            TaskItem::Coin(item) => item.set_drag_point(drag_point),
        }
    }
}

impl DrawableComponent for TaskItem {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        match self {
            TaskItem::Book(item) => item.get_object_mut().draw(ctx),
            TaskItem::Texture(item) => item.get_object_mut().draw(ctx),
            TaskItem::Coin(item) => item.get_object_mut().draw(ctx),
        }
    }

    fn hide(&mut self) {
        match self {
            TaskItem::Book(item) => item.get_object_mut().hide(),
            TaskItem::Texture(item) => item.get_object_mut().hide(),
            TaskItem::Coin(item) => item.get_object_mut().hide(),
        }
    }

    fn appear(&mut self) {
        match self {
            TaskItem::Book(item) => item.get_object_mut().appear(),
            TaskItem::Texture(item) => item.get_object_mut().appear(),
            TaskItem::Coin(item) => item.get_object_mut().appear(),
        }
    }

    fn is_visible(&self) -> bool {
        match self {
            TaskItem::Book(item) => item.get_object().is_visible(),
            TaskItem::Texture(item) => item.get_object().is_visible(),
            TaskItem::Coin(item) => item.get_object().is_visible(),
        }
    }

    fn set_drawing_depth(&mut self, depth: i8) {
        match self {
            TaskItem::Book(item) => item.get_object_mut().set_drawing_depth(depth),
            TaskItem::Texture(item) => item.get_object_mut().set_drawing_depth(depth),
            TaskItem::Coin(item) => item.get_object_mut().set_drawing_depth(depth),
        }
    }

    fn get_drawing_depth(&self) -> i8 {
        match self {
            TaskItem::Book(item) => item.get_object().get_drawing_depth(),
            TaskItem::Texture(item) => item.get_object().get_drawing_depth(),
            TaskItem::Coin(item) => item.get_object().get_drawing_depth(),
        }
    }
}

pub struct DeskObjectContainer {
    container: Vec<TaskItem>,
}

impl DeskObjectContainer {
    pub fn new() -> Self {
        DeskObjectContainer {
            container: Vec::new(),
        }
    }

    pub fn add_item(&mut self, obj: TaskItem) {
        self.container.push(obj);
    }

    pub fn sort_with_depth(&mut self) {
        self.container.sort_by(|a: &TaskItem, b: &TaskItem| {
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

    pub fn get_raw_container(&self) -> &Vec<TaskItem> {
        &self.container
    }

    pub fn get_raw_container_mut(&mut self) -> &mut Vec<TaskItem> {
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
            let current_depth = obj.get_drawing_depth();
            let next_depth: i16 = (current_depth as i16) + (offset as i16);

            if next_depth <= 127 && next_depth >= -128 {
                // 範囲内
                obj.set_drawing_depth(next_depth as i8);
            } else if next_depth > 0 {
                // 範囲外（上限）
                obj.set_drawing_depth(127);
            } else {
                // 範囲外（下限）
                obj.set_drawing_depth(-128);
            }
        }
    }
}

pub struct TaskManualBook {
    redraw_request: DrawRequest,
    pages: Vec<Box<dyn DrawableComponent>>,
    go_left_texture: UniTexture,
    go_right_texture: UniTexture,
    background: UniTexture,
    current_page_index: usize,
    canvas: SubScreen,
}

impl TaskManualBook {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, rect: numeric::Rect, depth: i8) -> Self {
        let mut borrowing_flow = Box::new(UniTexture::new(
            ctx.ref_texture(TextureID::ManualPageBorrowingFlow),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
        ));
        borrowing_flow.fit_scale(ctx.context, numeric::Vector2f::new(rect.w, rect.h));

        let mut return_flow = Box::new(UniTexture::new(
            ctx.ref_texture(TextureID::ManualPageReturnFlow),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
        ));
        return_flow.fit_scale(ctx.context, numeric::Vector2f::new(rect.w, rect.h));

        let mut title_details = Box::new(UniTexture::new(
            ctx.ref_texture(TextureID::ManualPageBookTitles),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
        ));
        title_details.fit_scale(ctx.context, numeric::Vector2f::new(rect.w, rect.h));

        let pages = vec![
            borrowing_flow as Box<dyn DrawableComponent>,
            return_flow as Box<dyn DrawableComponent>,
            title_details as Box<dyn DrawableComponent>,
        ];

        let mut left = UniTexture::new(
            ctx.ref_texture(TextureID::GoNextPageLeft),
            numeric::Point2f::new(0.0, rect.h - 48.0),
            numeric::Vector2f::new(0.75, 0.75),
            0.0,
            0,
        );
        left.hide();

        let right = UniTexture::new(
            ctx.ref_texture(TextureID::GoNextPageRight),
            numeric::Point2f::new(rect.w - 48.0, rect.h - 48.0),
            numeric::Vector2f::new(0.75, 0.75),
            0.0,
            0,
        );

        TaskManualBook {
            redraw_request: DrawRequest::InitDraw,
            pages: pages,
            go_left_texture: left,
            go_right_texture: right,
            current_page_index: 0,
            background: UniTexture::new(
                ctx.ref_texture(TextureID::TextBackground),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
            ),
            canvas: SubScreen::new(
                ctx.context,
                rect,
                depth,
                ggraphics::Color::from_rgba_u32(0xff),
            ),
        }
    }

    fn check_move_page_icon_visibility(&mut self) {
        self.go_right_texture.appear();
        self.go_left_texture.appear();

        if self.current_page_index == 0 {
            self.go_left_texture.hide();
        } else if self.current_page_index == self.pages.len() - 1 {
            self.go_right_texture.hide();
        }
    }

    fn go_right<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        if self.current_page_index != self.pages.len() - 1 {
            ctx.process_utility.redraw();
            self.redraw_request = DrawRequest::Draw;
            self.current_page_index += 1;
            self.check_move_page_icon_visibility();
        }
    }

    fn go_left<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        if self.current_page_index != 0 {
            self.redraw_request = DrawRequest::Draw;
            ctx.process_utility.redraw();
            self.current_page_index -= 1;
            self.check_move_page_icon_visibility();
        }
    }

    fn draw_current_page(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if let Some(page) = self.pages.get_mut(self.current_page_index) {
            page.draw(ctx)?;
        }

        Ok(())
    }

    pub fn click_handler<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
    ) -> bool {
        if !self.canvas.contains(point) {
            return false;
        }

        let rpoint = self.canvas.relative_point(point);
        if self.go_left_texture.contains(ctx.context, rpoint) {
            self.go_left(ctx);
        } else if self.go_right_texture.contains(ctx.context, rpoint) {
            self.go_right(ctx);
        }

        true
    }
}

impl DrawableComponent for TaskManualBook {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            if self.redraw_request != DrawRequest::Skip {
                self.redraw_request = DrawRequest::Skip;

                sub_screen::stack_screen(ctx, &self.canvas);

                self.background.draw(ctx)?;

                self.draw_current_page(ctx)?;

                self.go_left_texture.draw(ctx)?;
                self.go_right_texture.draw(ctx)?;

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

impl DrawableObject for TaskManualBook {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for TaskManualBook {
    impl_texture_object_for_wrapped! {canvas}
}
