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
    slot_id: u8,
    background: UniTexture,
    date_text: Option<VerticalText>,
    money_text: Option<VerticalText>,
    none_text: Option<VerticalText>,
    canvas: SubScreen,
}

impl DrawableSaveEntry {
    pub fn new<'a>(
	ctx: &mut SuzuContext<'a>,
	savable_data: Option<SavableData>,
        pos_rect: numeric::Rect,
	slot_id: u8,
    ) -> Self {
	let background = UniTexture::new(
	    ctx.resource.ref_texture(TextureID::Wood1),
	    numeric::Point2f::new(0.0, 0.0),
	    numeric::Vector2f::new(1.0, 1.0),
	    0.0,
	    0
	);
	
	if let Some(savable_data) = savable_data {
	    Self::new_some(ctx, background, savable_data, pos_rect, slot_id)
	} else {
	    Self::new_none(ctx, background, pos_rect, slot_id)
	}
    }

    fn new_some<'a>(
    	ctx: &mut SuzuContext<'a>,
	background: UniTexture,
	savable_data: SavableData,
        pos_rect: numeric::Rect,
	slot_id: u8,
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
	    background: background,
	    date_text: Some(date_text),
	    money_text:  Some(money_text),
	    none_text: None,
	    canvas: SubScreen::new(ctx.context, pos_rect, 0, ggraphics::Color::from_rgba_u32(0)),
	    slot_id: slot_id,
        }
    }

    fn new_none<'a>(
        ctx: &mut SuzuContext<'a>,
	background: UniTexture,
        pos_rect: numeric::Rect,
	slot_id: u8,
    ) -> Self {
	let mut none_text = VerticalText::new(
	    "記録ガ在リマセン".to_string(),
	    numeric::Point2f::new(0.0, 0.0),
	    numeric::Vector2f::new(1.0, 1.0),
	    0.0,
	    0,
	    FontInformation::new(
		ctx.resource.get_font(FontID::JpFude1),
		numeric::Vector2f::new(40.0, 40.0),
		ggraphics::Color::from_rgba_u32(0xff)
	    ),
	);

	none_text.make_center(ctx.context, numeric::Point2f::new(pos_rect.w / 2.0, pos_rect.h / 2.0));
	
        DrawableSaveEntry {
	    background: background,
	    date_text: None,
	    money_text:  None,
	    none_text: Some(none_text),
	    canvas: SubScreen::new(ctx.context, pos_rect, 0, ggraphics::Color::from_rgba_u32(0)),
	    slot_id: slot_id,
        }
    }

    pub fn save_action<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
	ctx.savable_data.save(self.slot_id);

	let savable_data = &ctx.savable_data;

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

	self.date_text = Some(date_text);
	self.money_text = Some(money_text);
	self.none_text = None;
    }
}

impl DrawableComponent for DrawableSaveEntry {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
	    sub_screen::stack_screen(ctx, &self.canvas);

	    self.background.draw(ctx)?;
	    
	    if let Some(vtext) = self.date_text.as_mut() {
		vtext.draw(ctx)?;
	    }

	    if let Some(vtext) = self.money_text.as_mut() {
		vtext.draw(ctx)?;
	    }

	    if let Some(vtext) = self.none_text.as_mut() {
		vtext.draw(ctx)?;
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

impl DrawableObject for DrawableSaveEntry {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for DrawableSaveEntry {
    impl_texture_object_for_wrapped! {canvas}
}

pub struct SaveEntryTable {
    canvas: SubScreen,
    background: UniTexture,
    appearance_frame: TileBatchFrame,
    entries: Vec<DrawableSaveEntry>,
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
	    numeric::Rect::new(0.0, 0.0, window_rect.w, window_rect.h),
	    numeric::Vector2f::new(0.8, 0.8),
	    0
	);

	let mut entries = Vec::new();
	let mut pos_rect = numeric::Rect::new(50.0, 125.0, 280.0, 500.0);

	for (index, maybe_save_data) in save_data_list.iter().enumerate() {
	    entries.push(
		DrawableSaveEntry::new(ctx, maybe_save_data.clone(), pos_rect, index as u8)
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
		numeric::Vector2f::new(35.0, 35.0),
		ggraphics::Color::from_rgba_u32(0xff)
	    ),
	);

	title_text.make_center(ctx.context, numeric::Point2f::new(window_rect.w / 2.0, 70.0));

	let background = UniTexture::new(
	    ctx.resource.ref_texture(TextureID::Paper1),
	    numeric::Point2f::new(0.0, 0.0),
	    numeric::Vector2f::new(1.4, 1.4),
	    0.0,
	    0
	);
	
	SaveEntryTable {
	    canvas: SubScreen::new(ctx.context, window_rect, draw_depth, ggraphics::Color::from_rgba_u32(0)),
	    background: background,
	    appearance_frame: appr_frame,
	    entries: entries,
	    title_text: title_text,
	}
    }

    pub fn click_handler<'a>(
	&mut self,
	ctx: &mut SuzuContext<'a>,
	point: numeric::Point2f,
    ) {
	let rpoint = self.canvas.relative_point(point);

	for entry in self.entries.iter_mut() {
	    if entry.contains(ctx.context, rpoint) {
		entry.save_action(ctx);
	    }
	}
    }
}

impl DrawableComponent for SaveEntryTable {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

	    self.background.draw(ctx)?;
            self.appearance_frame.draw(ctx)?;

	    for entry in self.entries.iter_mut() {
		entry.draw(ctx)?;
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
