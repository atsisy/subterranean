use std::rc::Rc;

use ggez::graphics as ggraphics;
use ggez::input::mouse::MouseButton;

use sub_screen::SubScreen;
use torifune::core::{Clock, Updatable};
use torifune::debug;
use torifune::device::*;
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::*;
use torifune::graphics::*;
use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;
use torifune::numeric;

use crate::core::{BookInformation, BookShelfInformation, FontID, GameData, TextureID};
use crate::object::move_fn;
use crate::object::util_object::*;
use crate::object::Clickable;
use crate::scene::suzuna_scene::TaskResult;
use crate::scene::DelayEventList;

use number_to_jk::number_to_jk;

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
            if vtext.contains(ctx, rpoint) {
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

    ///
    /// 請求番号を昇順に並べ替えるメソッド
    ///
    fn sort_book_info_greater(&mut self) {
        self.boxed_books
            .sort_by(|a, b| a.billing_number.cmp(&b.billing_number));
        self.shelving_books
            .sort_by(|a, b| a.billing_number.cmp(&b.billing_number));
    }

    ///
    /// Windowに描画される内容を更新するメソッド
    ///
    fn update_window(&mut self, ctx: &mut ggez::Context) {
        // 請求番号でソートする
        self.sort_book_info_greater();

        // 描画コンテンツを更新
        self.box_info_window.update_contents(ctx, &self.boxed_books);
        self.shelving_window
            .update_contents(ctx, &self.shelving_books);
    }

    ///
    /// 箱に入っている本を手持ちに加える処理
    ///    
    fn move_box_to_shelving(&mut self, ctx: &mut ggez::Context) {
        // 選択された本のインデックスを降順にソート
        self.box_info_window.sort_selecting_index_less();

        // インデックスは降順でソートされているため、本のデータを後ろから取り出していくことになる
        // したがって、インデックスをそのまま使える。
        // 結果的に、選択された本をすべてshelving_booksに追加することができる
        for selecting_index in self.box_info_window.get_selecting_index().iter() {
            debug::debug_screen_push_text(&format!("remove book: {}", selecting_index));
            let select_book = self.boxed_books.swap_remove(*selecting_index);
            self.shelving_books.push(select_book);
        }

        // 選択中データをクリア
        self.box_info_window.clear_selecting_index();
        self.shelving_window.clear_selecting_index();

        // Windowを更新
        self.update_window(ctx);
    }

    ///
    /// 手持ちの本を箱に戻す処理
    ///
    fn move_shelving_to_box(&mut self, ctx: &mut ggez::Context) {
        // 大体同じ
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

    ///
    /// 現在の本の所持状態を返すメソッド
    /// タプルで(箱入り, 手持ち)の本を返す
    ///
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
        // それぞれのオブジェクトに処理を渡すだけ

        let rpoint = self.canvas.relative_point(point);

        if self.box_info_window.contains(ctx, rpoint) {
            self.box_info_window
                .on_click(ctx, game_data, clock, button, rpoint);
        }

        if self.shelving_window.contains(ctx, rpoint) {
            self.shelving_window
                .on_click(ctx, game_data, clock, button, rpoint);
        }

        if self.move_box_to_shelving_button.contains(ctx, point) {
            self.move_box_to_shelving(ctx);
        }

        if self.move_shelving_to_box_button.contains(ctx, rpoint) {
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
    book_storable: Vec<bool>,
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
            book_storable: Vec::new(),
            book_font: font_info,
        };

        window.update_contents(ctx, book_shelf_info, &book_info);

        window
    }

    fn update_contents(
        &mut self,
        ctx: &mut ggez::Context,
        book_shelf_info: &BookShelfInformation,
        book_info: &Vec<BookInformation>,
    ) {
        let window_rect = self.canvas.get_drawing_area(ctx);
        let mut text_position = numeric::Point2f::new(window_rect.w - 150.0, 50.0);

        // ここには、そのインデックスの本が配架可能かどうかがboolで入る
        self.book_storable.clear();

        self.book_text = book_info
            .iter()
            .enumerate()
            .map(|enumerate_data| {
                let (index, info) = enumerate_data;

                // 12冊を超える場合は、一段下に表示 (FIXME)
                if index == 12 {
                    text_position.x = window_rect.w - 100.0;
                    text_position.y += 500.0;
                }

                // 配架可能か？
                let is_storable = book_shelf_info.contains_number(info.billing_number);
                // 配架可能状態をpush
                self.book_storable.push(is_storable);

                // 本情報のVerticalTextを生成
                let vtext = VerticalText::new(
                    format!(
                        "{0: <4}{1: <6}{2}",
                        if is_storable { "可" } else { "不可" },
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

    ///
    /// 選択中の本のインデックスを降順でソート
    ///
    pub fn sort_selecting_index_less(&mut self) {
        self.selecting_book_index.sort_by(|a, b| b.cmp(a));
    }

    ///
    /// 現在選択中の本のインデックスのベクトルへの参照を返す
    ///
    pub fn get_selecting_index(&self) -> &Vec<usize> {
        &self.selecting_book_index
    }

    ///
    /// 現在選択中の本のインデックス情報をすべてクリア
    ///
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

        // 本情報のVerticalTextへのクリック処理
        for (index, vtext) in self.book_text.iter_mut().enumerate() {
            if *self.book_storable.get(index).unwrap() {
                // 本情報をクリックしたか？
                if vtext.contains(ctx, rpoint) {
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
                numeric::Rect::new(70.0, 50.0, 850.0, 600.0),
                "配架中",
                &book_shelf_info,
                shelving_book.clone(),
            ),
            shelving_books: shelving_book,
            stored_books: Vec::new(),
            store_button: SelectButton::new(
                ctx,
                numeric::Rect::new(1000.0, 200.0, 100.0, 50.0),
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
                numeric::Rect::new(1000.0, 500.0, 100.0, 50.0),
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
        self.select_book_window
            .update_contents(ctx, &self.book_shelf_info, &self.shelving_books);
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

        if self.select_book_window.contains(ctx, rpoint) {
            self.select_book_window
                .on_click(ctx, game_data, clock, button, rpoint);
        }

        if self.reset_select_button.contains(ctx, rpoint) {
            self.select_book_window.clear_selecting_index();
        }

        if self.store_button.contains(ctx, rpoint) {
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

///
/// メニューに表示するやつ
///
pub struct ShelvingDetailContents {
    canvas: MovableWrap<SubScreen>,
    menu_rect: numeric::Rect,
    title: VerticalText,
    cell_desc: VerticalText,
    book_info: Vec<VerticalText>,
    background: UniTexture,
}

impl ShelvingDetailContents {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        menu_rect: numeric::Rect,
        t: Clock,
    ) -> Self {
        let title = VerticalText::new(
            "配架中".to_string(),
            numeric::Point2f::new(menu_rect.w - 60.0, 70.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                game_data.get_font(FontID::JpFude1),
                numeric::Vector2f::new(40.0, 40.0),
                ggraphics::Color::from_rgba_u32(0xff),
            ),
        );

        let cell_desc = VerticalText::new(
            "請求番号\t\t表題".to_string(),
            numeric::Point2f::new(menu_rect.w - 120.0, 150.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                game_data.get_font(FontID::JpFude1),
                numeric::Vector2f::new(34.0, 34.0),
                ggraphics::Color::from_rgba_u32(0xff),
            ),
        );

	let background = UniTexture::new(
	    game_data.ref_texture(TextureID::MenuArt2),
	    numeric::Point2f::new(menu_rect.w - 1366.0, 0.0),
	    numeric::Vector2f::new(1.0, 1.0),
	    0.0,
	    0
	);

        ShelvingDetailContents {
            canvas: MovableWrap::new(
                Box::new(SubScreen::new(
                    ctx,
                    menu_rect,
                    0,
                    ggraphics::Color::from_rgba_u32(0xffffffff),
                )),
                None,
                t,
            ),
            menu_rect: menu_rect,
            title: title,
            cell_desc: cell_desc,
            book_info: Vec::new(),
	    background: background,
        }
    }

    pub fn update_contents(
        &mut self,
        game_data: &GameData,
        player_shelving: &Vec<BookInformation>,
    ) {
        self.book_info.clear();

        let mut book_info_position = numeric::Point2f::new(self.menu_rect.w - 180.0, 150.0);
        let book_font_information = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(30.0, 30.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        self.book_info = player_shelving
            .iter()
            .map(|book_info| {
                let vtext = VerticalText::new(
                    format!(
                        "{0:<6}{1}",
                        number_to_jk(book_info.billing_number as u64),
                        &book_info.name
                    ),
                    book_info_position,
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    book_font_information.clone(),
                );

                book_info_position.x -= 35.0;

                vtext
            })
            .collect();
    }

    ///
    /// 移動関数を変更しスライドインするように見せる
    ///
    pub fn slide_appear(&mut self, slide_position: numeric::Point2f, t: Clock) {
        self.canvas
            .override_move_func(move_fn::devide_distance(slide_position, 0.5), t);
    }

    ///
    /// 移動関数を変更しスライドアウトするように見せる
    ///
    pub fn slide_hide(&mut self, t: Clock) {
        self.canvas.override_move_func(
            move_fn::devide_distance(numeric::Point2f::new(-self.menu_rect.w, 0.0), 0.2),
            t,
        );
    }
}

impl Updatable for ShelvingDetailContents {
    fn update(&mut self, _: &ggez::Context, t: Clock) {
        self.canvas.move_with_func(t);
    }
}

impl DrawableComponent for ShelvingDetailContents {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, self.canvas.ref_wrapped_object());

	    self.background.draw(ctx)?;

            self.title.draw(ctx)?;
            self.cell_desc.draw(ctx)?;
            for vtext in &mut self.book_info {
                vtext.draw(ctx)?;
            }

            sub_screen::pop_screen(ctx);
            self.canvas.draw(ctx)?;
        }
        Ok(())
    }

    #[inline(always)]
    fn hide(&mut self) {
        self.canvas.hide();
    }

    #[inline(always)]
    fn appear(&mut self) {
        self.canvas.appear();
    }

    #[inline(always)]
    fn is_visible(&self) -> bool {
        self.canvas.is_visible()
    }

    #[inline(always)]
    fn set_drawing_depth(&mut self, depth: i8) {
        self.canvas.set_drawing_depth(depth);
    }

    #[inline(always)]
    fn get_drawing_depth(&self) -> i8 {
        self.canvas.get_drawing_depth()
    }
}

///
/// メニューに表示するやつ
///
pub struct ShopMenuContents {
    day_text: VerticalText,
    copy_request: VerticalText,
    copy_request_num: VerticalText,
    wait_for_return: VerticalText,
    wait_for_return_num: VerticalText,
    not_shelved: VerticalText,
    not_shelved_num: VerticalText,
    kosuzu_level: VerticalText,
    kosuzu_level_num: VerticalText,
    drwob_essential: DrawableObjectEssential,
}

impl ShopMenuContents {
    pub fn new(game_data: &GameData) -> Self {
        let normal_scale_font = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(26.0, 26.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );

        let large_scale_font = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(30.0, 30.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );

        ShopMenuContents {
            day_text: VerticalText::new(
                format!("日付　{}月 {}日", number_to_jk(12), number_to_jk(12)),
                numeric::Point2f::new(370.0, 50.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                large_scale_font,
            ),
            copy_request: VerticalText::new(
                format!("写本受注数"),
                numeric::Point2f::new(300.0, 50.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                normal_scale_font,
            ),
            copy_request_num: VerticalText::new(
                format!("{}件", number_to_jk(0)),
                numeric::Point2f::new(260.0, 150.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                large_scale_font,
            ),
            wait_for_return: VerticalText::new(
                format!("返却待冊数"),
                numeric::Point2f::new(200.0, 50.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                normal_scale_font,
            ),
            wait_for_return_num: VerticalText::new(
                format!("{}冊", number_to_jk(0)),
                numeric::Point2f::new(160.0, 150.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                large_scale_font,
            ),
            not_shelved: VerticalText::new(
                format!("未配架冊数"),
                numeric::Point2f::new(100.0, 50.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                normal_scale_font,
            ),
            not_shelved_num: VerticalText::new(
                format!("{}冊", number_to_jk(0)),
                numeric::Point2f::new(60.0, 150.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                large_scale_font,
            ),
            kosuzu_level: VerticalText::new(
                format!("小鈴 習熟度"),
                numeric::Point2f::new(300.0, 350.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                normal_scale_font,
            ),
            kosuzu_level_num: VerticalText::new(
                format!("{}", number_to_jk(0)),
                numeric::Point2f::new(260.0, 450.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                large_scale_font,
            ),
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }

    pub fn update_contents(&mut self, game_data: &GameData, task_result: &TaskResult) {
        let _normal_scale_font = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(26.0, 26.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );

        let large_scale_font = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(30.0, 30.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );

        self.day_text = VerticalText::new(
            format!("日付　{}月 {}日", number_to_jk(12), number_to_jk(12)),
            numeric::Point2f::new(370.0, 50.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            large_scale_font,
        );

        self.copy_request_num = VerticalText::new(
            format!(
                "{}件",
                number_to_jk(task_result.remain_copy_request.len() as u64)
            ),
            numeric::Point2f::new(260.0, 150.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            large_scale_font,
        );

        self.wait_for_return_num = VerticalText::new(
            format!(
                "{}冊",
                number_to_jk(task_result.borrowing_books.len() as u64)
            ),
            numeric::Point2f::new(160.0, 150.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            large_scale_font,
        );

        self.not_shelved_num = VerticalText::new(
            format!(
                "{}冊",
                number_to_jk(task_result.not_shelved_books.len() as u64)
            ),
            numeric::Point2f::new(60.0, 150.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            large_scale_font,
        );

        self.kosuzu_level_num = VerticalText::new(
            format!("{}", number_to_jk((task_result.done_works / 3) as u64)),
            numeric::Point2f::new(260.0, 450.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            large_scale_font,
        );
    }
}

impl DrawableComponent for ShopMenuContents {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.day_text.draw(ctx).unwrap();

            self.copy_request.draw(ctx).unwrap();
            self.copy_request_num.draw(ctx).unwrap();

            self.wait_for_return.draw(ctx).unwrap();
            self.wait_for_return_num.draw(ctx).unwrap();

            self.not_shelved.draw(ctx).unwrap();
            self.not_shelved_num.draw(ctx).unwrap();

            self.kosuzu_level.draw(ctx).unwrap();
            self.kosuzu_level_num.draw(ctx).unwrap();
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

pub struct ShopMenu {
    canvas: MovableWrap<SubScreen>,
    menu_contents: ShopMenuContents,
    background: UniTexture,
    menu_canvas_size: numeric::Vector2f,
    now_appear: bool,
}

impl ShopMenu {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        size: numeric::Vector2f,
        t: Clock,
    ) -> Self {
        ShopMenu {
            canvas: MovableWrap::new(
                Box::new(SubScreen::new(
                    ctx,
                    numeric::Rect::new(-size.x, 0.0, size.x, size.y),
                    0,
                    ggraphics::Color::from_rgba_u32(0xffffffff),
                )),
                None,
                t,
            ),
	    background: UniTexture::new(
		game_data.ref_texture(TextureID::MenuArt1),
		numeric::Point2f::new(size.x - 1366.0, 0.0),
		numeric::Vector2f::new(1.0, 1.0),
		0.0,
		0),
            menu_contents: ShopMenuContents::new(game_data),
            menu_canvas_size: size,
            now_appear: false,
        }
    }

    pub fn slide_toggle(&mut self, t: Clock) {
        if self.now_appear {
            self.canvas.override_move_func(
                move_fn::devide_distance(numeric::Point2f::new(-self.menu_canvas_size.x, 0.0), 0.5),
                t,
            );
            self.now_appear = false;
        } else {
            self.canvas.override_move_func(
                move_fn::devide_distance(numeric::Point2f::new(0.0, 0.0), 0.5),
                t,
            );
            self.now_appear = true;
        }
    }

    pub fn appearing_now(&self) -> bool {
        self.now_appear
    }

    pub fn update_menu_contents(&mut self, game_data: &GameData, task_result: &TaskResult) {
        self.menu_contents.update_contents(game_data, task_result);
    }
}

impl Updatable for ShopMenu {
    fn update(&mut self, _: &ggez::Context, t: Clock) {
        self.canvas.move_with_func(t);
    }
}

impl DrawableComponent for ShopMenu {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        sub_screen::stack_screen(ctx, self.canvas.ref_wrapped_object());

	self.background.draw(ctx)?;
        self.menu_contents.draw(ctx).unwrap();

        sub_screen::pop_screen(ctx);
        self.canvas.draw(ctx)
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

#[derive(PartialEq, Clone, Copy)]
pub enum ShopDetailMenuSymbol {
    ShelvingBooks = 0,
    SuzunaMap,
    None,
}

pub struct ShopDetailMenuContents {
    shelving_info: ShelvingDetailContents,
    drwob_essential: DrawableObjectEssential,
    contents_switch: ShopDetailMenuSymbol,
    appear_position: numeric::Point2f,
    now_appear: bool,
}

impl ShopDetailMenuContents {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        appear_position: numeric::Point2f,
        shelving_rect: numeric::Rect,
        t: Clock,
    ) -> Self {
        ShopDetailMenuContents {
            shelving_info: ShelvingDetailContents::new(ctx, game_data, shelving_rect, t),
            drwob_essential: DrawableObjectEssential::new(true, 0),
            contents_switch: ShopDetailMenuSymbol::None,
            appear_position: appear_position,
            now_appear: false,
        }
    }

    pub fn update_contents(
        &mut self,
        game_data: &GameData,
        player_shelving: &Vec<BookInformation>,
    ) {
        self.shelving_info
            .update_contents(game_data, player_shelving);
    }

    pub fn detail_menu_is_open(&self) -> bool {
        self.now_appear
    }

    pub fn hide_toggle(&mut self, t: Clock) {
        self.now_appear = false;
        self.shelving_info.slide_hide(t);
    }

    pub fn appear_toggle(&mut self, t: Clock) {
        self.now_appear = true;
        self.shelving_info.slide_appear(self.appear_position, t);
    }

    pub fn slide_toggle(&mut self, t: Clock) {
        match self.contents_switch {
            ShopDetailMenuSymbol::ShelvingBooks => {
                if self.now_appear {
                    self.hide_toggle(t);
                } else {
                    self.appear_toggle(t);
                }
            }
            ShopDetailMenuSymbol::SuzunaMap => {
                // まだ
            }
            ShopDetailMenuSymbol::None => (),
        }
    }

    pub fn set_slide_contents(&mut self, contents_switch: ShopDetailMenuSymbol) {
        self.contents_switch = contents_switch;
    }
}

impl Updatable for ShopDetailMenuContents {
    fn update(&mut self, ctx: &ggez::Context, t: Clock) {
        self.shelving_info.update(ctx, t);
    }
}

impl DrawableComponent for ShopDetailMenuContents {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.shelving_info.draw(ctx)?;
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

pub struct ShopMenuMaster {
    first_menu: ShopMenu,
    detail_menu: ShopDetailMenuContents,
    canvas: SubScreen,
}

impl ShopMenuMaster {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        first_menu_size: numeric::Vector2f,
        t: Clock,
    ) -> Self {
        ShopMenuMaster {
            first_menu: ShopMenu::new(ctx, game_data, first_menu_size, t),
            detail_menu: ShopDetailMenuContents::new(
                ctx,
                game_data,
                numeric::Point2f::new(first_menu_size.x, 0.0),
                numeric::Rect::new(-450.0, 0.0, 450.0, 768.0),
                t,
            ),
            canvas: SubScreen::new(
                ctx,
                numeric::Rect::new(0.0, 0.0, 1366.0, 768.0),
                0,
                ggraphics::Color::from_rgba_u32(0),
            ),
        }
    }

    pub fn update_contents(
        &mut self,
        game_data: &GameData,
        task_result: &TaskResult,
        player_shelving: &Vec<BookInformation>,
    ) {
        self.first_menu.update_menu_contents(game_data, task_result);
        self.detail_menu.update_contents(game_data, player_shelving);
    }

    pub fn toggle_first_menu(&mut self, t: Clock) {
        self.first_menu.slide_toggle(t);
        if !self.first_menu_is_open() {
            self.detail_menu.hide_toggle(t);
        }
    }

    pub fn first_menu_is_open(&self) -> bool {
        self.first_menu.appearing_now()
    }

    pub fn menu_key_action(
        &mut self,
        _: &mut ggez::Context,
        _: &GameData,
        vkey: VirtualKey,
        t: Clock,
    ) {
        match vkey {
            VirtualKey::Action3 => {
                if self.first_menu_is_open() {
                    debug::debug_screen_push_text("slide detail menu");
                    self.detail_menu
                        .set_slide_contents(ShopDetailMenuSymbol::ShelvingBooks);
                    self.detail_menu.slide_toggle(t);
                }
            }
            _ => (),
        }
    }
}

impl Updatable for ShopMenuMaster {
    fn update(&mut self, ctx: &ggez::Context, t: Clock) {
        self.first_menu.update(ctx, t);
        self.detail_menu.update(ctx, t);
    }
}

impl DrawableComponent for ShopMenuMaster {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.detail_menu.draw(ctx)?;
            self.first_menu.draw(ctx)?;

            sub_screen::pop_screen(ctx);
            self.canvas.draw(ctx)?;
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

pub struct ShopSpecialObject {
    event_list: DelayEventList<Self>,
    shelving_select_ui: Option<MovableWrap<SelectShelvingBookUI>>,
    storing_select_ui: Option<MovableWrap<SelectStoreBookUI>>,
    drwob_essential: DrawableObjectEssential,
}

impl ShopSpecialObject {
    pub fn new() -> Self {
        ShopSpecialObject {
            event_list: DelayEventList::new(),
            shelving_select_ui: None,
            storing_select_ui: None,
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }

    pub fn show_shelving_select_ui(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        task_result: &TaskResult,
        player_shelving_books: Vec<BookInformation>,
        t: Clock,
    ) {
        if self.shelving_select_ui.is_none() {
            self.shelving_select_ui = Some(MovableWrap::new(
                Box::new(SelectShelvingBookUI::new(
                    ctx,
                    game_data,
                    numeric::Rect::new(0.0, -768.0, 1366.0, 768.0),
                    task_result.not_shelved_books.clone(),
                    player_shelving_books,
                )),
                move_fn::devide_distance(numeric::Point2f::new(0.0, 0.0), 0.4),
                t,
            ));
        }
    }

    pub fn show_storing_select_ui(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        book_shelf_info: BookShelfInformation,
        player_shelving_books: Vec<BookInformation>,
        t: Clock,
    ) {
        if self.storing_select_ui.is_none() {
            self.storing_select_ui = Some(MovableWrap::new(
                Box::new(SelectStoreBookUI::new(
                    ctx,
                    game_data,
                    numeric::Rect::new(0.0, -768.0, 1366.0, 768.0),
                    book_shelf_info,
                    player_shelving_books,
                )),
                move_fn::devide_distance(numeric::Point2f::new(0.0, 0.0), 0.4),
                t,
            ));
        }
    }

    pub fn hide_shelving_select_ui(
        &mut self,
        t: Clock,
    ) -> Option<(Vec<BookInformation>, Vec<BookInformation>)> {
        if let Some(ui) = self.shelving_select_ui.as_mut() {
            ui.override_move_func(
                move_fn::devide_distance(numeric::Point2f::new(0.0, -768.0), 0.4),
                t,
            );
            self.event_list.add_event(
                Box::new(|shop_special_object, _, _| {
                    shop_special_object.shelving_select_ui = None;
                }),
                t + 7,
            );

            Some(ui.ref_wrapped_object().get_select_result())
        } else {
            None
        }
    }

    pub fn hide_storing_select_ui(
        &mut self,
        t: Clock,
    ) -> Option<(Vec<BookInformation>, Vec<BookInformation>)> {
        if let Some(ui) = self.storing_select_ui.as_mut() {
            ui.override_move_func(
                move_fn::devide_distance(numeric::Point2f::new(0.0, -768.0), 0.4),
                t,
            );
            self.event_list.add_event(
                Box::new(|shop_special_object, _, _| {
                    shop_special_object.storing_select_ui = None;
                }),
                t + 7,
            );

            Some(ui.ref_wrapped_object().get_storing_result())
        } else {
            None
        }
    }

    pub fn is_enable_now(&self) -> bool {
        self.shelving_select_ui.is_some() || self.storing_select_ui.is_some()
    }

    pub fn mouse_down_action(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        button: MouseButton,
        point: numeric::Point2f,
        t: Clock,
    ) {
        if let Some(ui) = self.shelving_select_ui.as_mut() {
            ui.ref_wrapped_object_mut()
                .on_click(ctx, game_data, t, button, point);
        }

        if let Some(ui) = self.storing_select_ui.as_mut() {
            ui.ref_wrapped_object_mut()
                .on_click(ctx, game_data, t, button, point);
        }
    }

    pub fn run_delay_event(&mut self, ctx: &mut ggez::Context, game_data: &GameData, t: Clock) {
        // 最後の要素の所有権を移動
        while let Some(event) = self.event_list.move_top() {
            // 時間が来ていない場合は、取り出した要素をリストに戻して処理ループを抜ける
            if event.run_time > t {
                self.event_list.add(event);
                break;
            }

            // 所有権を移動しているため、selfを渡してもエラーにならない
            (event.func)(self, ctx, game_data);
        }
    }
}

impl Updatable for ShopSpecialObject {
    fn update(&mut self, _: &ggez::Context, t: Clock) {
        if let Some(ui) = self.shelving_select_ui.as_mut() {
            ui.move_with_func(t);
        }

        if let Some(storing_ui) = self.storing_select_ui.as_mut() {
            storing_ui.move_with_func(t);
        }
    }
}

impl DrawableComponent for ShopSpecialObject {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            if let Some(select_ui) = self.shelving_select_ui.as_mut() {
                select_ui.draw(ctx)?;
            }

            if let Some(store_ui) = self.storing_select_ui.as_mut() {
                store_ui.draw(ctx)?;
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
