use std::rc::Rc;

use ggez::graphics as ggraphics;

use torifune::core::Clock;
use torifune::numeric;
use torifune::graphics::*;
use torifune::graphics::object::*;
use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;
use torifune::graphics::object::sub_screen;
use sub_screen::SubScreen;
use torifune::debug;
use torifune::device::*;

use crate::core::{BookInformation, GameData, FontID};
use crate::object::Clickable;

pub struct SelectShelvingBookWindow {
    canvas: SubScreen,
    title: VerticalText,
    book_text: Vec<VerticalText>,
    selecting_book_index: Vec<usize>,
    book_font: FontInformation,
}

impl SelectShelvingBookWindow {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData, window_rect: numeric::Rect,
	       title: &str, book_info: Vec<BookInformation>) -> Self {

	let font_info = FontInformation::new(game_data.get_font(FontID::JpFude1),
					     numeric::Vector2f::new(32.0, 32.0),
					     ggraphics::Color::from_rgba_u32(0xff));
	
	let mut window = SelectShelvingBookWindow {
	    canvas: SubScreen::new(ctx, window_rect, 0, ggraphics::Color::from_rgba_u32(0xeeeeeeff)),
	    title: VerticalText::new(title.to_string(), numeric::Point2f::new(window_rect.w - 50.0, 50.0),
				     numeric::Vector2f::new(1.0, 1.0), 0.0, 0, font_info),
	    book_text: Vec::new(),
	    selecting_book_index: Vec::new(),
	    book_font: font_info,
	};

	window.update_contents(ctx, &book_info);
	
	window
    }

    fn update_contents(&mut self, ctx: &mut ggez::Context, book_info: &Vec<BookInformation>) {
	let window_rect = self.canvas.get_drawing_area(ctx);
	let mut text_position = numeric::Point2f::new(window_rect.w - 100.0, 50.0);
	
	self.book_text = book_info
	    .iter()
	    .enumerate()
	    .map(|enumerate_data| {
		let (index, info) = enumerate_data;

		if index == 8 {
		    text_position.x = window_rect.w - 100.0;
		    text_position.y += 500.0;
		}
		
		let vtext = VerticalText::new(info.name.to_string(), text_position,
					      numeric::Vector2f::new(1.0,1.0), 0.0, 0, self.book_font.clone());
		text_position.x -= 40.0;
		vtext
	    })
    	    .collect();
    }

    pub fn get_selecting_index(&self) -> &Vec<usize> {
	&self.selecting_book_index
    }

    pub fn clear_selecting_index(&mut self) {
	self.selecting_book_index.clear();
    }
}

impl DrawableComponent for SelectShelvingBookWindow {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
	    sub_screen::stack_screen(ctx, &self.canvas);

	    self.title.draw(ctx)?;
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

impl DrawableObject for SelectShelvingBookWindow {
    impl_drawable_object_for_wrapped!{canvas}
}

impl TextureObject for SelectShelvingBookWindow {
    impl_texture_object_for_wrapped!{canvas}
}

impl Clickable for SelectShelvingBookWindow {
    fn on_click(&mut self,
                ctx: &mut ggez::Context,
		_: &GameData,
		_: Clock,
                _button: ggez::input::mouse::MouseButton,
                point: numeric::Point2f) {
	let rpoint = self.canvas.relative_point(point);
	for (index, vtext) in self.book_text.iter_mut().enumerate() {
	    if vtext.get_drawing_area(ctx).contains(rpoint) {
		vtext.set_color(ggraphics::Color::from_rgba_u32(0xee0000ff));
		self.selecting_book_index.push(index);
		break;
	    }
	}

	debug::debug_screen_push_text(&format!("window select text: {:?}", self.selecting_book_index));
    }

    fn clickable_status(&mut self,
			_ctx: &mut ggez::Context,
			_point: numeric::Point2f) -> ggez::input::mouse::MouseCursor {
	ggez::input::mouse::MouseCursor::Default
    }
}

pub struct SelectShelvingBookUI {
    canvas: SubScreen,
    boxed_books: Vec<BookInformation>,
    shelving_books: Vec<BookInformation>,
    box_info_window: SelectShelvingBookWindow,
    shelving_window: SelectShelvingBookWindow,
}

impl SelectShelvingBookUI {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData, ui_rect: numeric::Rect,
	       box_book_info: Vec<BookInformation>, shelving_book: Vec<BookInformation>) -> Self {
	SelectShelvingBookUI {
	    canvas: SubScreen::new(ctx, ui_rect, 0, ggraphics::Color::from_rgba_u32(0)),
	    box_info_window: SelectShelvingBookWindow::new(ctx, game_data, numeric::Rect::new(70.0, 50.0, 600.0, 600.0),
							   "返却済み", box_book_info.clone()),
	    shelving_window: SelectShelvingBookWindow::new(ctx, game_data, numeric::Rect::new(720.0, 50.0, 600.0, 600.0),
							   "配架中", shelving_book.clone()),
	    boxed_books: box_book_info,
	    shelving_books: shelving_book,
	}
    }

    fn update_window(&mut self, ctx: &mut ggez::Context) {
	self.box_info_window.update_contents(ctx, &self.boxed_books);
	self.shelving_window.update_contents(ctx, &self.shelving_books);
    }

    fn move_box_to_shelving(&mut self, ctx: &mut ggez::Context) {
	for selecting_index in self.box_info_window.get_selecting_index().iter() {
	    debug::debug_screen_push_text(&format!("remove book: {}", selecting_index));
	    let select_book = self.boxed_books.swap_remove(*selecting_index);
	    self.shelving_books.push(select_book);
	}

	self.box_info_window.clear_selecting_index();
	self.update_window(ctx);
    }
}

impl DrawableComponent for SelectShelvingBookUI {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
	    sub_screen::stack_screen(ctx, &self.canvas);

	    self.box_info_window.draw(ctx)?;
	    self.shelving_window.draw(ctx)?;
	    
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

    fn virtual_key_event(&mut self, ctx: &mut ggez::Context, event_type: KeyboardEvent, vkey: VirtualKey) {
	match vkey {
	    VirtualKey::Action1 => {
		if event_type == KeyboardEvent::FirstPressed {
		    self.move_box_to_shelving(ctx);
		}
	    },
	    _ =>  (),
	}
    }
}

impl DrawableObject for SelectShelvingBookUI {
    impl_drawable_object_for_wrapped!{canvas}
}

impl TextureObject for SelectShelvingBookUI {
    impl_texture_object_for_wrapped!{canvas}
}

impl Clickable for SelectShelvingBookUI {
    fn on_click(&mut self,
                ctx: &mut ggez::Context,
		game_data: &GameData,
		clock: Clock,
                button: ggez::input::mouse::MouseButton,
                point: numeric::Point2f) {
	let rpoint = self.canvas.relative_point(point);
	
	if self.box_info_window.get_drawing_area(ctx).contains(rpoint) {
	    self.box_info_window.on_click(ctx, game_data, clock, button, rpoint);
	}

	if self.shelving_window.get_drawing_area(ctx).contains(rpoint) {
	    self.shelving_window.on_click(ctx, game_data, clock, button, rpoint);
	}
    }

    fn clickable_status(&mut self,
			_ctx: &mut ggez::Context,
			_point: numeric::Point2f) -> ggez::input::mouse::MouseCursor {
	ggez::input::mouse::MouseCursor::Default
    }
}
