use std::fs::File;
use std::path::Path;
use std::io::BufReader;

use std::collections::HashMap;

use tiled;
use torifune::numeric;
use ggez::graphics as ggraphics;

use torifune::graphics as tg;
use torifune::core::*;

pub struct TileSet {
    source_name: String,
    tile_size: numeric::Vector2u,
    tile_size_ratio: numeric::Vector2f,
    tile_count: numeric::Vector2u,
    drwob_essential: tg::DrawableObjectEssential,
    draw_param: ggraphics::DrawParam,
    first_gid: u32,
}

impl TileSet {
    fn new(ctx: &mut ggez::Context, tileset: &tiled::Tileset) -> (TileSet, ggraphics::Image) {

        let tiled_image = tileset.images.get(0).unwrap();
        
        let tile_size = numeric::Vector2u::new(tileset.tile_width, tileset.tile_height);
        let tile_count = numeric::Vector2u::new(
            (tiled_image.width as u32 / tile_size.x) as u32,
            (tiled_image.height as u32 / tile_size.y) as u32);

        let image = ggraphics::Image::new(ctx, format!("/{}", tiled_image.source)).unwrap();
        
        (TileSet {
            source_name: tileset.name.to_string(),
            tile_size: tile_size,
            tile_size_ratio: numeric::Vector2f::new(tile_size.x as f32 / image.width() as f32,
                                                    tile_size.y as f32 / image.height() as f32),
            tile_count: tile_count,
            drwob_essential: tg::DrawableObjectEssential::new(true, 0),
            draw_param: ggraphics::DrawParam::default(),
            first_gid: tileset.first_gid,
        }, image)
    }

    fn gid_to_crop(&self, gid: u32) -> numeric::Rect {
        let r_gid = gid - self.first_gid;
        let rows = r_gid / self.tile_count.x;
        let cols = r_gid % self.tile_count.x;

        numeric::Rect::new(cols as f32 * self.tile_size_ratio.x,
                           rows as f32 * self.tile_size_ratio.y,
                           self.tile_size_ratio.x,
                           self.tile_size_ratio.y)
    }

    fn total_tile_count(&self) -> u32 {
        self.tile_count.x * self.tile_count.y
    }
    
    fn contains_gid(&self, gid: u32) -> bool {
        gid >= self.first_gid && gid < self.first_gid + self.total_tile_count()
    }

    fn get_first_gid(&self) -> u32 {
        self.first_gid
    }
}

pub struct StageObjectMap {
    tile_map: tiled::Map,
    tilesets: Vec<TileSet>,
    tilesets_batchs: HashMap<u32, ggraphics::spritebatch::SpriteBatch>,
    drwob_essential: tg::DrawableObjectEssential,
}

impl StageObjectMap {
    pub fn new(ctx: &mut ggez::Context, path: &str) -> StageObjectMap {
        let tile_map = tiled::parse_file(std::path::Path::new(path)).unwrap();
        let mut batchs = HashMap::new();
        let tilesets = tile_map.tilesets.iter()
            .map(|ts| { let (ts, image) = TileSet::new(ctx, &ts);
                        batchs.insert(ts.first_gid, ggraphics::spritebatch::SpriteBatch::new(image));
                        ts })
            .collect();
        
        StageObjectMap {
            tile_map: tile_map,
            tilesets: tilesets,
            tilesets_batchs: batchs,
            drwob_essential: tg::DrawableObjectEssential::new(true, 0),
        }
    }

    pub fn print_info(&self, ctx: &mut ggez::Context) {

        for tile_set in &self.tile_map.tilesets {
            TileSet::new(ctx, tile_set);
        }

    }

    fn get_tileset_by_gid(&self, gid: u32) -> Option<&TileSet> {
        for tileset in &self.tilesets {
            if tileset.contains_gid(gid) {
                return Some(tileset);
            }
        }

        None
    }
}

impl tg::DrawableObject for StageObjectMap {
    fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        for (_, batch) in &self.tilesets_batchs {
            ggraphics::draw(ctx, batch, ggraphics::DrawParam {
                dest: numeric::Point2f::new(0.0, 0.0).into(),
                .. Default::default()
            })?;
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

impl Updatable for StageObjectMap {
    fn update(&mut self, _ctx: &ggez::Context, _t: Clock) -> Result<(), &'static str> {
        
        for layer in self.tile_map.layers.iter() {

            if !layer.visible {
                continue;
            }

            for (y, row) in layer.tiles.iter().enumerate() {
                for (x, &gid) in row.iter().enumerate() {
                    
                    if gid == 0 {
                        continue;
                    }
                    
                    let tileset = self.get_tileset_by_gid(gid).unwrap();
                    let crop = tileset.gid_to_crop(gid);
                    let first_gid = tileset.get_first_gid();
                    let tile_size = tileset.tile_size;
                    let batch = self.tilesets_batchs.get_mut(&first_gid).unwrap();
                    
                    batch.add(ggraphics::DrawParam {
                        src: numeric::Rect::new(crop.x, crop.y, crop.w, crop.h),
                        scale: numeric::Vector2f::new(0.1, 0.1).into(),
                        dest: numeric::Point2f::new((x as f32 * tile_size.x as f32) * 0.1,
                                                    (y as f32 * tile_size.y as f32) * 0.1).into(),
                        .. Default::default()
                    });

                }
            }
        }

        Ok(())
    }
}
