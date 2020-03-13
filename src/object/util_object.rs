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

    fn real_height(&self) -> f32 {
	let tile_size = self.tile_batch.get_tile_size();
	self.frame_data.height() + (((self.frame_data.each_cols_size.len() + 1) as u32 * tile_size.y) as f32)
    }

    fn real_width(&self) -> f32 {
	let tile_size = self.tile_batch.get_tile_size();
	self.frame_data.width() + (((self.frame_data.each_rows_size.len() + 1) as u32 * tile_size.x) as f32)
    }
    
    pub fn get_grid_position(&self, point: numeric::Point2f) -> numeric::Vector2u {
	let frame_position = self.get_position();
	let rpoint = numeric::Point2f::new(point.x - frame_position.x, point.y - frame_position.y);
	let mut remain = rpoint;
	let mut grid_position = numeric::Vector2u::new(0, 0);
	let tile_size = self.tile_batch.get_tile_size();

	for size in &self.frame_data.each_rows_size {
	    remain.x -= size + (tile_size.x as f32 * 1.5);
	    if remain.x < 0.0 {
		break;
	    }
	    grid_position.x += 1;
	}

	for size in &self.frame_data.each_cols_size {
	    remain.y -= size + (tile_size.y as f32 * 1.5);
	    if remain.y < 0.0 {
		break;
	    }
	    grid_position.y += 1;
	}

	grid_position
    }

    pub fn get_grid_topleft(&self, grid_position: numeric::Vector2u, offset: numeric::Vector2f) -> numeric::Point2f {
	let mut remain_grid_position = grid_position;
	let mut top_left = numeric::Point2f::new(0.0, 0.0);
	let tile_size = self.tile_batch.get_tile_size();

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

    fn stroke_vline_batch(&mut self, begin: numeric::Point2f) {
	let tile_size = self.tile_batch.get_tile_size();
	let height = self.real_height();

	let mut position = begin;
	for _ in 2..(height / tile_size.y as f32 + 0.5) as usize {
	    position.y += tile_size.y as f32;
            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(3, 0),
                position,
                numeric::Vector2f::new(1.0, 1.0),
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );
        }

	self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(3, 1),
	    begin,
            numeric::Vector2f::new(1.0, 1.0),
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

	self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(3, 2),
	    numeric::Point2f::new(begin.x, begin.y + height - tile_size.y as f32),
            numeric::Vector2f::new(1.0, 1.0),
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );
    }

    fn stroke_hline_batch(&mut self, begin: numeric::Point2f) {
	let tile_size = self.tile_batch.get_tile_size();
	let width = self.real_width();

	let mut position = begin;
	for _ in 2..(width / tile_size.y as f32 + 0.5) as usize {
	    position.x += tile_size.x as f32;
            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(4, 0),
                position,
                numeric::Vector2f::new(1.0, 1.0),
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );
        }

	self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(4, 1),
	    begin,
            numeric::Vector2f::new(1.0, 1.0),
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

	self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(4, 2),
	    numeric::Point2f::new(begin.x + width - tile_size.y as f32, begin.y),
            numeric::Vector2f::new(1.0, 1.0),
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );
    }

    pub fn update_tile_batch(&mut self) {
        self.tile_batch.clear_batch();

        let tile_size = self.tile_batch.get_tile_size();
	let width = self.real_width();
	let height = self.real_height();

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
                numeric::Vector2u::new(1, 2),
                bottom_dest_pos,
                numeric::Vector2f::new(1.0, 1.0),
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );
            top_dest_pos.x += tile_size.x as f32;
            bottom_dest_pos.x += tile_size.x as f32;
        }
	
        let mut left_dest_pos = numeric::Point2f::new(0.0, tile_size.y as f32);
        let mut right_dest_pos = numeric::Point2f::new(width - 16.0, tile_size.y as f32);
        for _ in 2..(height / tile_size.y as f32 + 0.5) as usize {
            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(0, 1),
                left_dest_pos,            numeric::Vector2f::new(1.0, 1.0),
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );

            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(2, 1),
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

	let mut position = numeric::Point2f::new(0.0, 0.0);
	for i in 0..self.frame_data.each_rows_size.len() - 1 {
	    position.x += self.frame_data.each_rows_size.get(i).unwrap() + tile_size.x as f32;
	    self.stroke_vline_batch(position);
	}

	let mut position = numeric::Point2f::new(0.0, 0.0);
	for i in 0..self.frame_data.each_cols_size.len() - 1 {
	    position.y += self.frame_data.each_cols_size.get(i).unwrap() + tile_size.y as f32;
	    self.stroke_hline_batch(position);
	}

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
