use std::rc::Rc;

use ggez::graphics as ggraphics;

use torifune::graphics::drawable::*;
use torifune::graphics::object::*;
use torifune::graphics::object::sub_screen;
use sub_screen::SubScreen;
use torifune::numeric;

use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;

use crate::core::*;
use crate::object::util_object::*;

use number_to_jk::number_to_jk;

pub struct DrawableSaveEntry {
    date_text: VerticalText,
    money_text: VerticalText,
    canvas: SubScreen,
}

impl DrawableSaveEntry {
    pub fn new<'a>(
	ctx: &mut SuzuContext<'a>,
	savable_data: SavableData,
        pos_rect: numeric::Rect,
    ) -> Self {
	let date_text = VerticalText::new(
	    format!("{}月{}日", number_to_jk(savable_data.date.month as u64), number_to_jk(savable_data.date.day as u64)),
	    numeric::Point2f::new(300.0, 50.0),
	    numeric::Vector2f::new(1.0, 1.0),
	    0.0,
	    0,
	    FontInformation::new(
		ctx.resource.get_font(FontID::JpFude1),
		numeric::Vector2f::new(35.0, 35.0),
		ggraphics::Color::from_rgba_u32(0xff)
	    ),
	);

	let money_text = VerticalText::new(
	    format!("所持金{}円", number_to_jk(savable_data.task_result.total_money as u64)),
	    numeric::Point2f::new(250.0, 150.0),
	    numeric::Vector2f::new(1.0, 1.0),
	    0.0,
	    0,
	    FontInformation::new(
		ctx.resource.get_font(FontID::JpFude1),
		numeric::Vector2f::new(35.0, 35.0),
		ggraphics::Color::from_rgba_u32(0xff)
	    ),
	);

	
        DrawableSaveEntry {
	    date_text: date_text,
	    money_text:  money_text,
	    canvas: SubScreen::new(ctx.context, pos_rect, 0, ggraphics::Color::from_rgba_u32(0)),
        }
    }
}

impl DrawableComponent for DrawableSaveEntry {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
	    sub_screen::stack_screen(ctx, &self.canvas);

	    self.date_text.draw(ctx)?;
	    self.money_text.draw(ctx)?;

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

pub struct SaveEntryTable {
    canvas: SubScreen,
    appearance_frame: TileBatchFrame,
    entries: Vec<Option<DrawableSaveEntry>>,
    title_text: UniText,
}

impl SaveEntryTable {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        window_rect: numeric::Rect,
	save_data_list: Vec<Option<SavableData>>,
	draw_depth: i8,
    ) -> Self {

	let appr_frame = TileBatchFrame::new(
	    ctx.resource,
	    TileBatchTextureID::TaishoStyle1,
	    window_rect,
	    numeric::Vector2f::new(0.8, 0.8),
	    0
	);

	let mut entries = Vec::new();
	let mut pos_rect = numeric::Rect::new(50.0, 100.0, 280.0, 550.0);

	for maybe_save_data in save_data_list {
	    entries.push(
		if let Some(save_data) = maybe_save_data {
		    Some(DrawableSaveEntry::new(ctx, save_data, pos_rect))
		} else {
		    None
		}
	    );

	    pos_rect.x += 300.0;
	}

	let mut title_text = UniText::new(
	    "鈴奈庵営業記録".to_string(),
	    numeric::Point2f::new(0.0, 0.0),
	    numeric::Vector2f::new(1.0, 1.0),
	    0.0,
	    0,
	    FontInformation::new(
		ctx.resource.get_font(FontID::JpFude1),
		numeric::Vector2f::new(30.0, 30.0),
		ggraphics::Color::from_rgba_u32(0xff)
	    ),
	);

	title_text.make_center(ctx.context, numeric::Point2f::new(window_rect.w / 2.0, 50.0));
	
	SaveEntryTable {
	    canvas: SubScreen::new(ctx.context, window_rect, draw_depth, ggraphics::Color::from_rgba_u32(0)),
	    appearance_frame: appr_frame,
	    entries: entries,
	    title_text: title_text,
	}
    }

}

impl DrawableComponent for SaveEntryTable {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.appearance_frame.draw(ctx)?;

	    for maybe_entry in self.entries.iter_mut() {
		if let Some(entry) = maybe_entry {
		    entry.draw(ctx)?;
		}
	    }

	    self.title_text.draw(ctx)?;
	    
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

impl DrawableObject for DrawableSaveEntry {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for DrawableSaveEntry {
    impl_texture_object_for_wrapped! {canvas}
}
