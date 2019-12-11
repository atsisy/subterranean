use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use tiled;
use torifune::numeric;
use ggez::graphics as ggraphics;

use torifune::graphics as tg;
use torifune::core::*;
use torifune::graphics::DrawableObject;
use torifune::graphics::object::TextureObject;

use collision::prelude::*;

use crate::object::collision::*;
use crate::object::Character;

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
    collision_info: HashMap<u32, Vec<CollisionType>>
}

impl TileSet {
    fn new(ctx: &mut ggez::Context, tileset: &tiled::Tileset) -> (TileSet, ggraphics::Image) {

        // tilesetが使用する画像を読み込む
        let tiled_image = tileset.images.get(0).unwrap();
        let mut collision_info: HashMap<u32, Vec<CollisionType>> = HashMap::new();

        for tile in &tileset.tiles {
            if let Some(group) = &tile.objectgroup {
                for object in &group.objects {
                    let c = match &object.shape {
                        &tiled::ObjectShape::Rect{ width, height } =>
                            Some(CollisionType::Rect(collision::Aabb2::new(cgmath::Point2::<f32>::new(object.x as f32,
                                                                                                      object.y as f32),
                                                                           cgmath::Point2::<f32>::new(
                                                                               (object.x + width) as f32,
                                                                               (object.y + height) as f32)))),
                        _ => None,
                    };
                    
                    if let Some(cobj) = c {
                        if collision_info.contains_key(&tile.id) {
                            collision_info.get_mut(&tile.id).unwrap().push(cobj);
                        } else {
                            collision_info.insert(tile.id, vec![cobj]);
                        }
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

    fn exist_collision_info(&self, gid: u32) -> bool {
        let r_gid = gid - self.first_gid;
        self.collision_info.contains_key(&r_gid)
    }

    ///
    /// キャラクターと特定のタイルとの当たり判定を行う
    ///
    fn __check_character_collision(&self, ctx: &mut ggez::Context,
                                   tile_col: &collision::Aabb2<f32>,
                                   chara: &Character) -> CollisionInformation {
        let area = chara.obj().get_drawing_area(ctx);
        // キャラクターが描画されている領域をaabbで表現
        let rect: collision::Aabb2<f32> = collision::Aabb2::<f32>::new(
            cgmath::Point2::<f32>::new(area.x as f32, area.y as f32),
            cgmath::Point2::<f32>::new((area.x + area.w) as f32, (area.y + area.h) as f32)
        );

        // 衝突しているか？
        if tile_col.intersects(&rect) {
            // ここでは、返す衝突情報を計算する
            
            // タイルが配置されている、カメラ上の相対描画位置
            let tile_pos = numeric::Vector2f::new(
                tile_col.min.x as f32,
                tile_col.min.y as f32);
            // タイルのサイズ
            let tile_size = numeric::Vector2f::new((tile_col.min.x - tile_col.max.x).abs(),
                                                   (tile_col.min.y - tile_col.max.y).abs());

            return CollisionInformation::new_collision(
                ggraphics::Rect::new(tile_pos.x, tile_pos.y, tile_size.x, tile_size.y), // タイルの位置とサイズ
                chara.obj().get_position(), // キャラクターの位置
                numeric::Vector2f::new(rect.center().x - tile_col.center().x, rect.center().y - tile_col.center().y) // お互いの中心同士の距離（ベクタ）
            );
        } else {
            return CollisionInformation::new_not_collision();
        }
    }
    
    fn check_character_collision(&self, ctx: &mut ggez::Context,
                                 gid: u32, abs_pos: numeric::Point2f,
                                 offset: numeric::Point2f,
                                 scale: numeric::Vector2f,
                                 chara: &Character) -> CollisionInformation {
        let r_gid = gid - self.first_gid;
        for c in self.collision_info.get(&r_gid).unwrap() {
            match c {
                CollisionType::Rect(aabb) => {
                    let mut cp = aabb.clone();
                    cp.min.x *= scale.x;
                    cp.min.y *= scale.y;

                    cp.max.x *= scale.x;
                    cp.max.y *= scale.y;
                    
                    cp.min.x += offset.x;
                    cp.min.y += offset.y;
                    cp.max.x += offset.x;
                    cp.max.y += offset.y;
                    
                    let info = self.__check_character_collision(ctx, &cp, chara);
                    if info.collision {
                        return info;
                    }
                },
                _ => (),
            }
        }

        CollisionInformation::new_not_collision()
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
    camera: Rc<RefCell<numeric::Rect>>,
    scale: numeric::Vector2f,
}

impl StageObjectMap {
    pub fn new(ctx: &mut ggez::Context, path: &str, camera: Rc<RefCell<numeric::Rect>>, scale: numeric::Vector2f) -> StageObjectMap {
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
            camera: camera,
            scale: scale,
        }
    }

    /// 引数で受け取ったタイルの情報から、そのタイルがカメラに写るか調べるメソッド
    fn tile_is_inside_of_camera(&self, dest: numeric::Point2f,
                                size: numeric::Vector2u) -> bool {
        let rect = numeric::Rect::new(dest.x, dest.y, dest.x + (size.x as f32 * self.scale.x), dest.y + (size.y as f32 * self.scale.y));
        self.camera.borrow().overlaps(&rect)
    }

    /// タイルが配置されるであろう座標を計算するメソッド
    fn calc_tile_dest_point(x: u32, y: u32, tile_size: numeric::Vector2u, scale: numeric::Vector2f) -> numeric::Point2f {
        numeric::Point2f::new(x as f32 * tile_size.x as f32 * scale.x,
                              y as f32 * tile_size.y as f32 * scale.y)
    }

    /// ある座標が、カメラに写ったときの座標を返すメソッド
    fn camera_relative_position(&self, p: numeric::Point2f) -> numeric::Point2f {
        numeric::Point2f::new(p.x - self.camera.borrow().x, p.y - self.camera.borrow().y)
    }

    pub fn check_character_collision(&self, ctx: &mut ggez::Context, chara: &Character) -> CollisionInformation {
        
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
                    let tile_size = tileset.tile_size; // 利用するタイルセットのタイルサイズを取得

                    let dest_pos = Self::calc_tile_dest_point(x as u32, y as u32, tile_size, self.scale);

                    // カメラに入っていないマップチップは描画しない
                    if !self.tile_is_inside_of_camera(dest_pos, tile_size) {
                        continue;
                    }

                    for loop_tileset in &self.tilesets {
                        if loop_tileset.exist_collision_info(gid) {
                            let info = loop_tileset.check_character_collision(ctx,
                                                                              gid,
                                                                              dest_pos,
                                                                              self.camera_relative_position(dest_pos),
                                                                              self.scale,
                                                                              chara);
                            if info.collision {
                                return info;
                            }
                        }
                    }
                }
            }
        }

        CollisionInformation::new_not_collision()
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
                    let tile_size = tileset.tile_size; // 利用するタイルセットのタイルサイズを取得

                    let dest_pos = Self::calc_tile_dest_point(x as u32, y as u32, tile_size, self.scale);

                    // カメラに入っていないマップチップは描画しない
                    if !self.tile_is_inside_of_camera(dest_pos, tile_size) {
                        continue;
                    }
                    
                    let crop = tileset.gid_to_crop(gid); // クロップする部分をgidから計算
                    let first_gid = tileset.get_first_gid(); // batch処理を行うタイルセットのfirst_gidを取得

                    let batch = self.tilesets_batchs.get_mut(&first_gid).unwrap(); // sprite batchをfirst_gidから取得

                    // batch処理を追加
                    batch.add(ggraphics::DrawParam {
                        src: numeric::Rect::new(crop.x, crop.y, crop.w, crop.h),
                        scale: self.scale.into(),
                        dest: dest_pos.into(),
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
                dest: numeric::Point2f::new(-self.camera.borrow().x.round(), -self.camera.borrow().y.round()).into(),
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
