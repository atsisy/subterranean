use std::rc::Rc;

use ggez::graphics as ggraphics;

use torifune::graphics::drawable::*;
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::*;
use torifune::numeric;
use torifune::roundup2f;

use torifune::graphics::object::sub_screen::SubScreen;

use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;

use crate::core::*;
use crate::object::util_object::*;
use crate::set_table_frame_cell_center;

use number_to_jk::number_to_jk;

pub struct SuzunaStatusMainPage {
    table_frame: TableFrame,
    desc_text: Vec<VerticalText>,
    reputation_text: VerticalText,
    money_text: VerticalText,
    day_text: VerticalText,
    kosuzu_level_text: VerticalText,
    drwob_essential: DrawableObjectEssential,
}

impl SuzunaStatusMainPage {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>) -> Self {
        let normal_scale_font = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(24.0, 24.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );

        let large_scale_font = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(36.0, 36.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );

        let table_frame = TableFrame::new(
            ctx.resource,
            numeric::Point2f::new(150.0, 30.0),
            TileBatchTextureID::OldStyleFrame,
            FrameData::new(vec![120.0, 220.0], vec![40.0; 3]),
            numeric::Vector2f::new(0.25, 0.25),
            0,
        );

        let mut desc_text = Vec::new();

        for (index, s) in vec!["評判", "習熟度", "所持金"].iter().enumerate() {
            let mut vtext = VerticalText::new(
                s.to_string(),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                normal_scale_font,
            );

            set_table_frame_cell_center!(
                ctx.context,
                table_frame,
                vtext,
                numeric::Vector2u::new(index as u32, 0)
            );

            desc_text.push(vtext);
        }

        let mut reputation_text = VerticalText::new(
            number_to_jk(ctx.savable_data.suzunaan_status.reputation as u64),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            normal_scale_font,
        );

        set_table_frame_cell_center!(
            ctx.context,
            table_frame,
            reputation_text,
            numeric::Vector2u::new(0, 1)
        );

        let mut money_text = VerticalText::new(
            format!(
                "{}円",
                number_to_jk(ctx.savable_data.task_result.total_money as u64)
            ),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            normal_scale_font,
        );

        set_table_frame_cell_center!(
            ctx.context,
            table_frame,
            money_text,
            numeric::Vector2u::new(2, 1)
        );

        let mut kosuzu_level_text = VerticalText::new(
            format!("{}", number_to_jk(0)),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            normal_scale_font,
        );

        set_table_frame_cell_center!(
            ctx.context,
            table_frame,
            kosuzu_level_text,
            numeric::Vector2u::new(1, 1)
        );

        SuzunaStatusMainPage {
            table_frame: table_frame,
            reputation_text: reputation_text,
            desc_text: desc_text,
            day_text: VerticalText::new(
                format!(
                    "{}月{}日",
                    number_to_jk::number_to_jk(ctx.savable_data.date.month as u64),
                    number_to_jk::number_to_jk(ctx.savable_data.date.day as u64),
                ),
                numeric::Point2f::new(600.0, 50.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                large_scale_font,
            ),
            money_text: money_text,
            kosuzu_level_text: kosuzu_level_text,
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }
}

impl DrawableComponent for SuzunaStatusMainPage {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.table_frame.draw(ctx).unwrap();
            self.day_text.draw(ctx).unwrap();

            for vtext in self.desc_text.iter_mut() {
                vtext.draw(ctx).unwrap();
            }

            self.reputation_text.draw(ctx).unwrap();
            self.money_text.draw(ctx).unwrap();

            self.kosuzu_level_text.draw(ctx).unwrap();
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

pub struct SuzunaStatusScreen {
    canvas: SubScreen,
    background: UniTexture,
    main_page: SuzunaStatusMainPage,
    go_left_texture: UniTexture,
    go_right_texture: UniTexture,
}

impl SuzunaStatusScreen {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        rect: numeric::Rect,
        depth: i8,
    ) -> SuzunaStatusScreen {
        let background_texture = UniTexture::new(
            ctx.ref_texture(TextureID::TextBackground),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
        );

        let mut left = UniTexture::new(
            ctx.ref_texture(TextureID::GoNextPageLeft),
            numeric::Point2f::new(0.0, rect.h - 32.0),
            numeric::Vector2f::new(0.5, 0.5),
            0.0,
            0,
        );
        left.hide();

        let right = UniTexture::new(
            ctx.ref_texture(TextureID::GoNextPageRight),
            numeric::Point2f::new(rect.w - 32.0, rect.h - 32.0),
            numeric::Vector2f::new(0.5, 0.5),
            0.0,
            0,
        );

        SuzunaStatusScreen {
            canvas: SubScreen::new(
                ctx.context,
                rect,
                depth,
                ggraphics::Color::from_rgba_u32(0xffffffff),
            ),
            background: background_texture,
            main_page: SuzunaStatusMainPage::new(ctx),
            go_left_texture: left,
            go_right_texture: right,
        }
    }

    pub fn click_handler<'a>(&mut self, _ctx: &mut SuzuContext<'a>, click_point: numeric::Point2f) {
        if !self.canvas.contains(click_point) {
            return;
        }
    }
}

impl DrawableComponent for SuzunaStatusScreen {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.background.draw(ctx)?;
            self.main_page.draw(ctx)?;

            self.go_right_texture.draw(ctx)?;
            self.go_left_texture.draw(ctx)?;

            sub_screen::pop_screen(ctx);
            self.canvas.draw(ctx).unwrap();
        }

        Ok(())
    }

    #[inline(always)]
    fn hide(&mut self) {
        self.canvas.hide()
    }

    #[inline(always)]
    fn appear(&mut self) {
        self.canvas.appear()
    }

    #[inline(always)]
    fn is_visible(&self) -> bool {
        self.canvas.is_visible()
    }

    #[inline(always)]
    fn set_drawing_depth(&mut self, depth: i8) {
        self.canvas.set_drawing_depth(depth)
    }

    #[inline(always)]
    fn get_drawing_depth(&self) -> i8 {
        self.canvas.get_drawing_depth()
    }
}

impl DrawableObject for SuzunaStatusScreen {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for SuzunaStatusScreen {
    impl_texture_object_for_wrapped! {canvas}
}
