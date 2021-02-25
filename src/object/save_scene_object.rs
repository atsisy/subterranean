use ggez::graphics as ggraphics;

use sub_screen::SubScreen;
use torifune::graphics::drawable::*;
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::*;
use torifune::numeric;
use torifune::roundup2f;

use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;

use crate::{core::*, set_table_frame_cell_center};
use crate::object::util_object::*;

use number_to_jk::number_to_jk;

pub enum SaveDataOperation {
    Saving,
    Deleting,
    Loading(u8),
    NoOperation,
}

pub struct DrawableSaveEntry {
    slot_id: u8,
    background: UniTexture,
    desc_text: Vec<VerticalText>,
    date_text: Option<VerticalText>,
    money_text: Option<VerticalText>,
    table_frame: TableFrame,
    save_button: SelectButton,
    delete_button: SelectButton,
    load_button: SelectButton,
    canvas: SubScreen,
    appearance_frame: TileBatchFrame,
}

impl DrawableSaveEntry {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        texture_id: TextureID,
        savable_data: Option<SavableData>,
        pos_rect: numeric::Rect,
        slot_id: u8,
    ) -> Self {
        let mut background = UniTexture::new(
            ctx.ref_texture(texture_id),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
        );

        let button_texture = Box::new(TextButtonTexture::new(
            ctx,
            numeric::Point2f::new(0.0, 0.0),
            "記録".to_string(),
            FontInformation::new(
                ctx.resource.get_font(FontID::Cinema),
                numeric::Vector2f::new(18.0, 18.0),
                ggraphics::Color::from_rgba_u32(0xf6e1d5ff),
            ),
            5.0,
            ggraphics::Color::from_rgba_u32(0x5a4f3fff),
            0,
        ));

        let save_button = SelectButton::new(
            ctx,
            numeric::Rect::new(30.0, pos_rect.h - 70.0, 60.0, 60.0),
            button_texture,
        );

        let button_texture = Box::new(TextButtonTexture::new(
            ctx,
            numeric::Point2f::new(0.0, 0.0),
            "削除".to_string(),
            FontInformation::new(
                ctx.resource.get_font(FontID::Cinema),
                numeric::Vector2f::new(18.0, 18.0),
                ggraphics::Color::from_rgba_u32(0xf6e1d5ff),
            ),
            5.0,
            ggraphics::Color::from_rgba_u32(0x5a4f3fff),
            0,
        ));

        let delete_button = SelectButton::new(
            ctx,
            numeric::Rect::new(110.0, pos_rect.h - 70.0, 60.0, 60.0),
            button_texture,
        );

        let button_texture = Box::new(TextButtonTexture::new(
            ctx,
            numeric::Point2f::new(0.0, 0.0),
            "再開".to_string(),
            FontInformation::new(
                ctx.resource.get_font(FontID::Cinema),
                numeric::Vector2f::new(18.0, 18.0),
                ggraphics::Color::from_rgba_u32(0xf6e1d5ff),
            ),
            5.0,
            ggraphics::Color::from_rgba_u32(0x5a4f3fff),
            0,
        ));

        let load_button = SelectButton::new(
            ctx,
            numeric::Rect::new(190.0, pos_rect.h - 70.0, 60.0, 60.0),
            button_texture,
        );

        let drawing_size = background.get_drawing_size(ctx.context);
        background.set_crop(numeric::Rect::new(
            0.0,
            0.0,
            (pos_rect.w - 16.0) / drawing_size.x,
            (pos_rect.h - 16.0) / drawing_size.y,
        ));
        let appr_frame = TileBatchFrame::new(
            ctx.resource,
            TileBatchTextureID::BlackFrame2,
            numeric::Rect::new(0.0, 0.0, pos_rect.w, pos_rect.h),
            numeric::Vector2f::new(0.25, 0.25),
            0,
        );

	let mut table_frame = TableFrame::new(
	    ctx.resource,
	    numeric::Point2f::new(20.0, 20.0),
	    TileBatchTextureID::OldStyleFrame,
	    FrameData::new(vec![80.0, 300.0], vec![50.0; 3]),
	    numeric::Vector2f::new(0.3, 0.3),
	    0
	);

	table_frame.make_center(numeric::Point2f::new(pos_rect.w / 2.0 - 10.0, pos_rect.h / 2.0 - 30.0));

        if let Some(savable_data) = savable_data {
            Self::new_some(
                ctx,
                background,
                appr_frame,
                savable_data,
                pos_rect,
		table_frame,
                save_button,
                load_button,
                delete_button,
                slot_id,
            )
        } else {
            Self::new_none(
                ctx,
                background,
                appr_frame,
                pos_rect,
		table_frame,
                save_button,
                load_button,
                delete_button,
                slot_id,
            )
        }
    }

    fn new_some<'a>(
        ctx: &mut SuzuContext<'a>,
        background: UniTexture,
        appr_frame: TileBatchFrame,
        savable_data: SavableData,
        pos_rect: numeric::Rect,
	table_frame: TableFrame,
        save_button: SelectButton,
        load_button: SelectButton,
        delete_button: SelectButton,
        slot_id: u8,
    ) -> Self {
        let mut entry = DrawableSaveEntry {
            background: background,
            date_text: None,
            money_text: None,
	    desc_text: Vec::new(),
            save_button: save_button,
            load_button: load_button,
            delete_button: delete_button,
            appearance_frame: appr_frame,
            canvas: SubScreen::new(ctx.context, pos_rect, 0, ggraphics::Color::from_rgba_u32(0)),
            slot_id: slot_id,
	    table_frame: table_frame,
        };

        entry.update_entry_contents(ctx.context, ctx.resource, &savable_data);
        entry
    }

    fn new_none<'a>(
        ctx: &mut SuzuContext<'a>,
        background: UniTexture,
        appr_frame: TileBatchFrame,
        pos_rect: numeric::Rect,
	table_frame: TableFrame,
        save_button: SelectButton,
        load_button: SelectButton,
        delete_button: SelectButton,
        slot_id: u8,
    ) -> Self {
        let mut entry = DrawableSaveEntry {
            background: background,
            date_text: None,
            money_text: None,
	    desc_text: Vec::new(),
            save_button: save_button,
            load_button: load_button,
            delete_button: delete_button,
            appearance_frame: appr_frame,
            canvas: SubScreen::new(ctx.context, pos_rect, 0, ggraphics::Color::from_rgba_u32(0)),
            slot_id: slot_id,
	    table_frame: table_frame,
        };

        entry.update_none_contents(ctx);
        entry
    }

    fn update_entry_contents(&mut self, ctx: &mut ggez::Context, resource: &GameResource, savable_data: &SavableData) {
	self.desc_text.clear();

	let mut mode_desc_text = VerticalText::new(
	    "規則".to_string(),
            numeric::Point2f::new(220.0, 60.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                resource.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0xff),
            ),
        );

	set_table_frame_cell_center!(
            ctx,
            self.table_frame,
            mode_desc_text,
            numeric::Vector2u::new(1, 0)
        );
	self.desc_text.push(mode_desc_text);
	
	let mut mode_text = VerticalText::new(
	    savable_data.game_mode.to_str_jp().to_string(),
            numeric::Point2f::new(220.0, 60.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                resource.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0xff),
            ),
        );

	set_table_frame_cell_center!(
            ctx,
            self.table_frame,
            mode_text,
            numeric::Vector2u::new(1, 1)
        );
	self.desc_text.push(mode_text);
	
	let mut day_desc_text = VerticalText::new(
	    "日付".to_string(),
            numeric::Point2f::new(220.0, 60.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                resource.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0xff),
            ),
        );

	set_table_frame_cell_center!(
            ctx,
            self.table_frame,
            day_desc_text,
            numeric::Vector2u::new(2, 0)
        );
	self.desc_text.push(day_desc_text);
	
        let mut date_text = VerticalText::new(
            format!(
                "{}月{}日",
                number_to_jk(savable_data.date.month as u64),
                number_to_jk(savable_data.date.day as u64)
            ),
            numeric::Point2f::new(220.0, 60.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                resource.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0xff),
            ),
        );
	set_table_frame_cell_center!(
            ctx,
            self.table_frame,
            date_text,
            numeric::Vector2u::new(2, 1)
        );

	let mut money_desc_text = VerticalText::new(
	    "所持金".to_string(),
            numeric::Point2f::new(220.0, 60.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                resource.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0xff),
            ),
        );

	set_table_frame_cell_center!(
            ctx,
            self.table_frame,
            money_desc_text,
            numeric::Vector2u::new(0, 0)
        );
	self.desc_text.push(money_desc_text);
	
        let mut money_text = VerticalText::new(
            format!(
                "{}円",
                number_to_jk(savable_data.task_result.total_money as u64)
            ),
            numeric::Point2f::new(150.0, 60.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                resource.get_font(FontID::JpFude1),
                numeric::Vector2f::new(18.0, 18.0),
                ggraphics::Color::from_rgba_u32(0xff),
            ),
        );
	set_table_frame_cell_center!(
            ctx,
            self.table_frame,
            money_text,
            numeric::Vector2u::new(0, 1)
        );

        self.date_text = Some(date_text);
        self.money_text = Some(money_text);
    }

    fn update_none_contents<'a>(&mut self, _ctx: &mut SuzuContext<'a>) {
        self.date_text = None;
        self.money_text = None;
    }

    fn save_action<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
	match ctx.save(self.slot_id) {
	    Err(_) => return,
	    _ => (),
	}

	if let Some(data) = ctx.savable_data.as_mut() {
	    self.update_entry_contents(ctx.context, ctx.resource, data);
	}

	ctx.process_utility.redraw();
    }

    fn delete_action<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        SavableData::delete(self.slot_id);
        self.update_none_contents(ctx);
	self.desc_text.clear();
    }

    pub fn click_handler<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
    ) -> SaveDataOperation {
        let rpoint = self.canvas.relative_point(point);

        if self.save_button.contains(ctx.context, rpoint) {
            self.save_action(ctx);
            SaveDataOperation::Saving
        } else if self.delete_button.contains(ctx.context, rpoint) {
            self.delete_action(ctx);
            SaveDataOperation::Deleting
        } else if self.load_button.contains(ctx.context, rpoint) {
            SaveDataOperation::Loading(self.slot_id)
        } else {
            SaveDataOperation::NoOperation
        }
    }
}

impl DrawableComponent for DrawableSaveEntry {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.appearance_frame.draw(ctx)?;
            self.background.draw(ctx)?;

	    self.table_frame.draw(ctx)?;

            if let Some(vtext) = self.date_text.as_mut() {
                vtext.draw(ctx)?;
            }

            if let Some(vtext) = self.money_text.as_mut() {
                vtext.draw(ctx)?;
            }

	    for vtext in self.desc_text.iter_mut() {
		vtext.draw(ctx)?;
	    }

            self.save_button.draw(ctx)?;
            self.load_button.draw(ctx)?;
            self.delete_button.draw(ctx)?;

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
            numeric::Vector2f::new(0.75, 0.75),
            0,
        );

        let mut entries = Vec::new();
        let mut pos_rect = numeric::Rect::new(40.0, 100.0, 288.0, 496.0);

        let texture_vec = vec![
            TextureID::Paper4,
            TextureID::Paper5,
            TextureID::Paper6,
            TextureID::Paper7,
        ];
        for (index, maybe_save_data) in save_data_list.iter().enumerate() {
            entries.push(DrawableSaveEntry::new(
                ctx,
                texture_vec[index],
                maybe_save_data.clone(),
                pos_rect,
                (index + 1) as u8,
            ));

            pos_rect.x += 300.0;
        }

        let mut title_text = UniText::new(
            "鈴奈庵営業記録".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                ctx.resource.get_font(FontID::Cinema),
                numeric::Vector2f::new(36.0, 36.0),
                ggraphics::Color::from_rgba_u32(0xff),
            ),
        );

        title_text.make_center(
            ctx.context,
            numeric::Point2f::new(window_rect.w / 2.0, 60.0),
        );

        let background = UniTexture::new(
            ctx.ref_texture(TextureID::Paper1),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.4, 1.4),
            0.0,
            0,
        );

        SaveEntryTable {
            canvas: SubScreen::new(
                ctx.context,
                window_rect,
                draw_depth,
                ggraphics::Color::from_rgba_u32(0),
            ),
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
    ) -> SaveDataOperation {
        let rpoint = self.canvas.relative_point(point);

        for entry in self.entries.iter_mut() {
            if entry.contains(ctx.context, rpoint) {
                return entry.click_handler(ctx, rpoint);
            }
        }

        SaveDataOperation::NoOperation
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
