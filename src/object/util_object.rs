use ggez::graphics as ggraphics;

use torifune::graphics::*;
use torifune::impl_drawable_object_for_wrapped;
use torifune::numeric;

use crate::core::{GameData, TileBatchTextureID};

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
}

impl TableFrame {
    pub fn new(
        game_data: &GameData,
        position: numeric::Point2f,
        frame_data: FrameData,
        draw_depth: i8,
    ) -> Self {

	let mut tile_batch = game_data.ref_tile_batch(TileBatchTextureID::OldStyleFrame);
	tile_batch.set_position(position);
	
        let mut table_frame = TableFrame {
            tile_batch: tile_batch,
            frame_data: frame_data,
            drwob_essential: DrawableObjectEssential::new(true, draw_depth),
        };

        table_frame.update_tile_batch();

        table_frame
    }

    pub fn update_tile_batch(&mut self) {
        self.tile_batch.clear_batch();

        let width = self.frame_data.width();
        let height = self.frame_data.height();
        let tile_size = self.tile_batch.get_tile_size();

        let mut top_dest_pos = numeric::Point2f::new(tile_size.x as f32, 0.0);
        let mut bottom_dest_pos = numeric::Point2f::new(tile_size.x as f32, height - 16.0);
        for _ in 2..(width / tile_size.x as f32 + 0.5) as usize {
            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(1, 0),
                top_dest_pos,
                numeric::Vector2f::new(1.0, 1.0),
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );

            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(1, 0),
                bottom_dest_pos,
                numeric::Vector2f::new(1.0, 1.0),
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );
            top_dest_pos.x += tile_size.x as f32;
            bottom_dest_pos.x += tile_size.x as f32;

            println!("{}:{}", top_dest_pos.x, top_dest_pos.y);
            println!("{}:{}", bottom_dest_pos.x, bottom_dest_pos.y);
        }

        let mut left_dest_pos = numeric::Point2f::new(0.0, tile_size.y as f32);
        let mut right_dest_pos = numeric::Point2f::new(width - 16.0, tile_size.y as f32);
        for _ in 2..(width / tile_size.x as f32 + 0.5) as usize {
            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(0, 1),
                left_dest_pos,
                numeric::Vector2f::new(1.0, 1.0),
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );

            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(0, 1),
                right_dest_pos,
                numeric::Vector2f::new(1.0, 1.0),
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );
            left_dest_pos.y += tile_size.y as f32;
            right_dest_pos.y += tile_size.y as f32;
        }

        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(0, 0),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(2, 0),
            numeric::Point2f::new(width - tile_size.x as f32, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(0, 2),
            numeric::Point2f::new(0.0, height - tile_size.y as f32),
            numeric::Vector2f::new(1.0, 1.0),
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(2, 2),
            numeric::Point2f::new(width - tile_size.x as f32, height - tile_size.y as f32),
            numeric::Vector2f::new(1.0, 1.0),
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );
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
