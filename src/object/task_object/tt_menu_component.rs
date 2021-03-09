use ggez::graphics as ggraphics;

use torifune::core::{Clock, Updatable};
use torifune::graphics::drawable::*;
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::graphics::object::*;
use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;
use torifune::numeric;
use torifune::roundup2f;

use super::Clickable;
use crate::core;
use crate::core::*;
use crate::core::{FontID, GensoDate, TextureID, TileBatchTextureID};
use crate::flush_delay_event;
use crate::flush_delay_event_and_redraw_check;
use crate::object::effect;
use crate::object::util_object::*;
use crate::scene::*;
use crate::set_table_frame_cell_center;

pub struct KosuzuMemory {
    remembered_book_info: Vec<BookInformation>,
    borrowing_written_book: Vec<BookInformation>,
    customers_name: Vec<String>,
    dates: Vec<GensoDate>,
}

impl KosuzuMemory {
    pub fn new() -> Self {
        KosuzuMemory {
            remembered_book_info: Vec::new(),
            borrowing_written_book: Vec::new(),
            customers_name: Vec::new(),
            dates: Vec::new(),
        }
    }

    pub fn add_book_info(&mut self, book_info: BookInformation) {
        if self.is_written_in_record(&book_info) {
            return;
        }

        if self
            .remembered_book_info
            .iter()
            .any(|info| info.name == book_info.name)
        {
            return;
        }

        self.remembered_book_info.push(book_info);
    }

    pub fn is_written_in_record(&self, book_info: &BookInformation) -> bool {
        self.borrowing_written_book.contains(book_info)
    }
    
    pub fn add_customer_name(&mut self, name: String) {
        if self.customers_name.contains(&name) {
            return;
        }
        self.customers_name.push(name);
    }

    pub fn add_date(&mut self, date: GensoDate) {
        self.dates.push(date);
    }

    pub fn add_book_to_written_list(&mut self, book_info: BookInformation) {
        self.borrowing_written_book.push(book_info);
    }

    pub fn get_book_info_remove(&mut self, index: usize) -> Option<BookInformation> {
        if self.remembered_book_info.len() <= index {
            None
        } else {
            Some(self.remembered_book_info.swap_remove(index))
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
        self.remembered_book_info.remove(index);
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
    background: UniTexture,
    apperance_frame: TileBatchFrame,
    canvas: EffectableWrap<MovableWrap<SubScreen>>,
    click_position: numeric::Point2f,
    drawable: D,
}

impl<D> DropDownArea<D>
where
    D: DrawableComponent,
{
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        click_position: numeric::Point2f,
        pos_rect: numeric::Rect,
        drawing_depth: i8,
        drawable: D,
        t: Clock,
    ) -> DropDownArea<D> {
        let background = UniTexture::new(
            ctx.ref_texture(TextureID::TextBackground),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
        );

        let appr_frame = TileBatchFrame::new(
            ctx.resource,
            TileBatchTextureID::TaishoStyle1,
            numeric::Rect::new(0.0, 0.0, pos_rect.w, pos_rect.h),
            numeric::Vector2f::new(0.4, 0.4),
            0,
        );

        DropDownArea::<D> {
            background: background,
            apperance_frame: appr_frame,
            canvas: EffectableWrap::new(
                MovableWrap::new(
                    Box::new(SubScreen::new(
                        ctx.context,
                        pos_rect,
                        drawing_depth,
                        ggraphics::Color::from_rgba_u32(0xffffffff),
                    )),
                    None,
                    t,
                ),
                Vec::new(),
            ),
            click_position: click_position,
            drawable: drawable,
        }
    }

    pub fn get_component(&self) -> &D {
        &self.drawable
    }

    pub fn get_component_mut(&mut self) -> &mut D {
        &mut self.drawable
    }

    pub fn get_click_position(&self) -> numeric::Point2f {
        self.click_position
    }
}

impl<D> DrawableComponent for DropDownArea<D>
where
    D: DrawableComponent,
{
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.background.draw(ctx)?;
            self.apperance_frame.draw(ctx)?;

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
    fn update(&mut self, ctx: &mut ggez::Context, t: Clock) {
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
    fn override_move_func(&mut self, move_fn: Option<GenericMoveFn>, now: Clock) {
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
    fn effect(&mut self, ctx: &mut ggez::Context, t: Clock) {
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

    fn is_empty_effect(&self) -> bool {
        self.canvas.is_empty_effect()
    }
}

impl<D> Clickable for DropDownArea<D>
where
    D: Clickable + DrawableComponent,
{
    fn button_down<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        t: Clock,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        let rpoint = self.canvas.relative_point(point);
        self.drawable.button_down(ctx, t, button, rpoint);
    }

    fn button_up<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        t: Clock,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        let rpoint = self.canvas.relative_point(point);
        self.drawable.button_up(ctx, t, button, rpoint);
    }

    fn on_click<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        t: Clock,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        let rpoint = self.canvas.relative_point(point);
        self.drawable.on_click(ctx, t, button, rpoint);
    }

    fn clickable_status(
        &mut self,
        _ctx: &mut ggez::Context,
        _point: numeric::Point2f,
    ) -> ggez::input::mouse::CursorIcon {
        ggez::input::mouse::CursorIcon::Default
    }
}

pub struct BookStatusButtonGroup {
    buttons: Vec<SelectButton>,
    drwob_essential: DrawableObjectEssential,
    last_clicked: Option<usize>,
}

impl BookStatusButtonGroup {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        mut button_rect: numeric::Rect,
        padding: f32,
        drawing_depth: i8,
    ) -> Self {
        let mut buttons = Vec::new();

        button_rect.y += padding * 1.5;

        for text_str in vec!["良", "可", "悪"] {
            button_rect.x += padding;

            let font_info = FontInformation::new(
                ctx.resource.get_font(FontID::Cinema),
                numeric::Vector2f::new(42.0, 42.0),
                ggraphics::Color::from_rgba_u32(0xff),
            );

            let text_texture = TextButtonTexture::new(
                ctx,
                numeric::Point2f::new(0.0, 0.0),
                text_str.to_string(),
                font_info,
                10.0,
                ggraphics::Color::from_rgba_u32(0xe8b5a2ff),
                0,
            );

            let button = SelectButton::new(ctx, button_rect, Box::new(text_texture));

            buttons.push(button);
            button_rect.x += button_rect.w;
        }

        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(36.0, 36.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let remove_texture = TextButtonTexture::new(
            ctx,
            numeric::Point2f::new(0.0, 0.0),
            "削除".to_string(),
            font_info,
            10.0,
            ggraphics::Color::from_rgba_u32(0xe8b5a2ff),
            0,
        );

        let remove_button = SelectButton::new(
            ctx,
            numeric::Rect::new(95.0, 110.0, 100.0, 50.0),
            Box::new(remove_texture),
        );
        buttons.push(remove_button);

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
    fn on_click<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        _t: Clock,
        _button: ggez::event::MouseButton,
        point: numeric::Point2f,
    ) {
        self.click_handler(ctx.context, point);
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
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        book_info_data: Vec<BookInformation>,
        drawing_depth: i8,
    ) -> Self {
        let mut title_vtext = Vec::new();

        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::JpFude1),
            numeric::Vector2f::new(32.0, 32.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let title_table_frame = TableFrame::new(
            ctx.resource,
            numeric::Point2f::new(48.0, 32.0),
            TileBatchTextureID::OldStyleFrame,
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

            set_table_frame_cell_center!(
                ctx.context,
                title_table_frame,
                vtext,
                numeric::Vector2u::new(index as u32, 0)
            );

            title_vtext.push(vtext);
        }

        let header_text = VerticalText::new(
            "題目一覧".to_string(),
            numeric::Point2f::new(title_table_frame.real_width() + 70.0, 30.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
		ctx.resource.get_font(FontID::Cinema),
		numeric::Vector2f::new(32.0, 32.0),
		ggraphics::Color::from_rgba_u32(0xff),
            ),
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
    fn on_click<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        _t: Clock,
        _button: ggez::event::MouseButton,
        point: numeric::Point2f,
    ) {
        self.click_handler(ctx.context, point);
    }
}

pub type BookTitleDropMenu = DropDownArea<BookTitleMenu>;

pub struct SimpleMessageMenu {
    title: VerticalText,
    message: VerticalText,
    menu_size: numeric::Vector2f,
    drwob_essential: DrawableObjectEssential,
}

impl SimpleMessageMenu {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        title: String,
        message: String,
        title_font_scale: f32,
        msg_font_scale: f32,
        drawing_depth: i8,
    ) -> Self {
        let menu_size = numeric::Vector2f::new(
            msg_font_scale + title_font_scale + 120.0,
            util::max(
                title.len() as f32 * title_font_scale,
                message.len() as f32 * msg_font_scale,
            ) / 2.0,
        );

        let title_font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(title_font_scale, title_font_scale),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let msg_font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(msg_font_scale, msg_font_scale),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let title_text = VerticalText::new(
            title,
            numeric::Point2f::new(menu_size.x - title_font_scale - 40.0, 35.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            title_font_info,
        );

        let mut msg_text = VerticalText::new(
            message,
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            msg_font_info,
        );

        msg_text.make_center(
            ctx.context,
            numeric::Point2f::new(menu_size.x / 2.0, menu_size.y / 2.0),
        );

        SimpleMessageMenu {
            title: title_text,
            message: msg_text,
            menu_size: menu_size,
            drwob_essential: DrawableObjectEssential::new(true, drawing_depth),
        }
    }

    pub fn get_size(&self) -> numeric::Vector2f {
        self.menu_size
    }
}

impl DrawableComponent for SimpleMessageMenu {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.message.draw(ctx)?;
            self.title.draw(ctx)?;
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

impl Clickable for SimpleMessageMenu {
    fn on_click<'a>(
        &mut self,
        _: &mut SuzuContext<'a>,
        _t: Clock,
        _button: ggez::event::MouseButton,
        _: numeric::Point2f,
    ) {
    }
}

pub type SimpleMessageDropMenu = DropDownArea<SimpleMessageMenu>;

pub struct CustomerNameMenu {
    raw_data: Vec<String>,
    name_table_frame: TableFrame,
    name_vtext: Vec<VerticalText>,
    header_text: UniText,
    drwob_essential: DrawableObjectEssential,
    last_clicked: Option<usize>,
}

impl CustomerNameMenu {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        customer_name_data: Vec<String>,
        drawing_depth: i8,
    ) -> Self {
        let mut title_vtext = Vec::new();

        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(24.0, 24.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let name_table_frame = TableFrame::new(
            ctx.resource,
            numeric::Point2f::new(48.0, 20.0),
            TileBatchTextureID::OldStyleFrame,
            FrameData::new(vec![250.0], vec![56.0; customer_name_data.len()]),
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

            set_table_frame_cell_center!(
                ctx.context,
                name_table_frame,
                vtext,
                numeric::Vector2u::new((customer_name_data.len() - index - 1) as u32, 0)
            );

            title_vtext.push(vtext);
        }

        let header_text = UniText::new(
            "御客一覧".to_string(),
            numeric::Point2f::new(0.0, 0.0),
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

    pub fn set_to_center<'a>(&mut self, ctx: &mut SuzuContext<'a>, center: numeric::Point2f) {
        self.name_table_frame
            .make_center(numeric::Point2f::new(center.x, center.y + 15.0));
        self.header_text
            .make_center(ctx.context, numeric::Point2f::new(center.x, 35.0));

        for (index, text) in self.name_vtext.iter_mut().enumerate() {
            set_table_frame_cell_center!(
                ctx.context,
                self.name_table_frame,
                text,
                numeric::Vector2u::new((self.raw_data.len() - index - 1) as u32, 0)
            );
        }
    }

    pub fn click_handler(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        let maybe_grid_position = self.name_table_frame.get_grid_position(ctx, point);
        if let Some(grid_position) = maybe_grid_position {
            self.last_clicked = Some(grid_position.x as usize);
        }
    }

    pub fn get_last_clicked_customer_name(&self) -> Option<String> {
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
    fn on_click<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        _t: Clock,
        _button: ggez::event::MouseButton,
        point: numeric::Point2f,
    ) {
        self.click_handler(ctx.context, point);
    }
}

pub type CustomerNameDropMenu = DropDownArea<CustomerNameMenu>;

pub struct DateMenu {
    date_data: Vec<GensoDate>,
    date_table_frame: TableFrame,
    date_vtext: Vec<VerticalText>,
    desc_vtext: Vec<VerticalText>,
    header_text: UniText,
    drwob_essential: DrawableObjectEssential,
    last_clicked: Option<usize>,
}

impl DateMenu {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, today: GensoDate, drawing_depth: i8) -> Self {
        let mut date_data = Vec::new();
        let mut date_vtext = Vec::new();
        let mut desc_vtext = Vec::new();

        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(24.0, 24.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let date_table_frame = TableFrame::new(
            ctx.resource,
            numeric::Point2f::new(48.0, 50.0),
            TileBatchTextureID::OldStyleFrame,
            FrameData::new(vec![110.0, 256.0], vec![56.0; 3]),
            numeric::Vector2f::new(0.3, 0.3),
            0,
        );

        let mut loop_date = today.clone();
        for index in 0..3 {
            let name_vtext_line = loop_date.to_string();
            let mut vtext = VerticalText::new(
                name_vtext_line,
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(0.85, 0.85),
                0.0,
                drawing_depth,
                font_info,
            );

            set_table_frame_cell_center!(
                ctx.context,
                date_table_frame,
                vtext,
                numeric::Vector2u::new(index, 1)
            );

            date_vtext.push(vtext);
            date_data.push(loop_date.clone());
            loop_date.add_day(7);
        }

        for (index, s) in vec!["本日", "短期返却日", "長期返却日"].iter().enumerate() {
            let mut vtext = VerticalText::new(
                s.to_string(),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(0.85, 0.85),
                0.0,
                drawing_depth,
                font_info,
            );

            set_table_frame_cell_center!(
                ctx.context,
                date_table_frame,
                vtext,
                numeric::Vector2u::new(index as u32, 0)
            );

            desc_vtext.push(vtext);
        }

        let mut header_text = UniText::new(
            "日付情報".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(0.85, 0.85),
            0.0,
            0,
            font_info,
        );

        header_text.make_center(
            ctx.context,
            numeric::Point2f::new((date_table_frame.size().x / 2.0) + 40.0, 35.0),
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

    pub fn set_to_center<'a>(&mut self, ctx: &mut SuzuContext<'a>, center: numeric::Point2f) {
        self.date_table_frame
            .make_center(numeric::Point2f::new(center.x, center.y + 15.0));
        self.header_text
            .make_center(ctx.context, numeric::Point2f::new(center.x, 35.0));

        for (index, text) in self.date_vtext.iter_mut().enumerate() {
            set_table_frame_cell_center!(
                ctx.context,
                self.date_table_frame,
                text,
                numeric::Vector2u::new(index as u32, 1)
            );
        }

        for (index, text) in self.desc_vtext.iter_mut().enumerate() {
            set_table_frame_cell_center!(
                ctx.context,
                self.date_table_frame,
                text,
                numeric::Vector2u::new(index as u32, 0)
            );
        }
    }

    pub fn click_handler(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        let maybe_grid_position = self.date_table_frame.get_grid_position(ctx, point);
        if let Some(grid_position) = maybe_grid_position {
            self.last_clicked = Some(grid_position.x as usize);
        }
    }

    pub fn get_last_clicked_genso_date(&self) -> Option<GensoDate> {
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
    fn on_click<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        _t: Clock,
        _button: ggez::event::MouseButton,
        point: numeric::Point2f,
    ) {
        self.click_handler(ctx.context, point);
    }
}

pub type DateDropMenu = DropDownArea<DateMenu>;

pub struct DateCheckMenu {
    date_data: Vec<GensoDate>,
    date_table_frame: TableFrame,
    desc_vtext: Vec<VerticalText>,
    date_vtext: Vec<VerticalText>,
    date_check_button: FramedButton,
    drwob_essential: DrawableObjectEssential,
    check_button_clicked: bool,
}

impl DateCheckMenu {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, today: GensoDate, return_date: GensoDate, drawing_depth: i8) -> Self {
	let mut desc_vtext = Vec::new();
	let mut date_data = Vec::new();
	let mut date_vtext = Vec::new();
	
	let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(20.0, 20.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let date_table_frame = TableFrame::new(
            ctx.resource,
            numeric::Point2f::new(48.0, 50.0),
            TileBatchTextureID::OldStyleFrame,
            FrameData::new(vec![110.0, 256.0], vec![56.0; 2]),
            numeric::Vector2f::new(0.3, 0.3),
            0,
        );

        for (index, s) in vec!["本日", "返却日"].iter().enumerate() {
            let mut vtext = VerticalText::new(
                s.to_string(),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                drawing_depth,
                font_info,
            );

            set_table_frame_cell_center!(
                ctx.context,
                date_table_frame,
                vtext,
                numeric::Vector2u::new(index as u32, 0)
            );

            desc_vtext.push(vtext);
        }

	for (index, date) in vec![today.clone(), return_date].iter().enumerate() {
            let name_vtext_line = date.to_string();
            let mut vtext = VerticalText::new(
                name_vtext_line,
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                drawing_depth,
                font_info,
            );

            set_table_frame_cell_center!(
                ctx.context,
                date_table_frame,
                vtext,
                numeric::Vector2u::new(index as u32, 1)
            );

            date_vtext.push(vtext);
            date_data.push(date.clone());
        }

	let area = date_table_frame.get_area();
	let date_check_button = FramedButton::create_design1(
	    ctx,
	    numeric::Point2f::new(area.x + 10.0, area.bottom() + 20.0),
	    "指摘",
	    numeric::Vector2f::new(24.0, 24.0)
	);

	DateCheckMenu {
	    date_data: date_data,
	    date_table_frame: date_table_frame,
	    date_vtext: date_vtext,
	    desc_vtext: desc_vtext,
	    date_check_button: date_check_button,
	    drwob_essential: DrawableObjectEssential::new(true, drawing_depth),
	    check_button_clicked: false,
	}
    }

    pub fn set_to_center<'a>(&mut self, ctx: &mut SuzuContext<'a>, center: numeric::Point2f) {
	self.date_table_frame
            .make_center(numeric::Point2f::new(center.x, center.y - 30.0));

        for (index, text) in self.date_vtext.iter_mut().enumerate() {
            set_table_frame_cell_center!(
                ctx.context,
                self.date_table_frame,
                text,
                numeric::Vector2u::new(index as u32, 1)
            );
        }

        for (index, text) in self.desc_vtext.iter_mut().enumerate() {
            set_table_frame_cell_center!(
                ctx.context,
                self.date_table_frame,
                text,
                numeric::Vector2u::new(index as u32, 0)
            );
        }
    }

    pub fn click_handler(&mut self, _ctx: &mut ggez::Context, point: numeric::Point2f) {
	if self.date_check_button.contains(point) {
	    self.check_button_clicked = true;
	}
    }

    pub fn check_button_is_clicked_volatile(&mut self) -> bool {
	let ret = self.check_button_clicked;
	self.check_button_clicked = false;
	ret
    }

    pub fn get_date_frame_size(&self) -> numeric::Vector2f {
	self.date_table_frame.size() + numeric::Vector2f::new(0.0, 50.0)
    }
}

impl DrawableComponent for DateCheckMenu {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
	    self.date_table_frame.draw(ctx)?;

            for vtext in &mut self.date_vtext {
                vtext.draw(ctx)?;
            }

            for vtext in &mut self.desc_vtext {
                vtext.draw(ctx)?;
            }
	    
	    self.date_check_button.draw(ctx)?;
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

impl Clickable for DateCheckMenu {
    fn on_click<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        _t: Clock,
        _button: ggez::event::MouseButton,
        point: numeric::Point2f,
    ) {
        self.click_handler(ctx.context, point);
    }
}

pub type DateCheckDropMenu = DropDownArea<DateCheckMenu>;

pub struct CustomerQuestionMenu {
    question_table_frame: TableFrame,
    question_vtext: Vec<VerticalText>,
    header_text: UniText,
    drwob_essential: DrawableObjectEssential,
    last_clicked: Option<usize>,
}

impl CustomerQuestionMenu {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, drawing_depth: i8) -> Self {
        let mut question_vtext = Vec::new();

        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(26.0, 26.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let question_table_frame = TableFrame::new(
            ctx.resource,
            numeric::Point2f::new(10.0, 10.0),
            TileBatchTextureID::OldStyleFrame,
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

            set_table_frame_cell_center!(
                ctx.context,
                question_table_frame,
                vtext,
                numeric::Vector2u::new(index as u32, 0)
            );

            question_vtext.push(vtext);
        }

        let header_text = UniText::new(
            "質問".to_string(),
            numeric::Point2f::new(0.0, 0.0),
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

    pub fn set_to_center<'a>(&mut self, ctx: &mut SuzuContext<'a>, center: numeric::Point2f) {
        self.header_text
            .make_center(ctx.context, numeric::Point2f::new(center.x, 40.0));
        self.question_table_frame
            .make_center(numeric::Point2f::new(center.x, center.y + 20.0));
        for (index, text) in self.question_vtext.iter_mut().enumerate() {
            set_table_frame_cell_center!(
                ctx.context,
                self.question_table_frame,
                text,
                numeric::Vector2u::new(index as u32, 0)
            );
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
    fn on_click<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        _t: Clock,
        _button: ggez::event::MouseButton,
        point: numeric::Point2f,
    ) {
        self.click_handler(ctx.context, point);
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
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, drawing_depth: i8, customer_name: String) -> Self {
        let mut select_vtext = Vec::new();

        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(26.0, 26.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let select_table_frame = TableFrame::new(
            ctx.resource,
            numeric::Point2f::new(10.0, 10.0),
            TileBatchTextureID::OldStyleFrame,
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

            set_table_frame_cell_center!(
                ctx.context,
                select_table_frame,
                vtext,
                numeric::Vector2u::new(index as u32, 0)
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
    fn on_click<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        _t: Clock,
        _button: ggez::event::MouseButton,
        point: numeric::Point2f,
    ) {
        self.click_handler(ctx.context, point);
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
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, drawing_depth: i8) -> Self {
        let mut select_vtext = Vec::new();

        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(26.0, 26.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let select_table_frame = TableFrame::new(
            ctx.resource,
            numeric::Point2f::new(10.0, 10.0),
            TileBatchTextureID::OldStyleFrame,
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

            set_table_frame_cell_center!(
                ctx.context,
                select_table_frame,
                vtext,
                numeric::Vector2u::new(index as u32, 0)
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
    fn on_click<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        _t: Clock,
        _button: ggez::event::MouseButton,
        point: numeric::Point2f,
    ) {
        self.click_handler(ctx.context, point);
    }
}

pub type OkDropMenu = DropDownArea<OkMenu>;

pub struct CustomerMenuGroup {
    event_list: DelayEventList<Self>,
    customer_question_menu: Option<CustomerQuestionDropMenu>,
    text_balloon_ok_menu: Option<OkDropMenu>,
    drwob_essential: DrawableObjectEssential,
}

impl CustomerMenuGroup {
    pub fn new(drawing_depth: i8) -> Self {
        CustomerMenuGroup {
            event_list: DelayEventList::new(),
            customer_question_menu: None,
            text_balloon_ok_menu: None,
            drwob_essential: DrawableObjectEssential::new(true, drawing_depth),
        }
    }

    pub fn is_some_menu_opened(&self) -> bool {
        self.customer_question_menu.is_some() || self.text_balloon_ok_menu.is_some()
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
            && self
                .customer_question_menu
                .as_ref()
                .unwrap()
                .contains(ctx, point)
    }

    pub fn contains_text_balloon_ok_menu(
        &self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> bool {
        self.text_balloon_ok_menu.is_some()
            && self
                .text_balloon_ok_menu
                .as_ref()
                .unwrap()
                .contains(ctx, point)
    }

    pub fn is_contains_any_menus(&self, ctx: &mut ggez::Context, point: numeric::Point2f) -> bool {
        self.contains_customer_question_menu(ctx, point)
            || self.contains_text_balloon_ok_menu(ctx, point)
    }

    pub fn get_customer_question_position(&self) -> Option<numeric::Point2f> {
        if let Some(question_menu) = self.customer_question_menu.as_ref() {
            Some(question_menu.get_position())
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
    pub fn click_customer_question_menu<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        // ボタンエリア内をクリックしていない場合は、即終了
        if !self.contains_customer_question_menu(ctx.context, point) {
            return false;
        }

        if let Some(customer_question_menu) = self.customer_question_menu.as_mut() {
            customer_question_menu.on_click(ctx, t, button, point);
            true
        } else {
            false
        }
    }

    ///
    /// メニューのエントリをクリックしていたらtrueを返し、そうでなければfalseを返す
    ///
    pub fn click_text_balloon_ok_menu<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        // ボタンエリア内をクリックしていない場合は、即終了
        if !self.contains_text_balloon_ok_menu(ctx.context, point) {
            return false;
        }

        if let Some(ok_menu) = self.text_balloon_ok_menu.as_mut() {
            ok_menu.on_click(ctx, t, button, point);
            true
        } else {
            false
        }
    }

    pub fn question_menu_last_clicked_index(&mut self) -> Option<usize> {
        if let Some(customer_question_menu) = self.customer_question_menu.as_mut() {
            customer_question_menu
                .get_component()
                .get_last_clicked_index()
        } else {
            None
        }
    }

    pub fn get_text_balloon_ok_index(&self) -> Option<usize> {
        if let Some(ok_menu) = self.text_balloon_ok_menu.as_ref() {
            ok_menu.get_component().get_last_clicked_index()
        } else {
            None
        }
    }

    pub fn close_all(&mut self, t: Clock) {
        self.close_customer_question_menu(t);
        self.close_text_balloon_ok_menu(t);
    }

    pub fn show_customer_question_menu<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        position: numeric::Point2f,
        t: Clock,
    ) {
        let mut question_menu = CustomerQuestionMenu::new(ctx, 0);

        let frame_size = question_menu.get_date_frame_size();

        let canvas_size = numeric::Vector2f::new(frame_size.x + 64.0, frame_size.y + 100.0);

        question_menu.set_to_center(
            ctx,
            numeric::Point2f::new(canvas_size.x / 2.0, canvas_size.y / 2.0),
        );

        let pos = util::find_proper_window_position(
            numeric::Rect::new(position.x, position.y, canvas_size.x, canvas_size.y),
            numeric::Rect::new(
                0.0,
                0.0,
                core::WINDOW_SIZE_X as f32,
                core::WINDOW_SIZE_Y as f32,
            ),
        );

        let menu_rect = numeric::Rect::new(pos.x, pos.y, canvas_size.x, canvas_size.y);

        let mut customer_question_menu_area =
            DropDownArea::new(ctx, position, menu_rect, 0, question_menu, t);

        customer_question_menu_area.add_effect(vec![effect::fade_in(10, t)]);

        self.customer_question_menu = Some(customer_question_menu_area);
    }

    pub fn show_text_balloon_ok_menu<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        position: numeric::Point2f,
        t: Clock,
    ) {
        let ok_menu = OkMenu::new(ctx, 0);

        let frame_size = ok_menu.get_date_frame_size();

        let mut ok_menu_area = OkDropMenu::new(
            ctx,
            position,
            numeric::Rect::new(
                position.x,
                position.y,
                frame_size.x + 128.0,
                frame_size.y + 64.0,
            ),
            0,
            ok_menu,
            t,
        );

        ok_menu_area.add_effect(vec![effect::fade_in(10, t)]);

        self.text_balloon_ok_menu = Some(ok_menu_area);
    }

    ///
    /// # 再描画要求有り
    ///
    pub fn update<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        flush_delay_event_and_redraw_check!(self, self.event_list, ctx, t, {});

        if let Some(customer_question_menu) = self.customer_question_menu.as_mut() {
            if !customer_question_menu.is_stop() || !customer_question_menu.is_empty_effect() {
                ctx.process_utility.redraw();
            }

            customer_question_menu.move_with_func(t);
            customer_question_menu.effect(ctx.context, t);
        }

        if let Some(ok_menu) = self.text_balloon_ok_menu.as_mut() {
            if !ok_menu.is_stop() || !ok_menu.is_empty_effect() {
                ctx.process_utility.redraw();
            }

            ok_menu.move_with_func(t);
            ok_menu.effect(ctx.context, t);
        }
    }
}

impl DrawableComponent for CustomerMenuGroup {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            if let Some(customer_question_menu) = self.customer_question_menu.as_mut() {
                customer_question_menu.draw(ctx)?;
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
    book_status_menu: Option<BookStatusMenu>,
    book_title_menu: Option<BookTitleDropMenu>,
    customer_name_menu: Option<CustomerNameDropMenu>,
    simple_message_menu: Option<SimpleMessageDropMenu>,
    date_menu: Option<DateDropMenu>,
    date_check_menu: Option<DateCheckDropMenu>,
    drwob_essential: DrawableObjectEssential,
}

impl RecordBookMenuGroup {
    pub fn new(drawing_depth: i8) -> Self {
        RecordBookMenuGroup {
            event_list: DelayEventList::new(),
            book_status_menu: None,
            book_title_menu: None,
            customer_name_menu: None,
            simple_message_menu: None,
            date_menu: None,
	    date_check_menu: None,
            drwob_essential: DrawableObjectEssential::new(true, drawing_depth),
        }
    }

    pub fn is_some_menu_opened(&self) -> bool {
        self.book_status_menu.is_some()
            || self.book_title_menu.is_some()
            || self.customer_name_menu.is_some()
            || self.date_menu.is_some()
	    || self.date_check_menu.is_some()
            || self.simple_message_menu.is_some()
    }

    pub fn close_book_status_menu(&mut self, t: Clock) {
        let button_group = match self.book_status_menu.as_mut() {
            Some(it) => it,
            _ => return,
        };
        button_group.add_effect(vec![effect::fade_out(10, t)]);
        self.event_list.add_event(
            Box::new(|slf: &mut RecordBookMenuGroup, _, _| slf.book_status_menu = None),
            t + 11,
        );
    }

    pub fn close_book_title_menu(&mut self, t: Clock) {
        let title_menu = match self.book_title_menu.as_mut() {
            Some(it) => it,
            _ => return,
        };
        title_menu.add_effect(vec![effect::fade_out(10, t)]);
        self.event_list.add_event(
            Box::new(|slf: &mut RecordBookMenuGroup, _, _| slf.book_title_menu = None),
            t + 11,
        );
    }

    pub fn close_customer_name_menu(&mut self, t: Clock) {
        let customer_name_menu = match self.customer_name_menu.as_mut() {
            Some(it) => it,
            _ => return,
        };
        customer_name_menu.add_effect(vec![effect::fade_out(10, t)]);
        self.event_list.add_event(
            Box::new(|slf: &mut RecordBookMenuGroup, _, _| slf.customer_name_menu = None),
            t + 11,
        );
    }

    pub fn close_date_menu(&mut self, t: Clock) {
        let date_menu = match self.date_menu.as_mut() {
            Some(it) => it,
            _ => return,
        };
        date_menu.add_effect(vec![effect::fade_out(10, t)]);
        self.event_list.add_event(
            Box::new(|slf: &mut RecordBookMenuGroup, _, _| slf.date_menu = None),
            t + 11,
        );
    }

    pub fn close_date_check_menu(&mut self, t: Clock) {
        let date_menu = match self.date_check_menu.as_mut() {
            Some(it) => it,
            _ => return,
        };
        date_menu.add_effect(vec![effect::fade_out(10, t)]);
        self.event_list.add_event(
            Box::new(|slf: &mut RecordBookMenuGroup, _, _| slf.date_check_menu = None),
            t + 11,
        );
    }

    pub fn close_simple_message_menu(&mut self, t: Clock) {
        let msg_menu = match self.simple_message_menu.as_mut() {
            Some(it) => it,
            _ => return,
        };
        msg_menu.add_effect(vec![effect::fade_out(10, t)]);
        self.event_list.add_event(
            Box::new(|slf: &mut RecordBookMenuGroup, _, _| slf.simple_message_menu = None),
            t + 11,
        );
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

    pub fn contains_date_check_menu(&self, ctx: &mut ggez::Context, point: numeric::Point2f) -> bool {
        self.date_check_menu.is_some() && self.date_check_menu.as_ref().unwrap().contains(ctx, point)
    }

    pub fn contains_simple_message_menu(
        &self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> bool {
        self.simple_message_menu.is_some()
            && self
                .simple_message_menu
                .as_ref()
                .unwrap()
                .contains(ctx, point)
    }

    pub fn is_contains_any_menus(&self, ctx: &mut ggez::Context, point: numeric::Point2f) -> bool {
        self.contains_book_title_menu(ctx, point)
            || self.contains_book_status_menu(ctx, point)
            || self.contains_customer_name_menu(ctx, point)
            || self.contains_date_menu(ctx, point)
	    || self.contains_date_check_menu(ctx, point)
            || self.contains_simple_message_menu(ctx, point)
    }

    pub fn get_book_status_menu_position(&self) -> Option<numeric::Point2f> {
        if let Some(book_status_menu) = self.book_status_menu.as_ref() {
            Some(book_status_menu.get_click_position())
        } else {
            None
        }
    }

    pub fn get_book_title_menu_position(&self) -> Option<numeric::Point2f> {
        if let Some(book_title_menu) = self.book_title_menu.as_ref() {
            Some(book_title_menu.get_click_position())
        } else {
            None
        }
    }

    pub fn get_customer_name_menu_position(&self) -> Option<numeric::Point2f> {
        if let Some(customer_name_menu) = self.customer_name_menu.as_ref() {
            Some(customer_name_menu.get_click_position())
        } else {
            None
        }
    }

    pub fn get_date_menu_position(&self) -> Option<numeric::Point2f> {
        if let Some(date_menu) = self.date_menu.as_ref() {
            Some(date_menu.get_click_position())
        } else {
            None
        }
    }

    pub fn get_date_check_menu_position(&self) -> Option<numeric::Point2f> {
        if let Some(date_menu) = self.date_check_menu.as_ref() {
            Some(date_menu.get_click_position())
        } else {
            None
        }
    }

    pub fn get_simple_message_menu_position(&self) -> Option<numeric::Point2f> {
        if let Some(menu) = self.simple_message_menu.as_ref() {
            Some(menu.get_click_position())
        } else {
            None
        }
    }

    ///
    /// メニューのエントリをクリックしていたらtrueを返し、そうでなければfalseを返す
    ///
    pub fn click_book_status_menu<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        // ボタンエリア内をクリックしていない場合は、即終了
        if !self.contains_book_status_menu(ctx.context, point) {
            return false;
        }

        if let Some(book_status_menu) = self.book_status_menu.as_mut() {
            book_status_menu.on_click(ctx, t, button, point);
            true
        } else {
            false
        }
    }

    ///
    /// メニューのエントリをクリックしていたらtrueを返し、そうでなければfalseを返す
    ///
    pub fn click_book_title_menu<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        // ボタンエリア内をクリックしていない場合は、即終了
        if !self.contains_book_title_menu(ctx.context, point) {
            return false;
        }

        if let Some(book_title_menu) = self.book_title_menu.as_mut() {
            book_title_menu.on_click(ctx, t, button, point);
            true
        } else {
            false
        }
    }

    ///
    /// メニューのエントリをクリックしていたらtrueを返し、そうでなければfalseを返す
    ///
    pub fn click_customer_name_menu<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        // ボタンエリア内をクリックしていない場合は、即終了
        if !self.contains_customer_name_menu(ctx.context, point) {
            return false;
        }

        if let Some(customer_name_menu) = self.customer_name_menu.as_mut() {
            customer_name_menu.on_click(ctx, t, button, point);
            true
        } else {
            false
        }
    }

    ///
    /// メニューのエントリをクリックしていたらtrueを返し、そうでなければfalseを返す
    ///
    pub fn click_date_menu<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        // ボタンエリア内をクリックしていない場合は、即終了
        if !self.contains_date_menu(ctx.context, point) {
            return false;
        }

        if let Some(date_menu) = self.date_menu.as_mut() {
            date_menu.on_click(ctx, t, button, point);
            true
        } else {
            false
        }
    }

    pub fn click_date_check_menu<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        // ボタンエリア内をクリックしていない場合は、即終了
        if !self.contains_date_check_menu(ctx.context, point) {
            return false;
        }

        if let Some(date_menu) = self.date_check_menu.as_mut() {
            date_menu.on_click(ctx, t, button, point);
            true
        } else {
            false
        }
    }

    pub fn book_status_menu_last_clicked(&mut self) -> Option<usize> {
        if let Some(book_status_menu) = self.book_status_menu.as_mut() {
            book_status_menu.get_component().get_last_clicked()
        } else {
            None
        }
    }

    pub fn book_title_menu_last_clicked(&mut self) -> Option<(usize, BookInformation)> {
        if let Some(book_title_menu) = self.book_title_menu.as_mut() {
            let component = book_title_menu.get_component();
            if let Some(index) = component.get_last_clicked_index() {
                Some((index, component.get_last_clicked_book_info().unwrap()))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn customer_name_menu_last_clicked(&mut self) -> Option<(usize, String)> {
        if let Some(customer_name_menu) = self.customer_name_menu.as_mut() {
            let component = customer_name_menu.get_component();
            if let Some(index) = component.get_last_clicked_index() {
                Some((index, component.get_last_clicked_customer_name().unwrap()))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn date_menu_last_clicked(&mut self) -> Option<(usize, GensoDate)> {
        if let Some(date_menu) = self.date_menu.as_mut() {
            let component = date_menu.get_component();
            if let Some(index) = component.get_last_clicked_index() {
                Some((index, component.get_last_clicked_genso_date().unwrap()))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn date_check_menu_check_button_clicked(&mut self) -> bool {
        if let Some(date_menu) = self.date_check_menu.as_mut() {
	    date_menu.get_component_mut().check_button_is_clicked_volatile()
        } else {
	    false
        }
    }

    pub fn close_all(&mut self, t: Clock) {
        self.close_book_status_menu(t);
        self.close_book_title_menu(t);
        self.close_customer_name_menu(t);
        self.close_date_menu(t);
	self.close_date_check_menu(t);
        self.close_simple_message_menu(t);
    }

    pub fn show_book_status_menu<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        position: numeric::Point2f,
        t: Clock,
    ) {
        let button_group =
            BookStatusButtonGroup::new(ctx, numeric::Rect::new(0.0, 0.0, 70.0, 70.0), 20.0, 0);

        let mut button_group_area = DropDownArea::new(
            ctx,
            position,
            numeric::Rect::new(position.x, position.y, 290.0, 180.0),
            0,
            button_group,
            t,
        );

        button_group_area.add_effect(vec![effect::fade_in(10, t)]);

        self.book_status_menu = Some(button_group_area);
    }

    pub fn show_book_title_menu<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        position: numeric::Point2f,
        kosuzu_memory: &KosuzuMemory,
        t: Clock,
    ) {
        if kosuzu_memory.remembered_book_info.is_empty() {
            self.show_simple_message_menu(
                ctx,
                position,
                "".to_string(),
                "なんていう本だっけ".to_string(),
                t,
            );
            return;
        }

        let book_title_menu =
            BookTitleMenu::new(ctx, kosuzu_memory.remembered_book_info.clone(), 0);

        let frame_size = book_title_menu.get_title_frame_size();

        let menu_size = numeric::Point2f::new(frame_size.x + 128.0, frame_size.y + 64.0);

        let pos = util::find_proper_window_position(
            numeric::Rect::new(position.x, position.y, menu_size.x, menu_size.y),
            numeric::Rect::new(
                0.0,
                0.0,
                core::WINDOW_SIZE_X as f32,
                core::WINDOW_SIZE_Y as f32,
            ),
        );

        let menu_rect = numeric::Rect::new(pos.x, pos.y, menu_size.x, menu_size.y);

        let mut book_title_menu_area =
            DropDownArea::new(ctx, position, menu_rect, 0, book_title_menu, t);
        book_title_menu_area.add_effect(vec![effect::fade_in(10, t)]);

        self.book_title_menu = Some(book_title_menu_area);
    }

    fn show_simple_message_menu<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        position: numeric::Point2f,
        title: String,
        message: String,
        t: Clock,
    ) {
        let menu = SimpleMessageMenu::new(ctx, title, message, 28.0, 28.0, 0);

        let menu_size = menu.get_size();

        let rect = if (position.y + menu_size.y) as i16 <= core::WINDOW_SIZE_Y
            && (position.x + menu_size.x) as i16 <= core::WINDOW_SIZE_X
        {
            numeric::Rect::new(position.x, position.y, menu_size.x, menu_size.y)
        } else if (position.y + menu_size.y) as i16 <= core::WINDOW_SIZE_Y {
            numeric::Rect::new(
                position.x - menu_size.x,
                position.y,
                menu_size.x,
                menu_size.y,
            )
        } else if (position.x + menu_size.x) as i16 <= core::WINDOW_SIZE_X {
            numeric::Rect::new(
                position.x,
                position.y - menu_size.y,
                menu_size.x,
                menu_size.y,
            )
        } else {
            numeric::Rect::new(
                position.x - menu_size.x,
                position.y - menu_size.y,
                menu_size.x,
                menu_size.y,
            )
        };

        let mut drop_menu = SimpleMessageDropMenu::new(ctx, position, rect, 0, menu, t);

        drop_menu.add_effect(vec![effect::fade_in(10, t)]);

        self.simple_message_menu = Some(drop_menu);
    }

    pub fn show_locked_menu<'a>(
	&mut self,
	ctx: &mut SuzuContext<'a>,
        position: numeric::Point2f,
	t: Clock,
    ) {
	self.show_simple_message_menu(
            ctx,
            position,
            "".to_string(),
            "この項じゃないかも".to_string(),
            t,
        );
    }

    pub fn show_customer_name_menu<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        position: numeric::Point2f,
        kosuzu_memory: &KosuzuMemory,
        t: Clock,
    ) {
        if kosuzu_memory.customers_name.is_empty() {
            self.show_simple_message_menu(
                ctx,
                position,
                "".to_string(),
                "お名前聞かないと".to_string(),
                t,
            );
            return;
        }

        let mut customer_name_menu =
            CustomerNameMenu::new(ctx, kosuzu_memory.customers_name.clone(), 0);

        let frame_size = customer_name_menu.get_name_frame_size();

        let menu_size = numeric::Point2f::new(frame_size.x + 96.0, frame_size.y + 96.0);

        customer_name_menu.set_to_center(
            ctx,
            numeric::Point2f::new(menu_size.x / 2.0, menu_size.y / 2.0),
        );

        let pos = util::find_proper_window_position(
            numeric::Rect::new(position.x, position.y, menu_size.x, menu_size.y),
            numeric::Rect::new(
                0.0,
                0.0,
                core::WINDOW_SIZE_X as f32,
                core::WINDOW_SIZE_Y as f32,
            ),
        );

        let menu_rect = numeric::Rect::new(pos.x, pos.y, menu_size.x, menu_size.y);

        let mut customer_name_menu_area =
            DropDownArea::new(ctx, position, menu_rect, 0, customer_name_menu, t);
        customer_name_menu_area.add_effect(vec![effect::fade_in(10, t)]);

        self.customer_name_menu = Some(customer_name_menu_area);
    }

    pub fn show_date_menu<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        position: numeric::Point2f,
        today: GensoDate,
        t: Clock,
    ) {
        let mut date_menu = DateMenu::new(ctx, today, 0);

        let frame_size = date_menu.get_date_frame_size();

        let menu_size = numeric::Point2f::new(frame_size.x + 80.0, frame_size.y + 80.0);

        let pos = util::find_proper_window_position(
            numeric::Rect::new(position.x, position.y, menu_size.x, menu_size.y),
            numeric::Rect::new(
                0.0,
                0.0,
                core::WINDOW_SIZE_X as f32,
                core::WINDOW_SIZE_Y as f32,
            ),
        );

        let menu_rect = numeric::Rect::new(pos.x, pos.y, menu_size.x, menu_size.y);

        date_menu.set_to_center(
            ctx,
            numeric::Point2f::new(menu_size.x / 2.0, menu_size.y / 2.0),
        );

        let mut date_menu_area = DropDownArea::new(ctx, position, menu_rect, 0, date_menu, t);
        date_menu_area.add_effect(vec![effect::fade_in(10, t)]);

        self.date_menu = Some(date_menu_area);
    }

    pub fn show_date_check_menu<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        position: numeric::Point2f,
        today: GensoDate,
	return_date: GensoDate,
        t: Clock,
    ) {
        let mut date_menu = DateCheckMenu::new(ctx, today, return_date, 0);

        let frame_size = date_menu.get_date_frame_size();

        let menu_size = numeric::Point2f::new(frame_size.x + 80.0, frame_size.y + 120.0);

        let pos = util::find_proper_window_position(
            numeric::Rect::new(position.x, position.y, menu_size.x, menu_size.y),
            numeric::Rect::new(
                0.0,
                0.0,
                core::WINDOW_SIZE_X as f32,
                core::WINDOW_SIZE_Y as f32,
            ),
        );

        let menu_rect = numeric::Rect::new(pos.x, pos.y, menu_size.x, menu_size.y);

        date_menu.set_to_center(
            ctx,
            numeric::Point2f::new(menu_size.x / 2.0, menu_size.y / 2.0),
        );

        let mut date_menu_area = DropDownArea::new(ctx, position, menu_rect, 0, date_menu, t);
        date_menu_area.add_effect(vec![effect::fade_in(10, t)]);

        self.date_check_menu = Some(date_menu_area);
    }

    ///
    /// # 再描画要求有り
    ///
    pub fn update<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        flush_delay_event_and_redraw_check!(self, self.event_list, ctx, t, {});

        if let Some(book_status_menu) = self.book_status_menu.as_mut() {
            book_status_menu.move_with_func(t);
            book_status_menu.effect(ctx.context, t);
            ctx.process_utility.redraw();
        }

        if let Some(book_title_menu) = self.book_title_menu.as_mut() {
            book_title_menu.move_with_func(t);
            book_title_menu.effect(ctx.context, t);
            ctx.process_utility.redraw();
        }

        if let Some(customer_name_menu) = self.customer_name_menu.as_mut() {
            customer_name_menu.move_with_func(t);
            customer_name_menu.effect(ctx.context, t);
            ctx.process_utility.redraw();
        }

        if let Some(date_menu) = self.date_menu.as_mut() {
            date_menu.move_with_func(t);
            date_menu.effect(ctx.context, t);
            ctx.process_utility.redraw();
        }

	if let Some(date_menu) = self.date_check_menu.as_mut() {
            date_menu.move_with_func(t);
            date_menu.effect(ctx.context, t);
            ctx.process_utility.redraw();
        }

        if let Some(msg_menu) = self.simple_message_menu.as_mut() {
            msg_menu.move_with_func(t);
            msg_menu.effect(ctx.context, t);
            ctx.process_utility.redraw();
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

	    if let Some(date_menu) = self.date_check_menu.as_mut() {
                date_menu.draw(ctx)?;
            }

            if let Some(msg_menu) = self.simple_message_menu.as_mut() {
                msg_menu.draw(ctx)?;
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

pub struct BookInfoDrawer {
    book_info_frame: TableFrame,
    header_text: Vec<VerticalText>,
    info_field_text: Vec<VerticalText>,
    drwob_essential: DrawableObjectEssential,
}

impl BookInfoDrawer {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        book_info: BookInformation,
        drawing_depth: i8,
    ) -> Self {
        let mut info_field_vtext = Vec::new();
        let mut header_text = Vec::new();

        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(32.0, 32.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let book_info_frame = TableFrame::new(
            ctx.resource,
            numeric::Point2f::new(50.0, 20.0),
            TileBatchTextureID::OldStyleFrame,
            FrameData::new(vec![150.0, 150.0], vec![56.0; 3]),
            numeric::Vector2f::new(0.3, 0.3),
            0,
        );

        for (index, s) in vec!["状態", "寸法", "妖魔本"].iter().enumerate() {
            let mut vtext = VerticalText::new(
                s.to_string(),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                drawing_depth,
                font_info,
            );

            set_table_frame_cell_center!(
                ctx.context,
                book_info_frame,
                vtext,
                numeric::Vector2u::new(index as u32, 0)
            );

            header_text.push(vtext);
        }

        for (index, s) in vec![
            book_info.get_condition_string(),
            book_info.size.clone(),
            "100".to_string(),
        ]
        .iter()
        .enumerate()
        {
            let mut vtext = VerticalText::new(
                s.to_string(),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                drawing_depth,
                font_info,
            );

            set_table_frame_cell_center!(
                ctx.context,
                book_info_frame,
                vtext,
                numeric::Vector2u::new(index as u32, 1)
            );

            info_field_vtext.push(vtext);
        }

        BookInfoDrawer {
            book_info_frame: book_info_frame,
            info_field_text: info_field_vtext,
            header_text: header_text,
            drwob_essential: DrawableObjectEssential::new(true, drawing_depth),
        }
    }

    pub fn make_center<'a>(&mut self, ctx: &mut SuzuContext<'a>, center_pos: numeric::Point2f) {
        self.book_info_frame.make_center(center_pos);

        for (index, vtext) in self.header_text.iter_mut().enumerate() {
            set_table_frame_cell_center!(
                ctx.context,
                self.book_info_frame,
                vtext,
                numeric::Vector2u::new(index as u32, 0)
            );
        }

        for (index, vtext) in self.info_field_text.iter_mut().enumerate() {
            set_table_frame_cell_center!(
                ctx.context,
                self.book_info_frame,
                vtext,
                numeric::Vector2u::new(index as u32, 1)
            );
        }
    }

    pub fn get_book_info_frame_size(&self) -> numeric::Vector2f {
        self.book_info_frame.size()
    }
}

impl DrawableComponent for BookInfoDrawer {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.book_info_frame.draw(ctx)?;

            for vtext in &mut self.info_field_text {
                vtext.draw(ctx)?;
            }

            for vtext in &mut self.header_text {
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

pub struct DeskBookMenu {
    header_text: UniText,
    book_info_drawer: BookInfoDrawer,
    book_info: BookInformation,
    memo_button: SelectButton,
    drwob_essential: DrawableObjectEssential,
    last_clicked: Option<usize>,
}

impl DeskBookMenu {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        book_info: BookInformation,
        drawing_depth: i8,
    ) -> Self {
        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(30.0, 30.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let header_text = UniText::new(
            "本の容態".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            font_info,
        );

        let book_info_drawer = BookInfoDrawer::new(ctx, book_info.clone(), 0);

        let mut button_rect = numeric::Rect::new(110.0, 410.0, 140.0, 50.0);
        let mut buttons = Vec::new();
        for s in vec!["メモ"].iter() {
            let text_texture = TextButtonTexture::new(
                ctx,
                numeric::Point2f::new(0.0, 0.0),
                s.to_string(),
                font_info,
                10.0,
                ggraphics::Color::from_rgba_u32(0xe8b5a2ff),
                0,
            );

            let button = SelectButton::new(ctx, button_rect, Box::new(text_texture));

            buttons.push(button);
            button_rect.x += button_rect.w - 15.0;
        }

        let mut menu = DeskBookMenu {
            header_text: header_text,
            book_info_drawer: book_info_drawer,
            book_info: book_info,
            memo_button: buttons.pop().unwrap(),
            drwob_essential: DrawableObjectEssential::new(true, drawing_depth),
            last_clicked: None,
        };

        let size = menu.get_date_frame_size();

        menu.header_text
            .make_center(ctx.context, numeric::Point2f::new(size.x / 2.0, 44.0));
        menu.book_info_drawer.make_center(
            ctx,
            numeric::Point2f::new(size.x / 2.0, (size.y / 2.0) - 12.0),
        );

        menu
    }

    pub fn click_handler(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        if self.memo_button.contains(ctx, point) {
            self.last_clicked = Some(0 as usize);
        }
    }

    pub fn get_last_clicked(&self) -> Option<usize> {
        self.last_clicked
    }

    pub fn get_target_book_info(&self) -> BookInformation {
        self.book_info.clone()
    }

    pub fn get_date_frame_size(&self) -> numeric::Vector2f {
        let header_text_size = self.header_text.get_font_scale();
        let book_info_size = self.book_info_drawer.get_book_info_frame_size();

        numeric::Vector2f::new(314.0, header_text_size.y + book_info_size.y + 150.0)
    }
}

impl DrawableComponent for DeskBookMenu {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.header_text.draw(ctx)?;
            self.book_info_drawer.draw(ctx)?;

            self.memo_button.draw(ctx)?;
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
    fn on_click<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        _t: Clock,
        _button: ggez::event::MouseButton,
        point: numeric::Point2f,
    ) {
        self.click_handler(ctx.context, point);
    }
}

pub type DeskBookDropMenu = DropDownArea<DeskBookMenu>;

pub struct OnDeskMenuGroup {
    event_list: DelayEventList<Self>,
    desk_book_menu: Option<DeskBookDropMenu>,
    drwob_essential: DrawableObjectEssential,
}

impl OnDeskMenuGroup {
    pub fn new(drawing_depth: i8) -> Self {
        OnDeskMenuGroup {
            event_list: DelayEventList::new(),
            desk_book_menu: None,
            drwob_essential: DrawableObjectEssential::new(true, drawing_depth),
        }
    }

    pub fn is_some_menu_opened(&self) -> bool {
        self.desk_book_menu.is_some()
    }

    pub fn close_desk_book_menu(&mut self, t: Clock) {
        if let Some(desk_book_menu) = self.desk_book_menu.as_mut() {
            desk_book_menu.add_effect(vec![effect::fade_out(10, t)]);
            self.event_list.add_event(
                Box::new(|slf: &mut OnDeskMenuGroup, _, _| slf.desk_book_menu = None),
                t + 11,
            );
        }
    }

    pub fn contains_desk_book_menu(
        &self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> bool {
        self.desk_book_menu.is_some() && self.desk_book_menu.as_ref().unwrap().contains(ctx, point)
    }

    pub fn is_contains_any_menus(&self, ctx: &mut ggez::Context, point: numeric::Point2f) -> bool {
        self.contains_desk_book_menu(ctx, point)
    }

    pub fn get_desk_book_menu_position(&self) -> Option<numeric::Point2f> {
        if let Some(desk_book_menu) = self.desk_book_menu.as_ref() {
            Some(desk_book_menu.get_position())
        } else {
            None
        }
    }

    ///
    /// メニューのエントリをクリックしていたらtrueを返し、そうでなければfalseを返す
    ///
    pub fn click_desk_book_menu<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
        t: Clock,
    ) -> bool {
        // ボタンエリア内をクリックしていない場合は、即終了
        if !self.contains_desk_book_menu(ctx.context, point) {
            return false;
        }

        if let Some(desk_book_menu) = self.desk_book_menu.as_mut() {
            desk_book_menu.on_click(ctx, t, button, point);
            true
        } else {
            false
        }
    }

    pub fn desk_book_menu_last_clicked(&mut self) -> Option<usize> {
        if let Some(desk_book_menu) = self.desk_book_menu.as_mut() {
            desk_book_menu.get_component().get_last_clicked()
        } else {
            None
        }
    }

    pub fn close_all(&mut self, t: Clock) {
        self.close_desk_book_menu(t);
    }

    pub fn show_desk_book_menu<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        position: numeric::Point2f,
        book_info: BookInformation,
        t: Clock,
    ) {
        let menu = DeskBookMenu::new(ctx, book_info, 0);

        let frame_size = menu.get_date_frame_size();

        let menu_size = numeric::Point2f::new(frame_size.x, frame_size.y);

        let pos = util::find_proper_window_position(
            numeric::Rect::new(position.x, position.y, menu_size.x, menu_size.y),
            numeric::Rect::new(
                0.0,
                0.0,
                core::WINDOW_SIZE_X as f32,
                core::WINDOW_SIZE_Y as f32,
            ),
        );

        let menu_rect = numeric::Rect::new(pos.x, pos.y, menu_size.x, menu_size.y);

        let mut dd_area = DropDownArea::new(ctx, position, menu_rect, 0, menu, t);

        dd_area.add_effect(vec![effect::fade_in(10, t)]);

        self.desk_book_menu = Some(dd_area);
    }

    ///
    /// # 再描画要求有り
    ///
    pub fn update<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        flush_delay_event_and_redraw_check!(self, self.event_list, ctx, t, {});

        if let Some(desk_book_menu) = self.desk_book_menu.as_mut() {
            if !desk_book_menu.is_stop() || !desk_book_menu.is_empty_effect() {
                ctx.process_utility.redraw();
            }
            desk_book_menu.move_with_func(t);
            desk_book_menu.effect(ctx.context, t);
        }
    }

    pub fn get_desk_menu_target_book_info(&self) -> Option<BookInformation> {
        if self.desk_book_menu.is_some() {
            Some(
                self.desk_book_menu
                    .as_ref()
                    .unwrap()
                    .get_component()
                    .get_target_book_info(),
            )
        } else {
            None
        }
    }
}

impl DrawableComponent for OnDeskMenuGroup {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            if let Some(desk_book_menu) = self.desk_book_menu.as_mut() {
                desk_book_menu.draw(ctx)?;
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
