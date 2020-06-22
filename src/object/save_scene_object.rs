use std::rc::Rc;

use ggez::graphics as ggraphics;

use sub_screen::SubScreen;
use torifune::graphics::drawable::*;
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::*;
use torifune::numeric;

use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;

use crate::core::*;
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
    date_text: Option<VerticalText>,
    money_text: Option<VerticalText>,
    none_text: Option<VerticalText>,
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
            ctx.resource.ref_texture(texture_id),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
        );

        let save_button = SelectButton::new(
            ctx,
            numeric::Rect::new(30.0, pos_rect.h - 80.0, 60.0, 60.0),
            Box::new(UniTexture::new(
                ctx.resource.ref_texture(TextureID::ChoicePanel1),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
            )),
        );

        let delete_button = SelectButton::new(
            ctx,
            numeric::Rect::new(110.0, pos_rect.h - 80.0, 60.0, 60.0),
            Box::new(UniTexture::new(
                ctx.resource.ref_texture(TextureID::ChoicePanel2),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
            )),
        );

        let load_button = SelectButton::new(
            ctx,
            numeric::Rect::new(190.0, pos_rect.h - 80.0, 60.0, 60.0),
            Box::new(UniTexture::new(
                ctx.resource.ref_texture(TextureID::ChoicePanel3),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
            )),
        );

	// 影を表示する領域を確保するために
	// 背景テクスチャを微妙に切り出し
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

        if let Some(savable_data) = savable_data {
            Self::new_some(
                ctx,
                background,
                appr_frame,
                savable_data,
                pos_rect,
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
        save_button: SelectButton,
        load_button: SelectButton,
        delete_button: SelectButton,
        slot_id: u8,
    ) -> Self {
        let mut entry = DrawableSaveEntry {
            background: background,
            date_text: None,
            money_text: None,
            none_text: None,
            save_button: save_button,
            load_button: load_button,
            delete_button: delete_button,
            appearance_frame: appr_frame,
            canvas: SubScreen::new(ctx.context, pos_rect, 0, ggraphics::Color::from_rgba_u32(0)),
            slot_id: slot_id,
        };

        entry.update_entry_contents(ctx, savable_data);
        entry
    }

    fn new_none<'a>(
        ctx: &mut SuzuContext<'a>,
        background: UniTexture,
        appr_frame: TileBatchFrame,
        pos_rect: numeric::Rect,
        save_button: SelectButton,
        load_button: SelectButton,
        delete_button: SelectButton,
        slot_id: u8,
    ) -> Self {
        let mut entry = DrawableSaveEntry {
            background: background,
            date_text: None,
            money_text: None,
            none_text: None,
            save_button: save_button,
            load_button: load_button,
            delete_button: delete_button,
            appearance_frame: appr_frame,
            canvas: SubScreen::new(ctx.context, pos_rect, 0, ggraphics::Color::from_rgba_u32(0)),
            slot_id: slot_id,
        };

        entry.update_none_contents(ctx);
        entry
    }

    fn update_entry_contents<'a>(&mut self, ctx: &mut SuzuContext<'a>, savable_data: SavableData) {
        let date_text = VerticalText::new(
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
                ctx.resource.get_font(FontID::JpFude1),
                numeric::Vector2f::new(40.0, 40.0),
                ggraphics::Color::from_rgba_u32(0xff),
            ),
        );

        let money_text = VerticalText::new(
            format!(
                "所持金{}円",
                number_to_jk(savable_data.task_result.total_money as u64)
            ),
            numeric::Point2f::new(150.0, 160.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                ctx.resource.get_font(FontID::JpFude1),
                numeric::Vector2f::new(35.0, 35.0),
                ggraphics::Color::from_rgba_u32(0xff),
            ),
        );

        self.date_text = Some(date_text);
        self.money_text = Some(money_text);
        self.none_text = None;
    }

    fn update_none_contents<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        let mut none_text = VerticalText::new(
            "記録ガ在リマセン".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                ctx.resource.get_font(FontID::JpFude1),
                numeric::Vector2f::new(40.0, 40.0),
                ggraphics::Color::from_rgba_u32(0xff),
            ),
        );

        let size = self.canvas.get_drawing_size(ctx.context);
        none_text.make_center(
            ctx.context,
            numeric::Point2f::new(size.x / 2.0, size.y / 2.0),
        );

        self.date_text = None;
        self.money_text = None;
        self.none_text = Some(none_text);
    }

    fn save_action<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        ctx.savable_data.save(self.slot_id).unwrap();
        let savable_data = ctx.savable_data.clone();
        self.update_entry_contents(ctx, savable_data);
    }

    fn delete_action<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        SavableData::delete(self.slot_id);
        self.update_none_contents(ctx);
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

            if let Some(vtext) = self.date_text.as_mut() {
                vtext.draw(ctx)?;
            }

            if let Some(vtext) = self.money_text.as_mut() {
                vtext.draw(ctx)?;
            }

            if let Some(vtext) = self.none_text.as_mut() {
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
        let mut pos_rect = numeric::Rect::new(50.0, 125.0, 288.0, 496.0);

        let texture_vec = vec![
            TextureID::TextBackground,
            TextureID::TextBackground,
            TextureID::TextBackground,
            TextureID::TextBackground,
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
                numeric::Vector2f::new(40.0, 40.0),
                ggraphics::Color::from_rgba_u32(0xff),
            ),
        );

        title_text.make_center(
            ctx.context,
            numeric::Point2f::new(window_rect.w / 2.0, 70.0),
        );

        let background = UniTexture::new(
            ctx.resource.ref_texture(TextureID::Paper1),
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
