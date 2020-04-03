use std::rc::Rc;

use ggez::graphics as ggraphics;

use torifune::core::{Clock, Updatable};
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::graphics::object::*;
use torifune::graphics::*;
use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;
use torifune::numeric;
use torifune::roundup2f;

use super::Clickable;
use crate::core::BookInformation;
use crate::core::{FontID, GameData, GensoDate, TextureID};
use crate::flush_delay_event;
use crate::object::effect;
use crate::object::util_object::*;
use crate::scene::*;

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

    pub fn remove_book_info_at(&mut self, index: usize) {
        self.books_info.remove(index);
    }

    pub fn remove_customer_name_at(&mut self, index: usize) {
        self.customers_name.remove(index);
    }

    pub fn remove_date_at(&mut self, index: usize) {
        self.dates.remove(index);
    }
}

pub struct DropDownArea<D>
where
    D: DrawableComponent,
{
    canvas: EffectableWrap<MovableWrap<SubScreen>>,
    drawable: D,
}

impl<D> DropDownArea<D>
where
    D: DrawableComponent,
{
    pub fn new(
        ctx: &mut ggez::Context,
        pos_rect: numeric::Rect,
        drawing_depth: i8,
        drawable: D,
        t: Clock,
    ) -> DropDownArea<D> {
        DropDownArea::<D> {
            canvas: EffectableWrap::new(
                MovableWrap::new(
                    Box::new(SubScreen::new(
                        ctx,
                        pos_rect,
                        drawing_depth,
                        ggraphics::Color::from_rgba_u32(0xffffffff),
                    )),
                    None,
                    t,
                ),
                Vec::new(),
            ),
            drawable: drawable,
        }
    }

    pub fn get_component(&self) -> &D {
        &self.drawable
    }

    pub fn get_component_mut(&mut self) -> &mut D {
        &mut self.drawable
    }
}

impl<D> DrawableComponent for DropDownArea<D>
where
    D: DrawableComponent,
{
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, self.canvas.ref_wrapped_object().ref_wrapped_object());

            self.drawable.draw(ctx)?;

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

impl<D> Updatable for DropDownArea<D>
where
    D: DrawableComponent,
{
    fn update(&mut self, ctx: &ggez::Context, t: Clock) {
        self.canvas.move_with_func(t);
        self.canvas.effect(ctx, t);
    }
}

impl<D> DrawableObject for DropDownArea<D>
where
    D: DrawableComponent,
{
    impl_drawable_object_for_wrapped! {canvas}
}

impl<D> TextureObject for DropDownArea<D>
where
    D: DrawableComponent,
{
    impl_texture_object_for_wrapped! {canvas}
}

impl<D> HasBirthTime for DropDownArea<D>
where
    D: DrawableComponent,
{
    fn get_birth_time(&self) -> Clock {
        self.canvas.get_birth_time()
    }
}

impl<D> MovableObject for DropDownArea<D>
where
    D: DrawableComponent,
{
    // 任意の関数に従って、座標を動かす
    fn move_with_func(&mut self, t: Clock) {
        self.canvas.move_with_func(t);
    }

    // 従う関数を変更する
    fn override_move_func(
        &mut self,
        move_fn: Option<Box<dyn Fn(&dyn MovableObject, Clock) -> numeric::Point2f>>,
        now: Clock,
    ) {
        self.canvas.override_move_func(move_fn, now);
    }

    // 動作する関数が設定された時間を返す
    fn mf_start_timing(&self) -> Clock {
        self.canvas.mf_start_timing()
    }

    // 現在停止しているかを返す
    fn is_stop(&self) -> bool {
        self.canvas.is_stop()
    }
}

impl<D> Effectable for DropDownArea<D>
where
    D: DrawableComponent,
{
    // エフェクト処理を行う
    fn effect(&mut self, ctx: &ggez::Context, t: Clock) {
        self.canvas.effect(ctx, t);
    }
}

impl<D> HasGenericEffect for DropDownArea<D>
where
    D: DrawableComponent,
{
    // 新しくエフェクトを追加するメソッド
    fn add_effect(&mut self, effect: Vec<GenericEffectFn>) {
        self.canvas.add_effect(effect);
    }

    fn clear_effect(&mut self) {
        self.canvas.clear_effect();
    }
}

impl<D> Clickable for DropDownArea<D>
where
    D: Clickable + DrawableComponent,
{
    fn button_down(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        t: Clock,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        let rpoint = self
            .canvas
            .ref_wrapped_object()
            .ref_wrapped_object()
            .relative_point(point);
        self.drawable.button_down(ctx, game_data, t, button, rpoint);
    }

    fn button_up(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        t: Clock,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        let rpoint = self
            .canvas
            .ref_wrapped_object()
            .ref_wrapped_object()
            .relative_point(point);
        self.drawable.button_up(ctx, game_data, t, button, rpoint);
    }

    fn on_click(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        t: Clock,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        let rpoint = self
            .canvas
            .ref_wrapped_object()
            .ref_wrapped_object()
            .relative_point(point);
        self.drawable.on_click(ctx, game_data, t, button, rpoint);
    }

    fn clickable_status(
        &mut self,
        _ctx: &mut ggez::Context,
        _point: numeric::Point2f,
    ) -> ggez::input::mouse::MouseCursor {
        ggez::input::mouse::MouseCursor::Default
    }
}

pub struct BookStatusButtonGroup {
    buttons: Vec<SelectButton>,
    drwob_essential: DrawableObjectEssential,
    last_clicked: Option<usize>,
}

impl BookStatusButtonGroup {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        mut button_rect: numeric::Rect,
        padding: f32,
        mut textures: Vec<TextureID>,
        drawing_depth: i8,
    ) -> Self {
        let mut buttons = Vec::new();

        button_rect.y += padding;

        while textures.len() > 0 {
            button_rect.x += padding;
            let texture = textures.swap_remove(0);

            let mut button_texture = UniTexture::new(
                game_data.ref_texture(texture),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
            );

            button_texture.fit_scale(ctx, numeric::Vector2f::new(button_rect.w, button_rect.h));

            let button = SelectButton::new(ctx, button_rect, Box::new(button_texture));

            buttons.push(button);
            button_rect.x += button_rect.w;
        }

        BookStatusButtonGroup {
            buttons: buttons,
            drwob_essential: DrawableObjectEssential::new(true, drawing_depth),
            last_clicked: None,
        }
    }

    pub fn click_handler(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        for (i, button) in self.buttons.iter().enumerate() {
            if button.contains(ctx, point) {
                self.last_clicked = Some(i);
                break;
            }
        }
    }

    pub fn get_last_clicked(&self) -> Option<usize> {
        self.last_clicked
    }
}

impl DrawableComponent for BookStatusButtonGroup {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            for button in &mut self.buttons {
                button.draw(ctx)?;
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

impl Clickable for BookStatusButtonGroup {
    fn on_click(
        &mut self,
        ctx: &mut ggez::Context,
        _game_data: &GameData,
        _t: Clock,
        _button: ggez::event::MouseButton,
        point: numeric::Point2f,
    ) {
        self.click_handler(ctx, point);
    }
}

pub type BookStatusMenu = DropDownArea<BookStatusButtonGroup>;

pub struct BookTitleMenu {
    raw_data: Vec<BookInformation>,
    title_table_frame: TableFrame,
    title_vtext: Vec<VerticalText>,
    header_text: VerticalText,
    drwob_essential: DrawableObjectEssential,
    last_clicked: Option<usize>,
}

impl BookTitleMenu {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        book_info_data: Vec<BookInformation>,
        drawing_depth: i8,
    ) -> Self {
        let mut title_vtext = Vec::new();

        let font_info = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(32.0, 32.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let title_table_frame = TableFrame::new(
            game_data,
            numeric::Point2f::new(10.0, 10.0),
            FrameData::new(vec![250.0], vec![64.0; book_info_data.len()]),
            numeric::Vector2f::new(0.3, 0.3),
            0,
        );

        for (index, book_info) in book_info_data.iter().enumerate() {
            let title_vtext_line = book_info.name.to_string();
            let mut vtext = VerticalText::new(
                title_vtext_line,
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                drawing_depth,
                font_info,
            );

            vtext.make_center(
                ctx,
                roundup2f!(title_table_frame.get_center_of(
                    numeric::Vector2u::new((book_info_data.len() - index - 1) as u32, 0),
                    title_table_frame.get_position()
                )),
            );

            title_vtext.push(vtext);
        }

        let header_text = VerticalText::new(
            "題目一覧".to_string(),
            numeric::Point2f::new(title_table_frame.real_width() + 20.0, 30.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            font_info,
        );

        BookTitleMenu {
            raw_data: book_info_data,
            title_table_frame: title_table_frame,
            title_vtext: title_vtext,
            drwob_essential: DrawableObjectEssential::new(true, drawing_depth),
            last_clicked: None,
            header_text: header_text,
        }
    }

    pub fn click_handler(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        let maybe_grid_position = self.title_table_frame.get_grid_position(ctx, point);
        if let Some(grid_position) = maybe_grid_position {
            self.last_clicked = Some(grid_position.x as usize);
        }
    }

    pub fn get_last_clicked_book_info(&self) -> Option<BookInformation> {
        if let Some(index) = self.last_clicked {
            Some(self.raw_data.get(index).unwrap().clone())
        } else {
            None
        }
    }

    pub fn get_last_clicked_index(&self) -> Option<usize> {
        self.last_clicked
    }

    pub fn get_title_frame_size(&self) -> numeric::Vector2f {
        self.title_table_frame.size()
    }
}

impl DrawableComponent for BookTitleMenu {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.title_table_frame.draw(ctx)?;

            for vtext in &mut self.title_vtext {
                vtext.draw(ctx)?;
            }

            self.header_text.draw(ctx)?;
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

impl Clickable for BookTitleMenu {
    fn on_click(
        &mut self,
        ctx: &mut ggez::Context,
        _game_data: &GameData,
        _t: Clock,
        _button: ggez::event::MouseButton,
        point: numeric::Point2f,
    ) {
        self.click_handler(ctx, point);
    }
}

pub type BookTitleDropMenu = DropDownArea<BookTitleMenu>;

pub struct CustomerNameMenu {
    raw_data: Vec<String>,
    name_table_frame: TableFrame,
    name_vtext: Vec<VerticalText>,
    header_text: VerticalText,
    drwob_essential: DrawableObjectEssential,
    last_clicked: Option<usize>,
}

impl CustomerNameMenu {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        customer_name_data: Vec<String>,
        drawing_depth: i8,
    ) -> Self {
        let mut title_vtext = Vec::new();

        let font_info = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(32.0, 32.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let name_table_frame = TableFrame::new(
            game_data,
            numeric::Point2f::new(10.0, 10.0),
            FrameData::new(vec![250.0], vec![64.0; customer_name_data.len()]),
            numeric::Vector2f::new(0.3, 0.3),
            0,
        );

        for (index, name) in customer_name_data.iter().enumerate() {
            let name_vtext_line = name.to_string();
            let mut vtext = VerticalText::new(
                name_vtext_line,
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                drawing_depth,
                font_info,
            );

            vtext.make_center(
                ctx,
                roundup2f!(name_table_frame.get_center_of(
                    numeric::Vector2u::new((customer_name_data.len() - index - 1) as u32, 0),
                    name_table_frame.get_position()
                )),
            );

            title_vtext.push(vtext);
        }

        let header_text = VerticalText::new(
            "御客一覧".to_string(),
            numeric::Point2f::new(name_table_frame.real_width() + 20.0, 30.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            font_info,
        );

        CustomerNameMenu {
            raw_data: customer_name_data,
            name_table_frame: name_table_frame,
            name_vtext: title_vtext,
            drwob_essential: DrawableObjectEssential::new(true, drawing_depth),
            last_clicked: None,
            header_text: header_text,
        }
    }

    pub fn click_handler(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        let maybe_grid_position = self.name_table_frame.get_grid_position(ctx, point);
        if let Some(grid_position) = maybe_grid_position {
            self.last_clicked = Some(grid_position.x as usize);
        }
    }

    pub fn get_last_clicked_book_info(&self) -> Option<String> {
        if let Some(index) = self.last_clicked {
            Some(self.raw_data.get(index).unwrap().clone())
        } else {
            None
        }
    }

    pub fn get_last_clicked_index(&self) -> Option<usize> {
        self.last_clicked
    }

    pub fn get_name_frame_size(&self) -> numeric::Vector2f {
        self.name_table_frame.size()
    }
}

impl DrawableComponent for CustomerNameMenu {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.name_table_frame.draw(ctx)?;

            for vtext in &mut self.name_vtext {
                vtext.draw(ctx)?;
            }

            self.header_text.draw(ctx)?;
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

impl Clickable for CustomerNameMenu {
    fn on_click(
        &mut self,
        ctx: &mut ggez::Context,
        _game_data: &GameData,
        _t: Clock,
        _button: ggez::event::MouseButton,
        point: numeric::Point2f,
    ) {
        self.click_handler(ctx, point);
    }
}

pub type CustomerNameDropMenu = DropDownArea<CustomerNameMenu>;

pub struct DateMenu {
    date_data: Vec<GensoDate>,
    date_table_frame: TableFrame,
    date_vtext: Vec<VerticalText>,
    desc_vtext: Vec<VerticalText>,
    header_text: VerticalText,
    drwob_essential: DrawableObjectEssential,
    last_clicked: Option<usize>,
}

impl DateMenu {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        today: GensoDate,
        drawing_depth: i8,
    ) -> Self {
        let mut date_data = Vec::new();
        let mut date_vtext = Vec::new();
        let mut desc_vtext = Vec::new();

        let font_info = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(24.0, 24.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let date_table_frame = TableFrame::new(
            game_data,
            numeric::Point2f::new(10.0, 10.0),
            FrameData::new(vec![140.0, 280.0], vec![64.0; 3]),
            numeric::Vector2f::new(0.3, 0.3),
            0,
        );

        let mut loop_date = today.clone();
        for index in 0..3 {
            let name_vtext_line = loop_date.to_string();
            let mut vtext = VerticalText::new(
                name_vtext_line,
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                drawing_depth,
                font_info,
            );

            vtext.make_center(
                ctx,
                roundup2f!(date_table_frame.get_center_of(
                    numeric::Vector2u::new(index, 1),
                    date_table_frame.get_position()
                )),
            );

            date_vtext.push(vtext);
            date_data.push(loop_date.clone());
            loop_date.add_day(7);
        }

        for (index, s) in vec!["本日", "短期返却日", "長期返却日"].iter().enumerate() {
            let mut vtext = VerticalText::new(
                s.to_string(),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                drawing_depth,
                font_info,
            );

            vtext.make_center(
                ctx,
                roundup2f!(date_table_frame.get_center_of(
                    numeric::Vector2u::new(index as u32, 0),
                    date_table_frame.get_position()
                )),
            );

            desc_vtext.push(vtext);
        }

        let header_text = VerticalText::new(
            "日付情報".to_string(),
            numeric::Point2f::new(date_table_frame.real_width() + 20.0, 30.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            font_info,
        );

        DateMenu {
            date_data: date_data,
            date_table_frame: date_table_frame,
            date_vtext: date_vtext,
            desc_vtext: desc_vtext,
            drwob_essential: DrawableObjectEssential::new(true, drawing_depth),
            last_clicked: None,
            header_text: header_text,
        }
    }

    pub fn click_handler(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        let maybe_grid_position = self.date_table_frame.get_grid_position(ctx, point);
        if let Some(grid_position) = maybe_grid_position {
            self.last_clicked = Some(grid_position.x as usize);
        }
    }

    pub fn get_last_clicked_book_info(&self) -> Option<GensoDate> {
        if let Some(index) = self.last_clicked {
            Some(self.date_data.get(index).unwrap().clone())
        } else {
            None
        }
    }

    pub fn get_last_clicked_index(&self) -> Option<usize> {
        self.last_clicked
    }

    pub fn get_date_frame_size(&self) -> numeric::Vector2f {
        self.date_table_frame.size()
    }
}

impl DrawableComponent for DateMenu {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.date_table_frame.draw(ctx)?;

            for vtext in &mut self.date_vtext {
                vtext.draw(ctx)?;
            }

            for vtext in &mut self.desc_vtext {
                vtext.draw(ctx)?;
            }

            self.header_text.draw(ctx)?;
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

impl Clickable for DateMenu {
    fn on_click(
        &mut self,
        ctx: &mut ggez::Context,
        _game_data: &GameData,
        _t: Clock,
        _button: ggez::event::MouseButton,
        point: numeric::Point2f,
    ) {
        self.click_handler(ctx, point);
    }
}

pub type DateDropMenu = DropDownArea<DateMenu>;


pub struct CustomerQuestionMenu {
    question_table_frame: TableFrame,
    question_vtext: Vec<VerticalText>,
    header_text: VerticalText,
    drwob_essential: DrawableObjectEssential,
    last_clicked: Option<usize>,
}

impl CustomerQuestionMenu {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        drawing_depth: i8,
    ) -> Self {
        let mut question_vtext = Vec::new();

        let font_info = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(26.0, 26.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let question_table_frame = TableFrame::new(
            game_data,
            numeric::Point2f::new(10.0, 10.0),
            FrameData::new(vec![240.0], vec![64.0; 2]),
            numeric::Vector2f::new(0.3, 0.3),
            0,
        );
	
        for (index, s) in vec!["御名前は？", "返却期限は？"].iter().enumerate() {
            let mut vtext = VerticalText::new(
                s.to_string(),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                drawing_depth,
                font_info,
            );

            vtext.make_center(
                ctx,
                roundup2f!(question_table_frame.get_center_of(
                    numeric::Vector2u::new(index as u32, 0),
                    question_table_frame.get_position()
                )),
            );

	    question_vtext.push(vtext);
        }
	
        let header_text = VerticalText::new(
            "質問".to_string(),
            numeric::Point2f::new(question_table_frame.real_width() + 20.0, 30.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            font_info,
        );

        CustomerQuestionMenu {
            question_table_frame: question_table_frame,
            question_vtext: question_vtext,
            drwob_essential: DrawableObjectEssential::new(true, drawing_depth),
            last_clicked: None,
            header_text: header_text,
        }
    }

    pub fn click_handler(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        let maybe_grid_position = self.question_table_frame.get_grid_position(ctx, point);
        if let Some(grid_position) = maybe_grid_position {
            self.last_clicked = Some(grid_position.x as usize);
        }
    }

    pub fn get_last_clicked_index(&self) -> Option<usize> {
        self.last_clicked
    }

    pub fn get_date_frame_size(&self) -> numeric::Vector2f {
        self.question_table_frame.size()
    }
}

impl DrawableComponent for CustomerQuestionMenu {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.question_table_frame.draw(ctx)?;

            for vtext in &mut self.question_vtext {
                vtext.draw(ctx)?;
            }

            self.header_text.draw(ctx)?;
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

impl Clickable for CustomerQuestionMenu {
    fn on_click(
        &mut self,
        ctx: &mut ggez::Context,
        _game_data: &GameData,
        _t: Clock,
        _button: ggez::event::MouseButton,
        point: numeric::Point2f,
    ) {
        self.click_handler(ctx, point);
    }
}

pub type CustomerQuestionDropMenu = DropDownArea<CustomerQuestionMenu>;


pub struct RememberCustomerNameMenu {
    remembered_customer_name: String,
    select_table_frame: TableFrame,
    select_vtext: Vec<VerticalText>,
    drwob_essential: DrawableObjectEssential,
    last_clicked: Option<usize>,
}

impl RememberCustomerNameMenu {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        drawing_depth: i8,
	customer_name: String,
    ) -> Self {
        let mut select_vtext = Vec::new();

        let font_info = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(26.0, 26.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let select_table_frame = TableFrame::new(
            game_data,
            numeric::Point2f::new(10.0, 10.0),
            FrameData::new(vec![240.0], vec![64.0]),
            numeric::Vector2f::new(0.3, 0.3),
            0,
        );
	
        for (index, s) in vec!["名前を記憶する"].iter().enumerate() {
            let mut vtext = VerticalText::new(
                s.to_string(),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                drawing_depth,
                font_info,
            );

            vtext.make_center(
                ctx,
                roundup2f!(select_table_frame.get_center_of(
                    numeric::Vector2u::new(index as u32, 0),
                    select_table_frame.get_position()
                )),
            );

	    select_vtext.push(vtext);
        }
	
        RememberCustomerNameMenu {
            select_table_frame: select_table_frame,
            select_vtext: select_vtext,
            drwob_essential: DrawableObjectEssential::new(true, drawing_depth),
            last_clicked: None,
	    remembered_customer_name: customer_name,
        }
    }

    pub fn click_handler(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        let maybe_grid_position = self.select_table_frame.get_grid_position(ctx, point);
        if let Some(grid_position) = maybe_grid_position {
            self.last_clicked = Some(grid_position.x as usize);
        }
    }

    pub fn get_last_clicked_index(&self) -> Option<usize> {
        self.last_clicked
    }

    pub fn get_date_frame_size(&self) -> numeric::Vector2f {
        self.select_table_frame.size()
    }

    pub fn get_remembered_customer_name(&self) -> String {
	self.remembered_customer_name.clone()
    }
}

impl DrawableComponent for RememberCustomerNameMenu {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.select_table_frame.draw(ctx)?;

            for vtext in &mut self.select_vtext {
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

impl Clickable for RememberCustomerNameMenu {
    fn on_click(
        &mut self,
        ctx: &mut ggez::Context,
        _game_data: &GameData,
        _t: Clock,
        _button: ggez::event::MouseButton,
        point: numeric::Point2f,
    ) {
        self.click_handler(ctx, point);
    }
}

pub type RememberCustomerNameDropMenu = DropDownArea<RememberCustomerNameMenu>;


pub struct OkMenu {
    select_table_frame: TableFrame,
    select_vtext: Vec<VerticalText>,
    drwob_essential: DrawableObjectEssential,
    last_clicked: Option<usize>,
}

impl OkMenu {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        drawing_depth: i8,
    ) -> Self {
        let mut select_vtext = Vec::new();

        let font_info = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(26.0, 26.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let select_table_frame = TableFrame::new(
            game_data,
            numeric::Point2f::new(10.0, 10.0),
            FrameData::new(vec![240.0], vec![64.0]),
            numeric::Vector2f::new(0.3, 0.3),
            0,
        );
	
        for (index, s) in vec!["確認"].iter().enumerate() {
            let mut vtext = VerticalText::new(
                s.to_string(),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                drawing_depth,
                font_info,
            );

            vtext.make_center(
                ctx,
                roundup2f!(select_table_frame.get_center_of(
                    numeric::Vector2u::new(index as u32, 0),
                    select_table_frame.get_position()
                )),
            );

	    select_vtext.push(vtext);
        }
	
        OkMenu {
            select_table_frame: select_table_frame,
            select_vtext: select_vtext,
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

    pub fn get_last_clicked_index(&self) -> Option<usize> {
        self.last_clicked
    }

    pub fn get_date_frame_size(&self) -> numeric::Vector2f {
        self.select_table_frame.size()
    }
}

impl DrawableComponent for OkMenu {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.select_table_frame.draw(ctx)?;

            for vtext in &mut self.select_vtext {
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

impl Clickable for OkMenu {
    fn on_click(
        &mut self,
        ctx: &mut ggez::Context,
        _game_data: &GameData,
        _t: Clock,
        _button: ggez::event::MouseButton,
        point: numeric::Point2f,
    ) {
        self.click_handler(ctx, point);
    }
}

pub type OkDropMenu = DropDownArea<OkMenu>;


pub struct CustomerMenuGroup {
    event_list: DelayEventList<Self>,
    customer_question_menu: Option<EffectableWrap<MovableWrap<CustomerQuestionDropMenu>>>,
    remember_name_menu: Option<EffectableWrap<MovableWrap<RememberCustomerNameDropMenu>>>,
    text_balloon_ok_menu: Option<EffectableWrap<MovableWrap<OkDropMenu>>>,
    drwob_essential: DrawableObjectEssential,
}

impl CustomerMenuGroup {
    pub fn new(drawing_depth: i8) -> Self {
        CustomerMenuGroup {
            event_list: DelayEventList::new(),
	    customer_question_menu: None,
	    remember_name_menu: None,
	    text_balloon_ok_menu: None,
            drwob_essential: DrawableObjectEssential::new(true, drawing_depth),
        }
    }

    pub fn is_some_menu_opened(&self) -> bool {
	self.customer_question_menu.is_some() ||
	    self.remember_name_menu.is_some() ||
	    self.text_balloon_ok_menu.is_some()
    }

    pub fn close_customer_question_menu(&mut self, t: Clock) {
        if let Some(customer_question) = self.customer_question_menu.as_mut() {
            customer_question.add_effect(vec![effect::fade_out(10, t)]);
            self.event_list.add_event(
                Box::new(|slf: &mut CustomerMenuGroup, _, _| slf.customer_question_menu = None),
                t + 11,
            );
        }
    }

    pub fn close_remember_name_menu(&mut self, t: Clock) {
        if let Some(remember_name_menu) = self.remember_name_menu.as_mut() {
            remember_name_menu.add_effect(vec![effect::fade_out(10, t)]);
            self.event_list.add_event(
                Box::new(|slf: &mut CustomerMenuGroup, _, _| slf.remember_name_menu = None),
                t + 11,
            );
        }
    }

    pub fn close_text_balloon_ok_menu(&mut self, t: Clock) {
        if let Some(text_balloon_ok_menu) = self.text_balloon_ok_menu.as_mut() {
            text_balloon_ok_menu.add_effect(vec![effect::fade_out(10, t)]);
            self.event_list.add_event(
                Box::new(|slf: &mut CustomerMenuGroup, _, _| slf.text_balloon_ok_menu = None),
                t + 11,
            );
        }
    }

    pub fn contains_customer_question_menu(
        &self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> bool {
	self.customer_question_menu.is_some()
            && self.customer_question_menu.as_ref().unwrap().contains(ctx, point)
    }

    pub fn contains_remember_name_menu(
        &self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> bool {
	self.remember_name_menu.is_some()
            && self.remember_name_menu.as_ref().unwrap().contains(ctx, point)
    }

    pub fn contains_text_balloon_ok_menu(
        &self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> bool {
	self.text_balloon_ok_menu.is_some()
            && self.text_balloon_ok_menu.as_ref().unwrap().contains(ctx, point)
    }

    pub fn is_contains_any_menus(&self, ctx: &mut ggez::Context, point: numeric::Point2f) -> bool {
	self.contains_customer_question_menu(ctx, point) ||
	    self.contains_remember_name_menu(ctx, point) ||
	    self.contains_text_balloon_ok_menu(ctx, point)
    }

    pub fn get_customer_question_position(&self) -> Option<numeric::Point2f> {
        if let Some(question_menu) = self.customer_question_menu.as_ref() {
            Some(question_menu.get_position())
        } else {
            None
        }
    }

    pub fn get_remember_name_position(&self) -> Option<numeric::Point2f> {
        if let Some(remember_name_menu) = self.remember_name_menu.as_ref() {
            Some(remember_name_menu.get_position())
        } else {
            None
        }
    }

    pub fn get_text_balloon_ok_position(&self) -> Option<numeric::Point2f> {
        if let Some(ok_menu) = self.text_balloon_ok_menu.as_ref() {
            Some(ok_menu.get_position())
        } else {
            None
        }
    }

    ///
    /// メニューのエントリをクリックしていたらtrueを返し、そうでなければfalseを返す
    ///
    pub fn click_customer_question_menu(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        // ボタンエリア内をクリックしていない場合は、即終了
        if !self.contains_customer_question_menu(ctx, point) {
            return false;
        }

        if let Some(customer_question_menu) = self.customer_question_menu.as_mut() {
            customer_question_menu
                .ref_wrapped_object_mut()
                .ref_wrapped_object_mut()
                .on_click(ctx, game_data, t, button, point);
            true
        } else {
            false
        }
    }

    ///
    /// メニューのエントリをクリックしていたらtrueを返し、そうでなければfalseを返す
    ///
    pub fn click_remember_name_menu(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        // ボタンエリア内をクリックしていない場合は、即終了
        if !self.contains_remember_name_menu(ctx, point) {
            return false;
        }

        if let Some(remember_name_menu) = self.remember_name_menu.as_mut() {
            remember_name_menu
                .ref_wrapped_object_mut()
                .ref_wrapped_object_mut()
                .on_click(ctx, game_data, t, button, point);
            true
        } else {
            false
        }
    }

    ///
    /// メニューのエントリをクリックしていたらtrueを返し、そうでなければfalseを返す
    ///
    pub fn click_text_balloon_ok_menu(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        // ボタンエリア内をクリックしていない場合は、即終了
        if !self.contains_text_balloon_ok_menu(ctx, point) {
            return false;
        }

        if let Some(ok_menu) = self.text_balloon_ok_menu.as_mut() {
            ok_menu
                .ref_wrapped_object_mut()
                .ref_wrapped_object_mut()
                .on_click(ctx, game_data, t, button, point);
            true
        } else {
            false
        }
    }
    
    pub fn question_menu_last_clicked_index(&mut self) -> Option<usize> {
        if let Some(customer_question_menu) = self.customer_question_menu.as_mut() {
            customer_question_menu
                .ref_wrapped_object_mut()
                .ref_wrapped_object_mut()
                .get_component()
                .get_last_clicked_index()
        } else {
            None
        }
    }

    pub fn remember_name_clicked_index(&mut self) -> Option<usize> {
        if let Some(remember_name_menu) = self.remember_name_menu.as_mut() {
            remember_name_menu
                .ref_wrapped_object_mut()
                .ref_wrapped_object_mut()
                .get_component()
                .get_last_clicked_index()
        } else {
            None
        }
    }

    pub fn get_remembered_customer_name(&self) -> Option<String> {
	if let Some(remember_menu) = self.remember_name_menu.as_ref() {
	    Some(
		remember_menu
    		    .ref_wrapped_object()
   		    .ref_wrapped_object()
    		    .get_component()
    		    .get_remembered_customer_name()
	    )
	} else {
	    None
	}
    }

    pub fn get_text_balloon_ok_index(&self) -> Option<usize> {
	if let Some(ok_menu) = self.text_balloon_ok_menu.as_ref() {
	    ok_menu
    		.ref_wrapped_object()
   		.ref_wrapped_object()
    		.get_component()
		.get_last_clicked_index()
	} else {
	    None
	}
    }

    pub fn close_all(&mut self, t: Clock) {
	self.close_customer_question_menu(t);
	self.close_remember_name_menu(t);
	self.close_text_balloon_ok_menu(t);
    }

    pub fn show_customer_question_menu(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        position: numeric::Point2f,
        t: Clock,
    ) {
        let question_menu = CustomerQuestionMenu::new(ctx, game_data, 0);

        let frame_size = question_menu.get_date_frame_size();

        let customer_question_menu_area = EffectableWrap::new(
            MovableWrap::new(
                Box::new(DropDownArea::new(
                    ctx,
                    numeric::Rect::new(
                        position.x,
                        position.y,
                        frame_size.x + 128.0,
                        frame_size.y + 64.0,
                    ),
                    0,
                    question_menu,
                    t,
                )),
                None,
                t,
            ),
            vec![effect::fade_in(10, t)],
        );

        self.customer_question_menu = Some(customer_question_menu_area);
    }

    pub fn show_remember_name_menu(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        position: numeric::Point2f,
	customer_name: String,
        t: Clock,
    ) {
	let remember_name_menu = RememberCustomerNameMenu::new(ctx, game_data, 0, customer_name);

        let frame_size = remember_name_menu.get_date_frame_size();

        let remember_name_menu_area = EffectableWrap::new(
            MovableWrap::new(
                Box::new(RememberCustomerNameDropMenu::new(
                    ctx,
                    numeric::Rect::new(
                        position.x,
                        position.y,
                        frame_size.x + 128.0,
                        frame_size.y + 64.0,
                    ),
                    0,
                    remember_name_menu,
                    t,
                )),
                None,
                t,
            ),
            vec![effect::fade_in(10, t)],
        );

        self.remember_name_menu = Some(remember_name_menu_area);
    }

    pub fn show_text_balloon_ok_menu(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        position: numeric::Point2f,
        t: Clock,
    ) {
	let ok_menu = OkMenu::new(ctx, game_data, 0);

        let frame_size = ok_menu.get_date_frame_size();

        let ok_menu_area = EffectableWrap::new(
            MovableWrap::new(
                Box::new(OkDropMenu::new(
                    ctx,
                    numeric::Rect::new(
                        position.x,
                        position.y,
                        frame_size.x + 128.0,
                        frame_size.y + 64.0,
                    ),
                    0,
                    ok_menu,
                    t,
                )),
                None,
                t,
            ),
            vec![effect::fade_in(10, t)],
        );

        self.text_balloon_ok_menu = Some(ok_menu_area);
    }

    pub fn update(&mut self, ctx: &mut ggez::Context, game_data: &GameData, t: Clock) {
        flush_delay_event!(self, self.event_list, ctx, game_data, t);

        if let Some(customer_question_menu) = self.customer_question_menu.as_mut() {
            customer_question_menu.move_with_func(t);
            customer_question_menu.effect(ctx, t);
        }

	if let Some(remember_name_menu) = self.remember_name_menu.as_mut() {
            remember_name_menu.move_with_func(t);
            remember_name_menu.effect(ctx, t);
        }

	if let Some(ok_menu) = self.text_balloon_ok_menu.as_mut() {
            ok_menu.move_with_func(t);
            ok_menu.effect(ctx, t);
        }
    }
}

impl DrawableComponent for CustomerMenuGroup {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            if let Some(customer_question_menu) = self.customer_question_menu.as_mut() {
                customer_question_menu.draw(ctx)?;
            }

	    if let Some(remember_name_menu) = self.remember_name_menu.as_mut() {
                remember_name_menu.draw(ctx)?;
            }

	    if let Some(ok_menu) = self.text_balloon_ok_menu.as_mut() {
                ok_menu.draw(ctx)?;
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

pub struct RecordBookMenuGroup {
    event_list: DelayEventList<Self>,
    book_status_menu: Option<EffectableWrap<MovableWrap<BookStatusMenu>>>,
    book_title_menu: Option<EffectableWrap<MovableWrap<BookTitleDropMenu>>>,
    customer_name_menu: Option<EffectableWrap<MovableWrap<CustomerNameDropMenu>>>,
    date_menu: Option<EffectableWrap<MovableWrap<DateDropMenu>>>,
    drwob_essential: DrawableObjectEssential,
}

impl RecordBookMenuGroup {
    pub fn new(drawing_depth: i8) -> Self {
        RecordBookMenuGroup {
            event_list: DelayEventList::new(),
            book_status_menu: None,
            book_title_menu: None,
            customer_name_menu: None,
            date_menu: None,
            drwob_essential: DrawableObjectEssential::new(true, drawing_depth),
        }
    }

    pub fn is_some_menu_opened(&self) -> bool {
        self.book_status_menu.is_some()
            || self.book_title_menu.is_some()
            || self.customer_name_menu.is_some()
            || self.date_menu.is_some()
    }

    pub fn close_book_status_menu(&mut self, t: Clock) {
        if let Some(button_group) = self.book_status_menu.as_mut() {
            button_group.add_effect(vec![effect::fade_out(10, t)]);
            self.event_list.add_event(
                Box::new(|slf: &mut RecordBookMenuGroup, _, _| slf.book_status_menu = None),
                t + 11,
            );
        }
    }

    pub fn close_book_title_menu(&mut self, t: Clock) {
        if let Some(title_menu) = self.book_title_menu.as_mut() {
            title_menu.add_effect(vec![effect::fade_out(10, t)]);
            self.event_list.add_event(
                Box::new(|slf: &mut RecordBookMenuGroup, _, _| slf.book_title_menu = None),
                t + 11,
            );
        }
    }

    pub fn close_customer_name_menu(&mut self, t: Clock) {
        if let Some(customer_name_menu) = self.customer_name_menu.as_mut() {
            customer_name_menu.add_effect(vec![effect::fade_out(10, t)]);
            self.event_list.add_event(
                Box::new(|slf: &mut RecordBookMenuGroup, _, _| slf.customer_name_menu = None),
                t + 11,
            );
        }
    }

    pub fn close_date_menu(&mut self, t: Clock) {
        if let Some(date_menu) = self.date_menu.as_mut() {
            date_menu.add_effect(vec![effect::fade_out(10, t)]);
            self.event_list.add_event(
                Box::new(|slf: &mut RecordBookMenuGroup, _, _| slf.date_menu = None),
                t + 11,
            );
        }
    }

    pub fn contains_book_status_menu(
        &self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> bool {
        self.book_status_menu.is_some()
            && self.book_status_menu.as_ref().unwrap().contains(ctx, point)
    }

    pub fn contains_book_title_menu(
        &self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> bool {
        self.book_title_menu.is_some()
            && self.book_title_menu.as_ref().unwrap().contains(ctx, point)
    }

    pub fn contains_customer_name_menu(
        &self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> bool {
        self.customer_name_menu.is_some()
            && self
                .customer_name_menu
                .as_ref()
                .unwrap()
                .contains(ctx, point)
    }

    pub fn contains_date_menu(&self, ctx: &mut ggez::Context, point: numeric::Point2f) -> bool {
        self.date_menu.is_some() && self.date_menu.as_ref().unwrap().contains(ctx, point)
    }

    pub fn is_contains_any_menus(&self, ctx: &mut ggez::Context, point: numeric::Point2f) -> bool {
        self.contains_book_title_menu(ctx, point)
            || self.contains_book_status_menu(ctx, point)
            || self.contains_customer_name_menu(ctx, point)
            || self.contains_date_menu(ctx, point)
    }

    pub fn get_book_status_menu_position(&self) -> Option<numeric::Point2f> {
        if let Some(book_status_menu) = self.book_status_menu.as_ref() {
            Some(book_status_menu.get_position())
        } else {
            None
        }
    }

    pub fn get_book_title_menu_position(&self) -> Option<numeric::Point2f> {
        if let Some(book_title_menu) = self.book_title_menu.as_ref() {
            Some(book_title_menu.get_position())
        } else {
            None
        }
    }

    pub fn get_customer_name_menu_position(&self) -> Option<numeric::Point2f> {
        if let Some(customer_name_menu) = self.customer_name_menu.as_ref() {
            Some(customer_name_menu.get_position())
        } else {
            None
        }
    }

    pub fn get_date_menu_position(&self) -> Option<numeric::Point2f> {
        if let Some(date_menu) = self.date_menu.as_ref() {
            Some(date_menu.get_position())
        } else {
            None
        }
    }

    ///
    /// メニューのエントリをクリックしていたらtrueを返し、そうでなければfalseを返す
    ///
    pub fn click_book_status_menu(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        // ボタンエリア内をクリックしていない場合は、即終了
        if !self.contains_book_status_menu(ctx, point) {
            return false;
        }

        if let Some(book_status_menu) = self.book_status_menu.as_mut() {
            book_status_menu
                .ref_wrapped_object_mut()
                .ref_wrapped_object_mut()
                .on_click(ctx, game_data, t, button, point);
            true
        } else {
            false
        }
    }

    ///
    /// メニューのエントリをクリックしていたらtrueを返し、そうでなければfalseを返す
    ///
    pub fn click_book_title_menu(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        // ボタンエリア内をクリックしていない場合は、即終了
        if !self.contains_book_title_menu(ctx, point) {
            return false;
        }

        if let Some(book_title_menu) = self.book_title_menu.as_mut() {
            book_title_menu
                .ref_wrapped_object_mut()
                .ref_wrapped_object_mut()
                .on_click(ctx, game_data, t, button, point);
            true
        } else {
            false
        }
    }

    ///
    /// メニューのエントリをクリックしていたらtrueを返し、そうでなければfalseを返す
    ///
    pub fn click_customer_name_menu(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        // ボタンエリア内をクリックしていない場合は、即終了
        if !self.contains_customer_name_menu(ctx, point) {
            return false;
        }

        if let Some(customer_name_menu) = self.customer_name_menu.as_mut() {
            customer_name_menu
                .ref_wrapped_object_mut()
                .ref_wrapped_object_mut()
                .on_click(ctx, game_data, t, button, point);
            true
        } else {
            false
        }
    }

    ///
    /// メニューのエントリをクリックしていたらtrueを返し、そうでなければfalseを返す
    ///
    pub fn click_date_menu(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        // ボタンエリア内をクリックしていない場合は、即終了
        if !self.contains_date_menu(ctx, point) {
            return false;
        }

        if let Some(date_menu) = self.date_menu.as_mut() {
            date_menu
                .ref_wrapped_object_mut()
                .ref_wrapped_object_mut()
                .on_click(ctx, game_data, t, button, point);
            true
        } else {
            false
        }
    }

    pub fn book_status_menu_last_clicked(&mut self) -> Option<usize> {
        if let Some(book_status_menu) = self.book_status_menu.as_mut() {
            book_status_menu
                .ref_wrapped_object_mut()
                .ref_wrapped_object_mut()
                .get_component()
                .get_last_clicked()
        } else {
            None
        }
    }

    pub fn book_title_menu_last_clicked(&mut self) -> Option<(usize, BookInformation)> {
        if let Some(book_title_menu) = self.book_title_menu.as_mut() {
            let component = book_title_menu
                .ref_wrapped_object_mut()
                .ref_wrapped_object_mut()
                .get_component();
            if let Some(index) = component.get_last_clicked_index() {
                Some((index, component.get_last_clicked_book_info().unwrap()))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn date_menu_last_clicked(&mut self) -> Option<(usize, GensoDate)> {
        if let Some(date_menu) = self.date_menu.as_mut() {
            let component = date_menu
                .ref_wrapped_object_mut()
                .ref_wrapped_object_mut()
                .get_component();
            if let Some(index) = component.get_last_clicked_index() {
                Some((index, component.get_last_clicked_book_info().unwrap()))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn close_all(&mut self, t: Clock) {
        self.close_book_status_menu(t);
        self.close_book_title_menu(t);
        self.close_customer_name_menu(t);
        self.close_date_menu(t);
    }

    pub fn show_book_status_menu(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        position: numeric::Point2f,
        t: Clock,
    ) {
        let button_group = BookStatusButtonGroup::new(
            ctx,
            game_data,
            numeric::Rect::new(0.0, 0.0, 70.0, 70.0),
            20.0,
            vec![
                TextureID::ChoicePanel1,
                TextureID::ChoicePanel2,
                TextureID::ChoicePanel3,
            ],
            0,
        );

        let button_group_area = EffectableWrap::new(
            MovableWrap::new(
                Box::new(DropDownArea::new(
                    ctx,
                    numeric::Rect::new(position.x, position.y, 290.0, 220.0),
                    0,
                    button_group,
                    t,
                )),
                None,
                t,
            ),
            vec![effect::fade_in(10, t)],
        );
        self.book_status_menu = Some(button_group_area);
    }

    pub fn show_book_title_menu(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        position: numeric::Point2f,
        kosuzu_memory: &KosuzuMemory,
        t: Clock,
    ) {
        let book_title_menu =
            BookTitleMenu::new(ctx, game_data, kosuzu_memory.books_info.clone(), 0);

        let frame_size = book_title_menu.get_title_frame_size();

        let book_title_menu_area = EffectableWrap::new(
            MovableWrap::new(
                Box::new(DropDownArea::new(
                    ctx,
                    numeric::Rect::new(
                        position.x,
                        position.y,
                        frame_size.x + 128.0,
                        frame_size.y + 64.0,
                    ),
                    0,
                    book_title_menu,
                    t,
                )),
                None,
                t,
            ),
            vec![effect::fade_in(10, t)],
        );

        self.book_title_menu = Some(book_title_menu_area);
    }

    pub fn show_customer_name_menu(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        position: numeric::Point2f,
        kosuzu_memory: &KosuzuMemory,
        t: Clock,
    ) {
        let customer_name_menu =
            CustomerNameMenu::new(ctx, game_data, kosuzu_memory.customers_name.clone(), 0);

        let frame_size = customer_name_menu.get_name_frame_size();

        let customer_name_menu_area = EffectableWrap::new(
            MovableWrap::new(
                Box::new(DropDownArea::new(
                    ctx,
                    numeric::Rect::new(
                        position.x,
                        position.y,
                        frame_size.x + 128.0,
                        frame_size.y + 64.0,
                    ),
                    0,
                    customer_name_menu,
                    t,
                )),
                None,
                t,
            ),
            vec![effect::fade_in(10, t)],
        );

        self.customer_name_menu = Some(customer_name_menu_area);
    }

    pub fn show_date_menu(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        position: numeric::Point2f,
        today: GensoDate,
        t: Clock,
    ) {
        let date_menu = DateMenu::new(ctx, game_data, today, 0);

        let frame_size = date_menu.get_date_frame_size();

        let date_menu_area = EffectableWrap::new(
            MovableWrap::new(
                Box::new(DropDownArea::new(
                    ctx,
                    numeric::Rect::new(
                        position.x,
                        position.y,
                        frame_size.x + 128.0,
                        frame_size.y + 64.0,
                    ),
                    0,
                    date_menu,
                    t,
                )),
                None,
                t,
            ),
            vec![effect::fade_in(10, t)],
        );

        self.date_menu = Some(date_menu_area);
    }

    pub fn update(&mut self, ctx: &mut ggez::Context, game_data: &GameData, t: Clock) {
        flush_delay_event!(self, self.event_list, ctx, game_data, t);

        if let Some(book_status_menu) = self.book_status_menu.as_mut() {
            book_status_menu.move_with_func(t);
            book_status_menu.effect(ctx, t);
        }

        if let Some(book_title_menu) = self.book_title_menu.as_mut() {
            book_title_menu.move_with_func(t);
            book_title_menu.effect(ctx, t);
        }

        if let Some(customer_name_menu) = self.customer_name_menu.as_mut() {
            customer_name_menu.move_with_func(t);
            customer_name_menu.effect(ctx, t);
        }

        if let Some(date_menu) = self.date_menu.as_mut() {
            date_menu.move_with_func(t);
            date_menu.effect(ctx, t);
        }
    }
}

impl DrawableComponent for RecordBookMenuGroup {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            if let Some(book_status_menu) = self.book_status_menu.as_mut() {
                book_status_menu.draw(ctx)?;
            }

            if let Some(book_title_menu) = self.book_title_menu.as_mut() {
                book_title_menu.draw(ctx)?;
            }

            if let Some(customer_name_menu) = self.customer_name_menu.as_mut() {
                customer_name_menu.draw(ctx)?;
            }

            if let Some(date_menu) = self.date_menu.as_mut() {
                date_menu.draw(ctx)?;
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
