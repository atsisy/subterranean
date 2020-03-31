use std::rc::Rc;

use ggez::graphics as ggraphics;

use torifune::core::{Clock, Updatable};
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;
use torifune::graphics::object::*;
use torifune::graphics::*;
use torifune::roundup2f;
use torifune::numeric;

use crate::core::BookInformation;
use crate::object::util_object::*;

use super::Clickable;
use crate::core::{FontID, GameData, TextureID};

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

    pub fn get_title_frame_size(&self) -> numeric::Vector2f {
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
