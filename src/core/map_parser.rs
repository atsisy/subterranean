use std::collections::HashMap;

use tiled;
use torifune::numeric;
use ggez::graphics as ggraphics;

use torifune::graphics as tg;
use torifune::core::*;

use collision::primitive::Primitive2 as CollisionType;

///
/// # TileSetの情報を保持している構造体
/// ## フィールド
/// ### tile_size
/// タイルのサイズ。読み込まれた画像は、このサイズで分割される。描画する最小単位。
///
/// ### tile_size_ratio
/// タイルのサイズと画像全体のサイズの比。オリジナルの画像が100でタイルが10なら、0.1が入る
///
/// ### tile_count
/// 何個のタイルに画像が分割されているかが格納される
///
/// ### first_gid
/// このtilesetのfirst_gidを格納している
///
pub struct TileSet {
    tile_size: numeric::Vector2u,
    tile_size_ratio: numeric::Vector2f,
    tile_count: numeric::Vector2u,
    first_gid: u32,
    collision_info: HashMap<u32, CollisionType<f32>>
}

impl TileSet {
    fn new(ctx: &mut ggez::Context, tileset: &tiled::Tileset) -> (TileSet, ggraphics::Image) {

        // tilesetが使用する画像を読み込む
        let tiled_image = tileset.images.get(0).unwrap();
        let mut collision_info = HashMap::new();

        for tile in &tileset.tiles {
            if let Some(group) = &tile.objectgroup {
                for object in &group.objects {
                    println!("{:?}", object);
                    
                    let c = match &object.shape {
                        &tiled::ObjectShape::Rect{ width, height } =>
                            Some(CollisionType::Rectangle(collision::primitive::Rectangle::new(width, height))),
                        tiled::ObjectShape::Polygon{ points } =>
                            Some(CollisionType::ConvexPolygon(
                                collision::primitive::ConvexPolygon::new(
                                    points.iter().map(|(x, y)| cgmath::Point2::<f32>::new(*x, *y)).collect()))),
                        _ => None,
                    };
                    
                    if let Some(cobj) = c {
                        collision_info.insert(tile.id, cobj);
                    }
                }
            }
        }

        // タイルのサイズとタイルの総数を計算
        let tile_size = numeric::Vector2u::new(tileset.tile_width, tileset.tile_height);
        let tile_count = numeric::Vector2u::new(
            (tiled_image.width as u32 / tile_size.x) as u32,
            (tiled_image.height as u32 / tile_size.y) as u32);

        // 使用する画像のパスから、ggezの画像を生成
        let image = ggraphics::Image::new(ctx, format!("/{}", tiled_image.source)).unwrap();

        // 新しいTileSetと読み込んだ画像データを返す
        (TileSet {
            tile_size: tile_size,
            tile_size_ratio: numeric::Vector2f::new(tile_size.x as f32 / image.width() as f32,
                                                    tile_size.y as f32 / image.height() as f32),
            tile_count: tile_count,
            first_gid: tileset.first_gid,
            collision_info: collision_info,
        }, image)
    }

    /// gidから、Tilesetのクロップ範囲を計算して返す
    fn gid_to_crop(&self, gid: u32) -> numeric::Rect {
        // 相対的なgid
        let r_gid = gid - self.first_gid;

        // 行と列を計算
        let rows = r_gid / self.tile_count.x;
        let cols = r_gid % self.tile_count.x;

        numeric::Rect::new(cols as f32 * self.tile_size_ratio.x, // クロップのx始点
                           rows as f32 * self.tile_size_ratio.y, // クロップのy始点
                           self.tile_size_ratio.x, // クロップの大きさ
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

///
/// # マップエディタで作ったマップを表示するための構造体
/// ## フィールド
/// ### tile_map
/// 読み込んだときの生データ
///
/// ### tilesets
/// 使用するタイルセットの情報
///
/// ### tilesets_batchs
/// タイルセットを描画するためのSpriteBatchたち
///
/// ### drwob_essential
/// 描画を行うときの情報
///
pub struct StageObjectMap {
    tile_map: tiled::Map,
    tilesets: Vec<TileSet>,
    tilesets_batchs: HashMap<u32, ggraphics::spritebatch::SpriteBatch>,
    drwob_essential: tg::DrawableObjectEssential,
}

impl StageObjectMap {
    pub fn new(ctx: &mut ggez::Context, path: &str) -> StageObjectMap {
        // マップ情報を読み込む
        let tile_map = tiled::parse_file(std::path::Path::new(path)).unwrap();

        // タイルセットを読み込み、それと同時にタイルセットの画像からSpriteBatchを生成する
        let mut batchs = HashMap::new();
        let tilesets: Vec<TileSet> = tile_map.tilesets.iter()
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

    /// 全てのsprite batch処理をクリアするメソッド
    fn clear_all_batchs(&mut self) {
        for (_, batch) in &mut self.tilesets_batchs {
            batch.clear();
        }
    }

    /// gidから、どのタイルセットを利用するかを決定するメソッド
    fn get_tileset_by_gid(&self, gid: u32) -> Option<&TileSet> {
        for tileset in &self.tilesets {
            if tileset.contains_gid(gid) {
                return Some(tileset);
            }
        }

        None
    }

    /// sprite batch処理を実際に行うメソッド
    fn update_sprite_batch(&mut self) {
        // batch処理を全てクリア
        self.clear_all_batchs();

        // 利用するスケール
        // FIXME 任意定数を設定できるようni
        let scale = numeric::Vector2f::new(1.0, 1.0);

        // 全てのレイヤーで描画を実行
        for layer in self.tile_map.layers.iter() {
            if !layer.visible {
                // レイヤーが非表示設定になっていれば、描画は行わない
                continue;
            }
            
            // 二次元のマップデータを全てbatch処理に掛ける
            for (y, row) in layer.tiles.iter().enumerate() {
                for (x, &tile) in row.iter().enumerate() {
                    let gid = tile.gid;
                    // gidが0のときは、何も配置されていない状態を表すので、描画は行わない
                    if gid == 0 {
                        continue;
                    }

                    let tileset = self.get_tileset_by_gid(gid).unwrap(); // 目的のタイルセットを取り出す
                    let crop = tileset.gid_to_crop(gid); // クロップする部分をgidから計算
                    let first_gid = tileset.get_first_gid(); // batch処理を行うタイルセットのfirst_gidを取得
                    let tile_size = tileset.tile_size; // 利用するタイルセットのタイルサイズを取得
                    let batch = self.tilesets_batchs.get_mut(&first_gid).unwrap(); // sprite batchをfirst_gidから取得

                    // batch処理を追加
                    batch.add(ggraphics::DrawParam {
                        src: numeric::Rect::new(crop.x, crop.y, crop.w, crop.h),
                        scale: scale.into(),
                        dest: numeric::Point2f::new((x as f32 * tile_size.x as f32) * scale.x,
                                                    (y as f32 * tile_size.y as f32) * scale.y).into(),
                        .. Default::default()
                    });
                    
                }
            }
        }
    }
}

impl tg::DrawableObject for StageObjectMap {
    fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        // 全てのsprite batchを描画
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
    fn update(&mut self, _ctx: &ggez::Context, _t: Clock) {
        self.update_sprite_batch();
    }
}
