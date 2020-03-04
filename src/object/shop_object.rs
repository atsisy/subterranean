use std::rc::Rc;

use ggez::graphics as ggraphics;

use sub_screen::SubScreen;
use torifune::core::Clock;
use torifune::debug;
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::*;
use torifune::graphics::*;
use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;
use torifune::numeric;

use crate::core::{BookInformation, BookShelfInformation, FontID, GameData, TextureID};
use crate::object::Clickable;

use number_to_jk::number_to_jk;

pub struct SelectButton {
    canvas: SubScreen,
    button_texture: Box<dyn TextureObject>,
    button_toggle: bool,
}

impl SelectButton {
    pub fn new(
        ctx: &mut ggez::Context,
        button_rect: numeric::Rect,
        mut texture: Box<dyn TextureObject>,
    ) -> Self {
        texture.set_position(numeric::Point2f::new(0.0, 0.0));

        SelectButton {
            canvas: SubScreen::new(ctx, button_rect, 0, ggraphics::Color::from_rgba_u32(0)),
            button_texture: texture,
            button_toggle: false,
        }
    }

    pub fn push(&mut self) {
        self.button_toggle = true;
        self.button_texture
            .set_color(ggraphics::Color::from_rgba_u32(0xffffffff));
    }

    pub fn release(&mut self) {
        self.button_toggle = false;
        self.button_texture
            .set_color(ggraphics::Color::from_rgba_u32(0x888888ff));
    }

    pub fn get_button_status(&self) -> bool {
        self.button_toggle
    }
}

impl DrawableComponent for SelectButton {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.button_texture.draw(ctx)?;

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

impl DrawableObject for SelectButton {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for SelectButton {
    impl_texture_object_for_wrapped! {canvas}
}

pub struct SelectBookWindow {
    canvas: SubScreen,
    title: VerticalText,
    cell_desc: VerticalText,
    book_text: Vec<VerticalText>,
    selecting_book_index: Vec<usize>,
    book_font: FontInformation,
}

impl SelectBookWindow {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        window_rect: numeric::Rect,
        title: &str,
        book_info: Vec<BookInformation>,
    ) -> Self {
        let font_info = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(32.0, 32.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let mut window = SelectBookWindow {
            canvas: SubScreen::new(
                ctx,
                window_rect,
                0,
                ggraphics::Color::from_rgba_u32(0xeeeeeeff),
            ),
            title: VerticalText::new(
                title.to_string(),
                numeric::Point2f::new(window_rect.w - 50.0, 50.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                font_info,
            ),
            cell_desc: VerticalText::new(
                "請求番号\t\t表題".to_string(),
                numeric::Point2f::new(window_rect.w - 100.0, 50.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                font_info,
            ),
            book_text: Vec::new(),
            selecting_book_index: Vec::new(),
            book_font: font_info,
        };

        window.update_contents(ctx, &book_info);

        window
    }

    fn update_contents(&mut self, ctx: &mut ggez::Context, book_info: &Vec<BookInformation>) {
        let window_rect = self.canvas.get_drawing_area(ctx);
        let mut text_position = numeric::Point2f::new(window_rect.w - 150.0, 50.0);

        self.book_text = book_info
            .iter()
            .enumerate()
            .map(|enumerate_data| {
                let (index, info) = enumerate_data;

                if index == 12 {
                    text_position.x = window_rect.w - 100.0;
                    text_position.y += 500.0;
                }

                let vtext = VerticalText::new(
                    format!(
                        "{0: <6}{1}",
                        number_to_jk(info.billing_number as u64),
                        info.name
                    ),
                    text_position,
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    self.book_font.clone(),
                );
                text_position.x -= 40.0;
                vtext
            })
            .collect();
    }

    pub fn sort_selecting_index_less(&mut self) {
        self.selecting_book_index.sort_by(|a, b| b.cmp(a));
    }

    pub fn get_selecting_index(&self) -> &Vec<usize> {
        &self.selecting_book_index
    }

    pub fn clear_selecting_index(&mut self) {
        self.selecting_book_index.clear();
    }
}

impl DrawableComponent for SelectBookWindow {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.title.draw(ctx)?;
            self.cell_desc.draw(ctx)?;
            for vtext in &mut self.book_text {
                vtext.draw(ctx)?;
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

    fn set_drawing_depth(&mut self, depth: i8) {
        self.canvas.set_drawing_depth(depth);
    }

    fn get_drawing_depth(&self) -> i8 {
        self.canvas.get_drawing_depth()
    }
}

impl DrawableObject for SelectBookWindow {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for SelectBookWindow {
    impl_texture_object_for_wrapped! {canvas}
}

impl Clickable for SelectBookWindow {
    fn on_click(
        &mut self,
        ctx: &mut ggez::Context,
        _: &GameData,
        _: Clock,
        _button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        let rpoint = self.canvas.relative_point(point);

        for (index, vtext) in self.book_text.iter_mut().enumerate() {
            if vtext.get_drawing_area(ctx).contains(rpoint) {
                // 既に選択されている場合は、削除
                if self.selecting_book_index.contains(&index) {
                    vtext.set_color(ggraphics::Color::from_rgba_u32(0x000000ff));
                    self.selecting_book_index
                        .retain(|inner_index| *inner_index != index);
                } else {
                    // テキストを赤に変更し、選択中のインデックスとして登録
                    vtext.set_color(ggraphics::Color::from_rgba_u32(0xee0000ff));
                    self.selecting_book_index.push(index);
                }

                break;
            }
        }

        debug::debug_screen_push_text(&format!(
            "window select text: {:?}",
            self.selecting_book_index
        ));
    }

    fn clickable_status(
        &mut self,
        _ctx: &mut ggez::Context,
        _point: numeric::Point2f,
    ) -> ggez::input::mouse::MouseCursor {
        ggez::input::mouse::MouseCursor::Default
    }
}

pub struct SelectShelvingBookUI {
    canvas: SubScreen,
    boxed_books: Vec<BookInformation>,
    shelving_books: Vec<BookInformation>,
    box_info_window: SelectBookWindow,
    shelving_window: SelectBookWindow,
    move_box_to_shelving_button: SelectButton,
    move_shelving_to_box_button: SelectButton,
}

impl SelectShelvingBookUI {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        ui_rect: numeric::Rect,
        mut box_book_info: Vec<BookInformation>,
        mut shelving_book: Vec<BookInformation>,
    ) -> Self {
        box_book_info.sort_by(|a, b| a.billing_number.cmp(&b.billing_number));
        shelving_book.sort_by(|a, b| a.billing_number.cmp(&b.billing_number));

        SelectShelvingBookUI {
            canvas: SubScreen::new(ctx, ui_rect, 0, ggraphics::Color::from_rgba_u32(0)),
            box_info_window: SelectBookWindow::new(
                ctx,
                game_data,
                numeric::Rect::new(70.0, 50.0, 550.0, 600.0),
                "返却済み",
                box_book_info.clone(),
            ),
            shelving_window: SelectBookWindow::new(
                ctx,
                game_data,
                numeric::Rect::new(770.0, 50.0, 550.0, 600.0),
                "配架中",
                shelving_book.clone(),
            ),
            boxed_books: box_book_info,
            shelving_books: shelving_book,
            move_box_to_shelving_button: SelectButton::new(
                ctx,
                numeric::Rect::new(650.0, 200.0, 100.0, 50.0),
                Box::new(UniTexture::new(
                    game_data.ref_texture(TextureID::ArrowRight),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(0.5, 0.5),
                    0.0,
                    0,
                )),
            ),
            move_shelving_to_box_button: SelectButton::new(
                ctx,
                numeric::Rect::new(650.0, 500.0, 100.0, 50.0),
                Box::new(UniTexture::new(
                    game_data.ref_texture(TextureID::ArrowLeft),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(0.5, 0.5),
                    0.0,
                    0,
                )),
            ),
        }
    }

    fn sort_book_info_greater(&mut self) {
        self.boxed_books
            .sort_by(|a, b| a.billing_number.cmp(&b.billing_number));
        self.shelving_books
            .sort_by(|a, b| a.billing_number.cmp(&b.billing_number));
    }

    fn update_window(&mut self, ctx: &mut ggez::Context) {
        self.sort_book_info_greater();
        self.box_info_window.update_contents(ctx, &self.boxed_books);
        self.shelving_window
            .update_contents(ctx, &self.shelving_books);
    }

    fn move_box_to_shelving(&mut self, ctx: &mut ggez::Context) {
        self.box_info_window.sort_selecting_index_less();
        for selecting_index in self.box_info_window.get_selecting_index().iter() {
            debug::debug_screen_push_text(&format!("remove book: {}", selecting_index));
            let select_book = self.boxed_books.swap_remove(*selecting_index);
            self.shelving_books.push(select_book);
        }

        self.box_info_window.clear_selecting_index();
        self.update_window(ctx);
    }

    fn move_shelving_to_box(&mut self, ctx: &mut ggez::Context) {
        self.shelving_window.sort_selecting_index_less();
        for selecting_index in self.shelving_window.get_selecting_index().iter() {
            debug::debug_screen_push_text(&format!("remove book: {}", selecting_index));
            let select_book = self.shelving_books.swap_remove(*selecting_index);
            self.boxed_books.push(select_book);
        }

        self.box_info_window.clear_selecting_index();
        self.shelving_window.clear_selecting_index();

        self.update_window(ctx);
    }

    pub fn get_select_result(&self) -> (Vec<BookInformation>, Vec<BookInformation>) {
        (self.boxed_books.clone(), self.shelving_books.clone())
    }
}

impl DrawableComponent for SelectShelvingBookUI {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.box_info_window.draw(ctx)?;
            self.shelving_window.draw(ctx)?;

            self.move_box_to_shelving_button.draw(ctx)?;
            self.move_shelving_to_box_button.draw(ctx)?;

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

impl DrawableObject for SelectShelvingBookUI {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for SelectShelvingBookUI {
    impl_texture_object_for_wrapped! {canvas}
}

impl Clickable for SelectShelvingBookUI {
    fn on_click(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        clock: Clock,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        let rpoint = self.canvas.relative_point(point);

        if self.box_info_window.get_drawing_area(ctx).contains(rpoint) {
            self.box_info_window
                .on_click(ctx, game_data, clock, button, rpoint);
        }

        if self.shelving_window.get_drawing_area(ctx).contains(rpoint) {
            self.shelving_window
                .on_click(ctx, game_data, clock, button, rpoint);
        }

        if self
            .move_box_to_shelving_button
            .get_drawing_area(ctx)
            .contains(rpoint)
        {
            self.move_box_to_shelving(ctx);
        }

        if self
            .move_shelving_to_box_button
            .get_drawing_area(ctx)
            .contains(rpoint)
        {
            self.move_shelving_to_box(ctx);
        }
    }

    fn clickable_status(
        &mut self,
        _ctx: &mut ggez::Context,
        _point: numeric::Point2f,
    ) -> ggez::input::mouse::MouseCursor {
        ggez::input::mouse::MouseCursor::Default
    }
}

pub struct SelectStoringBookWindow {
    canvas: SubScreen,
    title: VerticalText,
    cell_desc: VerticalText,
    book_text: Vec<VerticalText>,
    selecting_book_index: Vec<usize>,
    book_font: FontInformation,
}

impl SelectStoringBookWindow {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        window_rect: numeric::Rect,
        title: &str,
	book_shelf_info: &BookShelfInformation,
        book_info: Vec<BookInformation>,
    ) -> Self {
        let font_info = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(32.0, 32.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let mut window = SelectStoringBookWindow {
            canvas: SubScreen::new(
                ctx,
                window_rect,
                0,
                ggraphics::Color::from_rgba_u32(0xeeeeeeff),
            ),
            title: VerticalText::new(
                title.to_string(),
                numeric::Point2f::new(window_rect.w - 50.0, 50.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                font_info,
            ),
            cell_desc: VerticalText::new(
                "返却可否\t\t請求番号\t\t表題".to_string(),
                numeric::Point2f::new(window_rect.w - 100.0, 50.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                font_info,
            ),
            book_text: Vec::new(),
            selecting_book_index: Vec::new(),
            book_font: font_info,
        };

        window.update_contents(ctx, book_shelf_info, &book_info);

        window
    }

    fn update_contents(&mut self, ctx: &mut ggez::Context,
		       book_shelf_info: &BookShelfInformation, book_info: &Vec<BookInformation>) {
        let window_rect = self.canvas.get_drawing_area(ctx);
        let mut text_position = numeric::Point2f::new(window_rect.w - 150.0, 50.0);

        self.book_text = book_info
            .iter()
            .enumerate()
            .map(|enumerate_data| {
                let (index, info) = enumerate_data;

                if index == 12 {
                    text_position.x = window_rect.w - 100.0;
                    text_position.y += 500.0;
                }

                let vtext = VerticalText::new(
                    format!(
                        "{0: <4}{1: <6}{2}",
			if book_shelf_info.contains_number(info.billing_number) { "可" } else { "不可" },
                        number_to_jk(info.billing_number as u64),
                        info.name
                    ),
                    text_position,
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    self.book_font.clone(),
                );
                text_position.x -= 40.0;
                vtext
            })
            .collect();
    }

    pub fn sort_selecting_index_less(&mut self) {
        self.selecting_book_index.sort_by(|a, b| b.cmp(a));
    }

    pub fn get_selecting_index(&self) -> &Vec<usize> {
        &self.selecting_book_index
    }

    pub fn clear_selecting_index(&mut self) {
        self.selecting_book_index.clear();

	for vtext in self.book_text.iter_mut() {
	    vtext.set_color(ggraphics::Color::from_rgba_u32(0x000000ff));
        }
    }
}

impl DrawableComponent for SelectStoringBookWindow {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.title.draw(ctx)?;
            self.cell_desc.draw(ctx)?;
            for vtext in &mut self.book_text {
                vtext.draw(ctx)?;
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

    fn set_drawing_depth(&mut self, depth: i8) {
        self.canvas.set_drawing_depth(depth);
    }

    fn get_drawing_depth(&self) -> i8 {
        self.canvas.get_drawing_depth()
    }
}

impl DrawableObject for SelectStoringBookWindow {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for SelectStoringBookWindow {
    impl_texture_object_for_wrapped! {canvas}
}

impl Clickable for SelectStoringBookWindow {
    fn on_click(
        &mut self,
        ctx: &mut ggez::Context,
        _: &GameData,
        _: Clock,
        _button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        let rpoint = self.canvas.relative_point(point);

        for (index, vtext) in self.book_text.iter_mut().enumerate() {
            if vtext.get_drawing_area(ctx).contains(rpoint) {
                // 既に選択されている場合は、削除
                if self.selecting_book_index.contains(&index) {
                    vtext.set_color(ggraphics::Color::from_rgba_u32(0x000000ff));
                    self.selecting_book_index
                        .retain(|inner_index| *inner_index != index);
                } else {
                    // テキストを赤に変更し、選択中のインデックスとして登録
                    vtext.set_color(ggraphics::Color::from_rgba_u32(0xee0000ff));
                    self.selecting_book_index.push(index);
                }

                break;
            }
        }

        debug::debug_screen_push_text(&format!(
            "window select text: {:?}",
            self.selecting_book_index
        ));
    }

    fn clickable_status(
        &mut self,
        _ctx: &mut ggez::Context,
        _point: numeric::Point2f,
    ) -> ggez::input::mouse::MouseCursor {
        ggez::input::mouse::MouseCursor::Default
    }
}

pub struct SelectStoreBookUI {
    canvas: SubScreen,
    shelving_books: Vec<BookInformation>,
    stored_books: Vec<BookInformation>,
    select_book_window: SelectStoringBookWindow,
    store_button: SelectButton,
    reset_select_button: SelectButton,
    book_shelf_info: BookShelfInformation,
}

impl SelectStoreBookUI {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        ui_rect: numeric::Rect,
	book_shelf_info: BookShelfInformation,
        mut shelving_book: Vec<BookInformation>,
    ) -> Self {
        shelving_book.sort_by(|a, b| a.billing_number.cmp(&b.billing_number));

        SelectStoreBookUI {
            canvas: SubScreen::new(ctx, ui_rect, 0, ggraphics::Color::from_rgba_u32(0)),
            select_book_window: SelectStoringBookWindow::new(
                ctx,
                game_data,
                numeric::Rect::new(70.0, 50.0, 550.0, 600.0),
                "配架中",
		&book_shelf_info,
                shelving_book.clone(),
            ),
            shelving_books: shelving_book,
	    stored_books: Vec::new(),
            store_button: SelectButton::new(
                ctx,
                numeric::Rect::new(650.0, 200.0, 100.0, 50.0),
                Box::new(UniTexture::new(
                    game_data.ref_texture(TextureID::StoreButton),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(0.5, 0.5),
                    0.0,
                    0,
                )),
            ),
            reset_select_button: SelectButton::new(
                ctx,
                numeric::Rect::new(650.0, 500.0, 100.0, 50.0),
                Box::new(UniTexture::new(
                    game_data.ref_texture(TextureID::ResetButton),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(0.5, 0.5),
                    0.0,
                    0,
                )),
            ),
	    book_shelf_info: book_shelf_info,
        }
    }

    fn sort_book_info_greater(&mut self) {
        self.shelving_books
            .sort_by(|a, b| a.billing_number.cmp(&b.billing_number));
    }

    fn update_window(&mut self, ctx: &mut ggez::Context) {
        self.sort_book_info_greater();
        self.select_book_window.update_contents(ctx, &self.book_shelf_info, &self.shelving_books);
    }

    fn store_shelving_books(&mut self, ctx: &mut ggez::Context) {
        self.select_book_window.sort_selecting_index_less();
        for selecting_index in self.select_book_window.get_selecting_index().iter() {
            debug::debug_screen_push_text(&format!("remove book: {}", selecting_index));
            let returned = self.shelving_books.swap_remove(*selecting_index);
	    self.stored_books.push(returned);
        }

        self.update_window(ctx);
    }

    ///
    /// 返却イベントの結果を返す
    /// (返却済み, 配架中)
    ///
    pub fn get_storing_result(&self) -> (Vec<BookInformation>, Vec<BookInformation>) {
	(self.stored_books.clone(), self.shelving_books.clone())
    }
}

impl DrawableComponent for SelectStoreBookUI {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.select_book_window.draw(ctx)?;

            self.reset_select_button.draw(ctx)?;
            self.store_button.draw(ctx)?;

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

impl DrawableObject for SelectStoreBookUI {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for SelectStoreBookUI {
    impl_texture_object_for_wrapped! {canvas}
}

impl Clickable for SelectStoreBookUI {
    fn on_click(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        clock: Clock,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        let rpoint = self.canvas.relative_point(point);

        if self.select_book_window.get_drawing_area(ctx).contains(rpoint) {
            self.select_book_window
                .on_click(ctx, game_data, clock, button, rpoint);
        }

        if self
            .reset_select_button
            .get_drawing_area(ctx)
            .contains(rpoint)
        {
	    self.select_book_window.clear_selecting_index();
        }

        if self
            .store_button
            .get_drawing_area(ctx)
            .contains(rpoint)
        {
	    self.store_shelving_books(ctx);
        }
    }

    fn clickable_status(
        &mut self,
        _ctx: &mut ggez::Context,
        _point: numeric::Point2f,
    ) -> ggez::input::mouse::MouseCursor {
        ggez::input::mouse::MouseCursor::Default
    }
}
