use ggez::graphics as ggraphics;

use torifune::graphics::object::sub_screen;
use sub_screen::SubScreen;
use torifune::numeric;
use torifune::graphics::object::*;
use torifune::graphics::drawable::{DrawableComponent, DrawableObjectEssential};

use crate::core::{GameData, FontID};

pub trait NotificationContents: DrawableComponent {
    fn required_size(&self) -> numeric::Vector2f;
}

pub struct CustomerCallNotification {
    main_text: VerticalText,
    header_text: UniText,
    drwob_essential: DrawableObjectEssential,
}

impl CustomerCallNotification {
    pub fn new(game_data: &GameData, depth: i8) -> Self {
	CustomerCallNotification {
	    main_text: VerticalText::new(
		"お客が呼んでいます".to_string(),
		numeric::Point2f::new(50.0, 10.0),
		numeric::Vector2f::new(1.0, 1.0),
		0.0,
		0,
		FontInformation::new(
		    game_data.get_font(FontID::JpFude1),
		    numeric::Vector2f::new(23.0, 23.0),
		    ggraphics::Color::from_rgba_u32(0xff)),
	    ),
	    header_text: UniText::new(
		"セラ知オ".to_string(),
		numeric::Point2f::new(10.0, 10.0),
		numeric::Vector2f::new(1.0, 1.0),
		0.0,
		0,
		FontInformation::new(
		    game_data.get_font(FontID::JpFude1),
		    numeric::Vector2f::new(23.0, 23.0),
		    ggraphics::Color::from_rgba_u32(0xff)),
	    ),
	    drwob_essential: DrawableObjectEssential::new(true, depth),
	}
    }
}

impl DrawableComponent for CustomerCallNotification {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
	if self.is_visible() {
	    self.main_text.draw(ctx)?;
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

impl NotificationContents for CustomerCallNotification {
    fn required_size(&self) -> numeric::Vector2f {
	numeric::Vector2f::new(100.0, 180.0)
    }
}

pub struct NotificationArea {
    right_top_position: numeric::Point2f,
    contents: Option<Box<dyn NotificationContents>>,
    area: Option<SubScreen>,
    drwob_essential: DrawableObjectEssential,
}

impl NotificationArea {
    pub fn new(right_top_position: numeric::Point2f, depth: i8) -> Self {
	NotificationArea {
	    right_top_position: right_top_position,
	    contents: None,
	    area: None,
	    drwob_essential: DrawableObjectEssential::new(true, depth)
	}
    }

    pub fn insert_new_contents(&mut self, ctx: &mut ggez::Context, contents: Box<dyn NotificationContents>) {
	self.contents = Some(contents);
	self.update_area_canvas(ctx);
    }

    fn update_area_canvas(&mut self, ctx: &mut ggez::Context) {
	let area_size = self.contents.as_ref().unwrap().required_size();
	self.area = Some(
	    SubScreen::new(
		ctx,
		numeric::Rect::new(self.right_top_position.x - area_size.x , 10.0, area_size.x, area_size.y),
		0,
		ggraphics::Color::from_rgba_u32(0),
	    )
	);
    }
}

impl DrawableComponent for NotificationArea {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
	if self.is_visible() {
	    if let Some(canvas) = self.area.as_mut() {
		sub_screen::stack_screen(ctx, canvas);

		if let Some(contents) = self.contents.as_mut() {
		    contents.draw(ctx)?;
		}

		sub_screen::pop_screen(ctx);
		canvas.draw(ctx).unwrap();
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
