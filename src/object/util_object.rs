use std::collections::HashMap;
use std::rc::Rc;

use ggez::graphics as ggraphics;

use sub_screen::SubScreen;
use torifune::core::Clock;
use torifune::graphics::drawable::*;
use torifune::graphics::object::shape::MeshShape;
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::tile_batch::*;
use torifune::graphics::object::*;
use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;
use torifune::numeric;

use crate::core::*;

extern crate mint;

pub struct FrameData {
    each_cols_size: Vec<f32>,
    each_rows_size: Vec<f32>,
}

impl FrameData {
    pub fn new(each_cols_size: Vec<f32>, each_rows_size: Vec<f32>) -> Self {
        FrameData {
            each_cols_size: each_cols_size,
            each_rows_size: each_rows_size,
        }
    }

    pub fn width(&self) -> f32 {
        self.each_rows_size.iter().fold(0.0, |sum, size| sum + size)
    }

    pub fn height(&self) -> f32 {
        self.each_cols_size.iter().fold(0.0, |sum, size| sum + size)
    }

    pub fn get_row_size_at(&self, index: usize) -> f32 {
        *self.each_rows_size.get(index).unwrap()
    }

    pub fn get_col_size_at(&self, index: usize) -> f32 {
        *self.each_cols_size.get(index).unwrap()
    }
}

pub struct TableFrame {
    tile_batch: TileBatch,
    frame_data: FrameData,
    drwob_essential: DrawableObjectEssential,
    frame_scale: numeric::Vector2f,
}

impl TableFrame {
    pub fn new(
        game_data: &GameResource,
        position: numeric::Point2f,
        frame_batch_texture: TileBatchTextureID,
        frame_data: FrameData,
        frame_scale: numeric::Vector2f,
        draw_depth: i8,
    ) -> Self {
        let mut tile_batch = game_data.ref_tile_batch(frame_batch_texture);
        tile_batch.set_position(position);

        let mut table_frame = TableFrame {
            tile_batch: tile_batch,
            frame_data: frame_data,
            drwob_essential: DrawableObjectEssential::new(true, draw_depth),
            frame_scale: frame_scale,
        };

        table_frame.update_tile_batch();

        table_frame
    }

    fn contains(&self, point: numeric::Point2f) -> bool {
        let current_position = self.get_position();
        point.x >= current_position.x
            && point.y >= current_position.y
            && point.x <= (current_position.x + self.real_width())
            && point.y <= (current_position.y + self.real_height())
    }

    fn contains_at(&self, grid_position: numeric::Vector2u, point: numeric::Point2f) -> bool {
        let self_position = self.get_position();
        let grid_lefttop = self.get_grid_topleft(
            grid_position,
            numeric::Vector2f::new(self_position.x, self_position.y),
        );
        let height = self
            .frame_data
            .each_cols_size
            .get(grid_position.y as usize)
            .unwrap();
        let width = self
            .frame_data
            .each_rows_size
            .get(grid_position.x as usize)
            .unwrap();
        let click_area = numeric::Rect::new(grid_lefttop.x, grid_lefttop.y, *width, *height);
        click_area.contains(point)
    }

    fn get_scaled_tile_size(&self) -> numeric::Vector2f {
        let tile_size = self.tile_batch.get_tile_size();
        numeric::Vector2f::new(
            tile_size.x as f32 * self.frame_scale.x,
            tile_size.y as f32 * self.frame_scale.y,
        )
    }

    pub fn get_center_of(
        &self,
        grid_pos: numeric::Vector2u,
        offset: numeric::Point2f,
    ) -> numeric::Point2f {
        let left_top = self.get_grid_topleft(grid_pos, numeric::Vector2f::new(offset.x, offset.y));
        numeric::Point2f::new(
            left_top.x + (self.frame_data.get_row_size_at(grid_pos.x as usize) / 2.0),
            left_top.y + (self.frame_data.get_col_size_at(grid_pos.y as usize) / 2.0),
        )
    }

    ///
    /// 線の幅を含めてTableFrameの高さを返す
    ///
    pub fn real_height(&self) -> f32 {
        let tile_size = self.get_scaled_tile_size();
        self.frame_data.height() + (self.frame_data.each_cols_size.len() as f32 * tile_size.y)
    }

    ///
    /// 線の幅を含めてTableFrameの幅を返す
    ///
    pub fn real_width(&self) -> f32 {
        let tile_size = self.get_scaled_tile_size();
        self.frame_data.width() + (self.frame_data.each_rows_size.len() as f32 * tile_size.x)
    }

    pub fn get_cols(&self) -> usize {
        self.frame_data.each_cols_size.len()
    }

    pub fn get_rows(&self) -> usize {
        self.frame_data.each_rows_size.len()
    }

    fn tile_per_vline(&self, length: f32) -> usize {
        let tile_size = self.get_scaled_tile_size();
        (length / tile_size.y) as usize
    }

    fn tile_per_hline(&self, length: f32) -> usize {
        let tile_size = self.get_scaled_tile_size();
        (length / tile_size.x) as usize
    }

    fn tile_remaining_vline(&self, length: f32) -> f32 {
        let tile_size = self.get_scaled_tile_size();
        let num = self.tile_per_vline(length) as f32;
        length - (num * tile_size.y)
    }

    fn tile_remaining_hline(&self, length: f32) -> f32 {
        let tile_size = self.get_scaled_tile_size();
        let num = self.tile_per_hline(length) as f32;
        length - (num * tile_size.x)
    }

    pub fn size(&self) -> numeric::Vector2f {
        numeric::Vector2f::new(self.real_width(), self.real_height())
    }

    ///
    /// あるPointが含まれているグリッドの位置を返す
    ///
    pub fn get_grid_position(
        &self,
        _: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> Option<numeric::Vector2u> {
        if !self.contains(point) {
            return None;
        }

        let frame_position = self.get_position();
        let rpoint = numeric::Point2f::new(point.x - frame_position.x, point.y - frame_position.y);
        let mut remain = rpoint;
        let mut grid_position = numeric::Vector2u::new(0, 0);
        let tile_size = self.get_scaled_tile_size();

        for size in &self.frame_data.each_rows_size {
            remain.x -= size + tile_size.x as f32;
            if remain.x < 0.0 {
                break;
            }
            grid_position.x += 1;
        }

        for size in &self.frame_data.each_cols_size {
            remain.y -= size + tile_size.y as f32;
            if remain.y < 0.0 {
                break;
            }
            grid_position.y += 1;
        }

        if self.frame_data.each_rows_size.len() > grid_position.x as usize
            && self.frame_data.each_cols_size.len() > grid_position.y as usize
        {
            Some(grid_position)
        } else {
            None
        }
    }

    ///
    /// あるPointがどの枠に含まれていて、その枠はどこを始点として描画されているかを返す
    ///
    pub fn get_grid_topleft(
        &self,
        grid_position: numeric::Vector2u,
        offset: numeric::Vector2f,
    ) -> numeric::Point2f {
        let mut remain_grid_position = grid_position;
        let mut top_left = numeric::Point2f::new(0.0, 0.0);
        let tile_size = self.get_scaled_tile_size();

        for size in &self.frame_data.each_rows_size {
            top_left.x += tile_size.x as f32;
            if remain_grid_position.x == 0 {
                break;
            }
            top_left.x += size;
            remain_grid_position.x -= 1;
        }

        for size in &self.frame_data.each_cols_size {
            top_left.y += tile_size.y as f32;
            if remain_grid_position.y == 0 {
                break;
            }
            top_left.y += size;
            remain_grid_position.y -= 1;
        }

        top_left + offset
    }

    ///
    /// 垂直方向の線を引くメソッド
    ///
    fn stroke_vline_batch(&mut self, begin: numeric::Point2f) {
        let tile_size = self.get_scaled_tile_size();
        let height = self.real_height();

        let begin = numeric::Point2f::new(begin.x.round(), begin.y.round());
        let mut position = begin;
        println!("{}", self.tile_per_vline(height));

        for _ in 1..self.tile_per_vline(height) {
            position.y += tile_size.y;
            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(3, 0),
                position,
                self.frame_scale,
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );
        }

        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(3, 1),
            begin,
            self.frame_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(3, 2),
            numeric::Point2f::new(begin.x, begin.y + height - tile_size.y),
            self.frame_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );
    }

    ///
    /// 水平方向の線を引くメソッド
    ///
    fn stroke_hline_batch(&mut self, begin: numeric::Point2f) {
        let tile_size = self.get_scaled_tile_size();
        let width = self.real_width();

        let begin = numeric::Point2f::new(begin.x, begin.y);
        let mut position = begin;
        for _ in 1..self.tile_per_hline(width) {
            position.x += tile_size.x;
            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(4, 0),
                position,
                self.frame_scale,
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );
        }

        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(4, 1),
            begin,
            self.frame_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(4, 2),
            numeric::Point2f::new(begin.x + width - tile_size.y, begin.y),
            self.frame_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );
    }

    ///
    /// Tile Batchの情報を更新する
    ///
    pub fn update_tile_batch(&mut self) {
        self.tile_batch.clear_batch();

        let tile_size = self.get_scaled_tile_size();
        let width = self.real_width();
        let height = self.real_height();

        //
        // 水平方向の枠だけ描画
        //
        let mut top_dest_pos = numeric::Point2f::new(tile_size.x, 0.0);
        let mut bottom_dest_pos =
            numeric::Point2f::new(tile_size.x, (height - tile_size.x).round());
        for _ in 1..(self.tile_per_hline(width) - 1) {
            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(1, 0),
                top_dest_pos,
                self.frame_scale,
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );

            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(1, 2),
                bottom_dest_pos,
                self.frame_scale,
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );
            top_dest_pos.x += tile_size.x;
            bottom_dest_pos.x += tile_size.x;
        }
        let last_scale = numeric::Vector2f::new(
            self.tile_remaining_hline(width) / (self.tile_batch.get_tile_size().x as f32),
            self.frame_scale.y,
        );
        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(1, 0),
            top_dest_pos,
            last_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );
        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(1, 2),
            bottom_dest_pos,
            last_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

        //
        // 垂直方向の枠だけ描画
        //
        let mut left_dest_pos = numeric::Point2f::new(0.0, tile_size.y);
        let mut right_dest_pos = numeric::Point2f::new(width - tile_size.x, tile_size.y);
        for _ in 1..(self.tile_per_vline(height) - 1) {
            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(0, 1),
                left_dest_pos,
                self.frame_scale,
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );

            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(2, 1),
                right_dest_pos,
                self.frame_scale,
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );
            left_dest_pos.y += tile_size.y;
            right_dest_pos.y += tile_size.y;
        }
        let last_scale = numeric::Vector2f::new(
            self.frame_scale.x,
            self.tile_remaining_vline(height) / (self.tile_batch.get_tile_size().y as f32),
        );
        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(0, 1),
            left_dest_pos,
            last_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );
        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(2, 1),
            right_dest_pos,
            last_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

        //
        // 枠の角を描画
        //
        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(0, 0),
            numeric::Point2f::new(0.0, 0.0),
            self.frame_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(2, 0),
            numeric::Point2f::new(width - tile_size.x, 0.0),
            self.frame_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(0, 2),
            numeric::Point2f::new(0.0, height - tile_size.y),
            self.frame_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(2, 2),
            numeric::Point2f::new(width - tile_size.x as f32, height - tile_size.y as f32),
            self.frame_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

        // 描画するものがなければ、垂直の線は描画しない
        if self.frame_data.each_rows_size.is_empty() {
            return ();
        }

        //
        // 中身のグリッドを描画
        //
        let mut position = numeric::Point2f::new(0.0, 0.0);
        for i in 0..self.frame_data.each_rows_size.len() - 1 {
            position.x += self.frame_data.each_rows_size.get(i).unwrap() + tile_size.x;
            self.stroke_vline_batch(position);
        }

        let mut position = numeric::Point2f::new(0.0, 0.0);
        for i in 0..self.frame_data.each_cols_size.len() - 1 {
            position.y += self.frame_data.each_cols_size.get(i).unwrap() + tile_size.y;
            self.stroke_hline_batch(position);
        }
    }

    pub fn make_center(&mut self, point: numeric::Point2f) {
        let half_size = numeric::Vector2f::new(self.real_width() / 2.0, self.real_height() / 2.0);
        let next_position = numeric::Point2f::new(point.x - half_size.x, point.y - half_size.y);
        self.set_position(next_position);
    }
}

impl DrawableComponent for TableFrame {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.tile_batch.draw(ctx).unwrap()
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

impl DrawableObject for TableFrame {
    impl_drawable_object_for_wrapped! {tile_batch}
}

#[macro_export]
macro_rules! set_table_frame_cell_center {
    ($ctx: expr, $table_frame: expr, $obj: expr, $point: expr) => {
        $obj.make_center(
            $ctx,
            roundup2f!($table_frame.get_center_of($point, $table_frame.get_position())),
        );
    };
}

///
/// # ボタンみたいなものを表示する構造体
///
pub struct SelectButton {
    canvas: SubScreen,
    button_texture: Box<dyn TextureObject>,
    button_toggle: bool,
}

impl SelectButton {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        button_rect: numeric::Rect,
        mut texture: Box<dyn TextureObject>,
    ) -> Self {
        texture.set_position(numeric::Point2f::new(0.0, 0.0));
        texture.fit_scale(
            ctx.context,
            numeric::Vector2f::new(button_rect.w, button_rect.h),
        );

        SelectButton {
            canvas: SubScreen::new(
                ctx.context,
                button_rect,
                0,
                ggraphics::Color::from_rgba_u32(0),
            ),
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

pub struct TextButtonTexture {
    text: UniText,
    background: ggraphics::Mesh,
    drwob_essential: DrawableObjectEssential,
    button_pos: numeric::Rect,
}

impl TextButtonTexture {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        pos: numeric::Point2f,
        text_str: String,
        font_info: FontInformation,
        padding: f32,
        bg_color: ggraphics::Color,
        depth: i8,
    ) -> Self {
        let text_pos = numeric::Point2f::new(pos.x + padding, pos.y + padding);
        let text = UniText::new(
            text_str,
            text_pos,
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            depth,
            font_info,
        );

        let text_size = text.get_drawing_size(ctx.context);

        let background_size =
            numeric::Vector2f::new(text_size.x + (2.0 * padding), text_size.y + (2.0 * padding));

        let background_shape = shape::Rectangle::new(
            numeric::Rect::new(0.0, 0.0, background_size.x, background_size.y),
            ggraphics::DrawMode::fill(),
            bg_color,
        );

        let mut builder = ggraphics::MeshBuilder::new();
        background_shape.add_to_builder(&mut builder);

        TextButtonTexture {
            text: text,
            background: builder.build(ctx.context).unwrap(),
            drwob_essential: DrawableObjectEssential::new(true, depth),
            button_pos: numeric::Rect::new(pos.x, pos.y, background_size.x, background_size.y),
        }
    }

    fn get_padding(&self) -> f32 {
        self.text.get_position().x - self.button_pos.x
    }
}

impl DrawableComponent for TextButtonTexture {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            ggraphics::draw(
                ctx,
                &self.background,
                ggraphics::DrawParam {
                    dest: numeric::Point2f::new(self.button_pos.x, self.button_pos.y).into(),
                    ..Default::default()
                },
            )?;
            self.text.draw(ctx)?;
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

impl DrawableObject for TextButtonTexture {
    fn set_position(&mut self, pos: numeric::Point2f) {
        let diff = pos - self.get_position();
        self.move_diff(diff);
    }

    fn get_position(&self) -> numeric::Point2f {
        self.button_pos.point().into()
    }

    fn move_diff(&mut self, offset: numeric::Vector2f) {
        self.button_pos.x += offset.x;
        self.button_pos.y += offset.y;
        self.text.move_diff(offset);
    }
}

impl TextureObject for TextButtonTexture {
    fn set_scale(&mut self, _scale: numeric::Vector2f) {}

    fn get_scale(&self) -> numeric::Vector2f {
        numeric::Vector2f::new(1.0, 1.0)
    }

    fn set_rotation(&mut self, _rad: f32) {}

    fn get_rotation(&self) -> f32 {
        0.0
    }

    fn set_crop(&mut self, _crop: ggraphics::Rect) {}

    fn get_crop(&self) -> ggraphics::Rect {
        numeric::Rect::new(0.0, 0.0, 1.0, 1.0)
    }

    fn set_drawing_color(&mut self, _color: ggraphics::Color) {}

    fn get_drawing_color(&self) -> ggraphics::Color {
        ggraphics::WHITE
    }

    fn set_alpha(&mut self, _alpha: f32) {}

    fn get_alpha(&self) -> f32 {
        1.0
    }

    fn set_transform_offset(&mut self, _offset: numeric::Point2f) {}

    fn get_transform_offset(&self) -> numeric::Point2f {
        numeric::Point2f::new(0.0, 0.0)
    }

    fn get_texture_size(&self, _ctx: &mut ggez::Context) -> numeric::Vector2f {
        numeric::Vector2f::new(self.button_pos.x, self.button_pos.y)
    }

    fn replace_texture(&mut self, _texture: Rc<ggraphics::Image>) {}

    fn set_color(&mut self, _color: ggraphics::Color) {}

    fn get_color(&mut self) -> ggraphics::Color {
        ggraphics::WHITE
    }
}

pub struct TileBatchFrame {
    tile_batch: TileBatch,
    tile_batch_texture_id: TileBatchTextureID,
    rect: numeric::Rect,
    drwob_essential: DrawableObjectEssential,
    frame_scale: numeric::Vector2f,
}

impl TileBatchFrame {
    pub fn new(
        game_data: &GameResource,
        tile_batch_texture: TileBatchTextureID,
        rect_pos: numeric::Rect,
        frame_scale: numeric::Vector2f,
        draw_depth: i8,
    ) -> Self {
        let mut tile_batch = game_data.ref_tile_batch(tile_batch_texture);
        tile_batch.set_position(numeric::Point2f::new(rect_pos.x, rect_pos.y));

        let mut frame = TileBatchFrame {
            tile_batch: tile_batch,
            tile_batch_texture_id: tile_batch_texture,
            rect: rect_pos,
            drwob_essential: DrawableObjectEssential::new(true, draw_depth),
            frame_scale: frame_scale,
        };

        frame.update_tile_batch();

        frame
    }

    pub fn get_frame_texture_id(&self) -> TileBatchTextureID {
        self.tile_batch_texture_id
    }

    ///
    /// Tile Batchの情報を更新する
    ///
    pub fn update_tile_batch(&mut self) {
        self.tile_batch.clear_batch();

        let tile_size = self.get_scaled_tile_size();
        let frame_size = self.frame_size();
        let width = frame_size.x;
        let height = frame_size.y;

        //
        // 水平方向の枠だけ描画
        //
        let mut top_dest_pos = numeric::Point2f::new(tile_size.x, 0.0);
        let mut bottom_dest_pos =
            numeric::Point2f::new(tile_size.x, (height - tile_size.x).round());
        for _ in 1..(self.tile_per_hline(width) - 1) {
            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(1, 0),
                top_dest_pos,
                self.frame_scale,
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );

            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(1, 2),
                bottom_dest_pos,
                self.frame_scale,
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );
            top_dest_pos.x += tile_size.x;
            bottom_dest_pos.x += tile_size.x;
        }
        let last_scale = numeric::Vector2f::new(
            self.tile_remaining_hline(width) / (self.tile_batch.get_tile_size().x as f32),
            self.frame_scale.y,
        );
        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(1, 0),
            top_dest_pos,
            last_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );
        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(1, 2),
            bottom_dest_pos,
            last_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

        //
        // 垂直方向の枠だけ描画
        //
        let mut left_dest_pos = numeric::Point2f::new(0.0, tile_size.y);
        let mut right_dest_pos = numeric::Point2f::new(width - tile_size.x, tile_size.y);
        for _ in 1..(self.tile_per_vline(height) - 1) {
            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(0, 1),
                left_dest_pos,
                self.frame_scale,
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );

            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(2, 1),
                right_dest_pos,
                self.frame_scale,
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );
            left_dest_pos.y += tile_size.y;
            right_dest_pos.y += tile_size.y;
        }
        let last_scale = numeric::Vector2f::new(
            self.frame_scale.x,
            self.tile_remaining_vline(height) / (self.tile_batch.get_tile_size().y as f32),
        );
        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(0, 1),
            left_dest_pos,
            last_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );
        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(2, 1),
            right_dest_pos,
            last_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

        //
        // 枠の角を描画
        //
        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(0, 0),
            numeric::Point2f::new(0.0, 0.0),
            self.frame_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(2, 0),
            numeric::Point2f::new(width - tile_size.x, 0.0),
            self.frame_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(0, 2),
            numeric::Point2f::new(0.0, height - tile_size.y),
            self.frame_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(2, 2),
            numeric::Point2f::new(width - tile_size.x as f32, height - tile_size.y as f32),
            self.frame_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );
    }

    fn tile_per_vline(&self, length: f32) -> usize {
        let tile_size = self.get_scaled_tile_size();
        (length / tile_size.y) as usize
    }

    fn tile_remaining_vline(&self, length: f32) -> f32 {
        let tile_size = self.get_scaled_tile_size();
        let num = self.tile_per_vline(length) as f32;
        length - (num * tile_size.y)
    }

    fn tile_per_hline(&self, length: f32) -> usize {
        let tile_size = self.get_scaled_tile_size();
        (length / tile_size.x) as usize
    }

    fn tile_remaining_hline(&self, length: f32) -> f32 {
        let tile_size = self.get_scaled_tile_size();
        let num = self.tile_per_hline(length) as f32;
        length - (num * tile_size.x)
    }

    fn get_scaled_tile_size(&self) -> numeric::Vector2f {
        let tile_size = self.tile_batch.get_tile_size();
        numeric::Vector2f::new(
            tile_size.x as f32 * self.frame_scale.x,
            tile_size.y as f32 * self.frame_scale.y,
        )
    }

    pub fn frame_size(&self) -> numeric::Vector2f {
        numeric::Vector2f::new(self.rect.w, self.rect.h)
    }
}

impl DrawableComponent for TileBatchFrame {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.tile_batch.draw(ctx).unwrap()
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

pub trait Scrollable: DrawableComponent {
    fn scroll<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        offset: numeric::Vector2f,
    );
}

pub enum ScrollDirection {
    Vertical = 0,
    Horizon,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectDirection {
    Up,
    Down,
    Right,
    Left,
}

pub struct ScrollableWindow<D>
where
    D: Scrollable,
{
    canvas: SubScreen,
    scroll_rate: numeric::Vector2f,
    scroll_direction: ScrollDirection,
    drawable: D,
}

impl<D> ScrollableWindow<D>
where
    D: Scrollable,
{
    pub fn new(
        ctx: &mut ggez::Context,
        rect: numeric::Rect,
        drawable: D,
        depth: i8,
        scroll_rate: numeric::Vector2f,
        direction: ScrollDirection,
    ) -> ScrollableWindow<D> {
        ScrollableWindow::<D> {
            canvas: SubScreen::new(ctx, rect, depth, ggraphics::Color::from_rgba_u32(0)),
            scroll_rate: scroll_rate,
            scroll_direction: direction,
            drawable: drawable,
        }
    }

    pub fn ref_object(&self) -> &D {
        &self.drawable
    }

    pub fn ref_object_mut(&mut self) -> &mut D {
        &mut self.drawable
    }

    pub fn relative_point(&self, point: numeric::Point2f) -> numeric::Point2f {
        self.canvas.relative_point(point)
    }

    pub fn scroll<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        x: f32,
        y: f32,
    ) {
        match self.scroll_direction {
            ScrollDirection::Vertical => self.drawable.scroll(
                ctx,
                point,
                numeric::Vector2f::new(self.scroll_rate.x * x, self.scroll_rate.y * y),
            ),
            ScrollDirection::Horizon => self.drawable.scroll(
                ctx,
                point,
                numeric::Vector2f::new(self.scroll_rate.x * y, self.scroll_rate.y * x),
            ),
        }
    }
}

impl<D> DrawableComponent for ScrollableWindow<D>
where
    D: Scrollable,
{
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

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

impl<D> DrawableObject for ScrollableWindow<D>
where
    D: Scrollable,
{
    impl_drawable_object_for_wrapped! {canvas}
}

impl<D> TextureObject for ScrollableWindow<D>
where
    D: Scrollable,
{
    impl_texture_object_for_wrapped! {canvas}
}

///
/// ある範囲内に速さを収めたい時に使用する構造体
///
pub struct SpeedBorder {
    pub positive_x: f32,
    pub negative_x: f32,
    pub positive_y: f32,
    pub negative_y: f32,
}

impl SpeedBorder {
    ///
    /// あるx方向の速さを範囲内に丸め込む
    ///
    pub fn round_speed_x(&self, speed: f32) -> f32 {
        if speed > self.positive_x {
            self.positive_x
        } else if speed < self.negative_x {
            self.negative_x
        } else {
            speed
        }
    }

    ///
    /// あるy方向の速さを範囲内に丸め込む
    ///
    pub fn round_speed_y(&self, speed: f32) -> f32 {
        if speed > self.positive_y {
            self.positive_y
        } else if speed < self.negative_y {
            self.negative_y
        } else {
            speed
        }
    }
}

pub struct TextureSpeedInfo {
    speed: numeric::Vector2f,
    speed_border: SpeedBorder,
}

impl TextureSpeedInfo {
    pub fn new(speed: numeric::Vector2f, border: SpeedBorder) -> TextureSpeedInfo {
        TextureSpeedInfo {
            speed: speed,
            speed_border: border,
        }
    }

    pub fn add_speed(&mut self, speed: numeric::Vector2f) {
        self.speed += speed;
    }

    pub fn set_speed(&mut self, speed: numeric::Vector2f) {
        self.speed = speed;
    }

    pub fn set_speed_x(&mut self, speed: f32) {
        self.speed.x = self.speed_border.round_speed_x(speed);
    }

    pub fn set_speed_y(&mut self, speed: f32) {
        self.speed.y = self.speed_border.round_speed_y(speed);
    }

    pub fn get_speed(&self) -> numeric::Vector2f {
        self.speed
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum AnimationType {
    OneShot,
    Loop,
    Times(usize, usize),
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum AnimationStatus {
    Playing,
    OneLoopFinish,
}

struct SeqTexture {
    textures: Vec<Rc<ggraphics::Image>>,
    index: usize,
}

impl SeqTexture {
    pub fn new(textures: Vec<Rc<ggraphics::Image>>) -> Self {
        SeqTexture {
            textures: textures,
            index: 0,
        }
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }

    pub fn current_frame(&self) -> Rc<ggraphics::Image> {
        self.textures[self.index % self.textures.len()].clone()
    }

    pub fn next_frame(
        &mut self,
        t: AnimationType,
    ) -> Result<Rc<ggraphics::Image>, AnimationStatus> {
        self.index += 1;

        match t {
            AnimationType::OneShot | AnimationType::Times(_, _) => {
                if self.index == self.textures.len() {
                    return Err(AnimationStatus::OneLoopFinish);
                }
            }
            _ => (),
        }

        return Ok(self.current_frame());
    }
}

pub struct TextureAnimation {
    textures: HashMap<ObjectDirection, SeqTexture>,
    current_mode: ObjectDirection,
    object: SimpleObject,
    animation_type: AnimationType,
    next_mode: ObjectDirection,
    frame_speed: Clock,
}

impl TextureAnimation {
    pub fn new(
        obj: SimpleObject,
        mode_order: Vec<ObjectDirection>,
        textures: Vec<Vec<Rc<ggraphics::Image>>>,
        mode: ObjectDirection,
        frame_speed: Clock,
    ) -> Self {
        let mut texture_table = HashMap::new();

        for (index, texure_vec) in textures.iter().enumerate() {
            texture_table.insert(
                mode_order.get(index).unwrap().clone(),
                SeqTexture::new(texure_vec.to_vec()),
            );
        }

        TextureAnimation {
            textures: texture_table,
            current_mode: mode,
            object: obj,
            animation_type: AnimationType::Loop,
            next_mode: mode,
            frame_speed: frame_speed,
        }
    }

    pub fn get_object(&self) -> &SimpleObject {
        &self.object
    }

    pub fn get_mut_object(&mut self) -> &mut SimpleObject {
        &mut self.object
    }

    pub fn change_mode(
        &mut self,
        mode: ObjectDirection,
        animation_type: AnimationType,
        next_mode: ObjectDirection,
    ) {
        if self.current_mode != mode {
            self.current_mode = mode;
            self.next_mode = next_mode;
            self.animation_type = animation_type;
            self.textures
                .get_mut(&self.current_mode)
                .as_mut()
                .unwrap()
                .reset();
        }
    }

    fn next_frame(&mut self) {
        let current_mode = self.current_mode;
        let current_texture = self.textures.get_mut(&current_mode).unwrap();

        match current_texture.next_frame(self.animation_type) {
            // アニメーションは再生中. 特に操作は行わず、ただテクスチャを切り替える
            Ok(texture) => self.get_mut_object().replace_texture(texture),

            // アニメーションが終点に到達なんらかの処理を施す必要がある
            Err(status) => {
                // アニメーションに関してイベントが発生. イベントの種類ごとに何ら可の処理を施す
                match status {
                    // 一回のループが終了したらしい. これは、AnimationType::{OneShot, Times}で発生する
                    AnimationStatus::OneLoopFinish => {
                        // 現在のアニメーションのタイプごとに処理を行う
                        let t = &self.animation_type;
                        match t {
                            &AnimationType::OneShot => {
                                // OneShotの場合
                                // デフォルトのループに切り替える
                                self.animation_type = AnimationType::Loop;
                                self.current_mode = self.next_mode;
                            }
                            &AnimationType::Times(mut cur, lim) => {
                                // Timesの場合
                                // ループカウンタをインクリメントする
                                cur += 1;

                                // まだループする予定
                                if cur < lim {
                                    // 最初のテクスチャに戻し、アニメーションを再開
                                    current_texture.reset();
                                    let texture = current_texture.current_frame();
                                    self.get_mut_object().replace_texture(texture);
                                } else {
                                    // OneShotの場合と同じく、デフォルトのループに切り替える
                                    self.animation_type = AnimationType::Loop;
                                    self.current_mode = self.next_mode;
                                }
                            }
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    pub fn try_next_frame(&mut self, t: Clock) {
        if t % self.frame_speed == 0 {
            self.next_frame();
        }
    }

    pub fn get_current_mode(&self) -> ObjectDirection {
        self.current_mode.clone()
    }
}

pub struct GraphDrawer {
    canvas: SubScreen,
    graph_area: numeric::Rect,
    data: Vec<numeric::Vector2f>,
    shapes: ggraphics::Mesh,
}

impl GraphDrawer {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        rect: numeric::Rect,
        graph_area: numeric::Rect,
        data: Vec<numeric::Vector2f>,
        point_radius: f32,
        point_color: ggraphics::Color,
        line_width: f32,
        line_color: ggraphics::Color,
        depth: i8,
    ) -> Self {
        let mut builder = ggraphics::MeshBuilder::new();

        let max_data = data.iter().fold(numeric::Vector2f::new(0.0, 0.0), |m, v| {
            numeric::Vector2f::new(m.x.max(v.x), m.y.max(v.y))
        });

        let scaled_points: Vec<numeric::Point2f> = data
            .iter()
            .map(|p| {
                numeric::Point2f::new(
                    graph_area.x + (graph_area.w * (p.x / max_data.x)),
                    graph_area.y + (graph_area.h - (graph_area.h * (p.y / max_data.y))),
                )
            })
            .collect();

        for scaled_point in scaled_points.iter() {
            builder.circle(
                ggraphics::DrawMode::fill(),
                mint::Point2::from_slice(&[scaled_point.x, scaled_point.y]),
                point_radius,
                0.01,
                point_color,
            );
        }

        let mint_p_vec: Vec<mint::Point2<f32>> = scaled_points
            .iter()
            .map(|p| mint::Point2::from_slice(&[p.x, p.y]))
            .collect();
        builder
            .line(mint_p_vec.as_slice(), line_width, line_color)
            .unwrap();

        GraphDrawer {
            canvas: SubScreen::new(
                ctx.context,
                rect,
                depth,
                ggraphics::Color::from_rgba_u32(0xffffffff),
            ),
            graph_area: graph_area,
            data: data,
            shapes: builder.build(ctx.context).unwrap(),
        }
    }
}

impl DrawableComponent for GraphDrawer {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            ggraphics::draw(ctx, &self.shapes, ggraphics::DrawParam::default()).unwrap();

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

impl DrawableObject for GraphDrawer {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for GraphDrawer {
    impl_texture_object_for_wrapped! {canvas}
}

pub enum PauseResult {
    ReleasePause,
    GoToTitle,
}

pub struct PauseScreenSet {
    entries: Vec<VerticalText>,
    cursored_index: Option<usize>,
    drwob_essential: DrawableObjectEssential,
}

impl PauseScreenSet {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, depth: i8) -> Self {
        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(28.0, 28.0),
            ggraphics::BLACK,
        );

        let mut entries_vtext = Vec::new();
        let mut text_pos = numeric::Point2f::new(750.0, 200.0);

        for text in vec!["開始画面へ", "再開"] {
            entries_vtext.push(VerticalText::new(
                text.to_string(),
                text_pos,
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                font_info.clone(),
            ));

            text_pos.x -= 50.0;
        }

        PauseScreenSet {
            entries: entries_vtext,
            drwob_essential: DrawableObjectEssential::new(true, depth),
            cursored_index: None,
        }
    }

    ///
    /// 再描画要求有り
    ///
    fn select_entries_handler<'a>(&mut self, ctx: &mut SuzuContext<'a>, index: usize) {
        if let Some(cursored_index) = self.cursored_index {
            if cursored_index == index {
                return;
            }
        }

        self.entries
            .get_mut(index)
            .unwrap()
            .set_color(ggraphics::Color::from_rgba_u32(0x222222ff));

        self.cursored_index = Some(index);
        ctx.process_utility.redraw();
    }

    fn unselect_entries_handler(&mut self) {
        for vtext in self.entries.iter_mut() {
            vtext.set_color(ggraphics::BLACK);
        }
    }

    pub fn mouse_motion_handler<'a>(&mut self, ctx: &mut SuzuContext<'a>, point: numeric::Point2f) {
        for (index, vtext) in self.entries.iter().enumerate() {
            if vtext.contains(ctx.context, point) {
                self.select_entries_handler(ctx, index);
                return;
            }
        }

        // ここまで到達するのは、すべてのテキストにカーソルが重なっていなかった場合
        if self.cursored_index.is_some() {
            self.unselect_entries_handler();
            self.cursored_index = None;
        }
    }

    pub fn mouse_click_handler<'a>(
        &self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
    ) -> Option<PauseResult> {
        for (index, vtext) in self.entries.iter().enumerate() {
            if vtext.contains(ctx.context, point) {
                return match index {
                    0 => Some(PauseResult::GoToTitle),
                    1 => Some(PauseResult::ReleasePause),
                    _ => panic!("index is out of bounds"),
                };
            }
        }

        None
    }
}

impl DrawableComponent for PauseScreenSet {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            for vtext in self.entries.iter_mut() {
                vtext.draw(ctx)?;
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

pub struct SeekBar {
    rect: numeric::Rect,
    seek_offset: numeric::Vector2f,
    handle: UniTexture,
    seek_edge: ggraphics::Mesh,
    dragging: bool,
    drwob_essential: DrawableObjectEssential,
    min_value: f32,
    max_value: f32,
    current_value: f32,
}

impl SeekBar {
    pub fn new<'a>(
	ctx: &mut SuzuContext<'a>,
	pos_rect: numeric::Rect,
	bar_height: f32,
	max_value: f32,
	min_value: f32,
	init_value: f32,
	depth: i8,
    ) -> Self {
	let mut handle = UniTexture::new(
            ctx.ref_texture(TextureID::ChoicePanel1),
	    numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(0.25, 0.25),
            0.0,
            0,
        );

	handle.set_position(
	    numeric::Point2f::new(
		pos_rect.x + ((pos_rect.w - handle.get_drawing_size(ctx.context).x) * (init_value / max_value)),
		pos_rect.y
	    )
	);

	let handle_area = handle.get_drawing_area(ctx.context);
	
        let seek = shape::Rectangle::new(
            numeric::Rect::new(
                pos_rect.x,
                pos_rect.y + (handle_area.h / 2.0) - (bar_height / 2.0),
                pos_rect.w,
                bar_height,
            ),
            ggraphics::DrawMode::fill(),
            ggraphics::Color::from_rgba_u32(0xffffffff),
        );

        let mut builder = ggraphics::MeshBuilder::new();

        seek.add_to_builder(&mut builder);
	
        SeekBar {
            rect: pos_rect,
            seek_offset: numeric::Vector2f::new(0.0, 0.0),
            handle: handle,
            seek_edge: builder.build(ctx.context).unwrap(),
            drwob_essential: DrawableObjectEssential::new(true, depth),
            dragging: false,
	    current_value: init_value,
	    min_value: min_value,
	    max_value: max_value,
        }
    }

    fn update_current_value<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
	let handle_position = self.handle.get_drawing_area(ctx.context);

	let ratio = (handle_position.left() - self.rect.left()) / (self.rect.w - handle_position.w);
	self.current_value = (ratio * (self.max_value - self.min_value)) + self.min_value;
    }

    pub fn start_dragging_check<'a>(&mut self, ctx: &mut SuzuContext<'a>, point: numeric::Point2f) {
        if !self.handle.contains(ctx.context, point) {
            return;
        }

        self.dragging = true;
        let handle_pos = self.handle.get_position();
        self.seek_offset = numeric::Vector2f::new(point.x - handle_pos.x, point.y - handle_pos.y);
    }

    pub fn dragging_handler<'a>(&mut self, ctx: &mut SuzuContext<'a>, point: numeric::Point2f) {
        if !self.dragging {
            return;
        }

        let original = self.handle.get_drawing_area(ctx.context);
        let mut seek_position = numeric::Point2f::new(point.x - self.seek_offset.x, original.y);

        if seek_position.x <= self.rect.x {
            seek_position.x = self.rect.x;
        } else if seek_position.x >= self.rect.right() - original.w {
            seek_position.x = self.rect.right() - original.w;
        }

        ctx.process_utility.redraw();
        self.handle.set_position(seek_position);
	self.update_current_value(ctx);
    }

    pub fn release_handler(&mut self) {
        if !self.dragging {
            return;
        }

        self.dragging = false;
    }

    pub fn get_current_value(&self) -> f32 {
	self.current_value
    }

    pub fn set_value<'a>(&mut self, ctx: &mut SuzuContext<'a>, value: f32) {
	self.handle.set_position(
	    numeric::Point2f::new(
		self.rect.x + ((self.rect.w - self.handle.get_drawing_size(ctx.context).x) * (value / self.max_value)),
		self.rect.y
	    )
	);

	self.current_value = value;
    }
}

impl DrawableComponent for SeekBar {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            ggraphics::draw(
                ctx,
                &self.seek_edge,
                ggraphics::DrawParam::default(),
            )?;
            self.handle.draw(ctx)?;
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
