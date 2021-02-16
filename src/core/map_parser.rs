use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use pathfinding::prelude::dijkstra;

use ggez::graphics as ggraphics;
use tiled;
use torifune::numeric;

use sub_screen::SubScreen;
use torifune::core::*;
use torifune::graphics::drawable::*;
use torifune::graphics::object::sub_screen;
use torifune::{mintv, mintp};

use collision::prelude::*;

use crate::object::collision::*;
use crate::object::map_object::MapObject;

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
    collision_info: HashMap<u32, Vec<CollisionType>>,
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
                        &tiled::ObjectShape::Rect { width, height } => {
                            Some(CollisionType::Rect(collision::Aabb2::new(
                                cgmath::Point2::<f32>::new(object.x as f32, object.y as f32),
                                cgmath::Point2::<f32>::new(
                                    (object.x + width) as f32,
                                    (object.y + height) as f32,
                                ),
                            )))
                        }
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
            (tiled_image.height as u32 / tile_size.y) as u32,
        );

        // 使用する画像のパスから、ggezの画像を生成
        let image = ggraphics::Image::new(ctx, format!("/{}", tiled_image.source)).unwrap();

        // 新しいTileSetと読み込んだ画像データを返す
        (
            TileSet {
                tile_size: tile_size,
                tile_size_ratio: numeric::Vector2f::new(
                    tile_size.x as f32 / image.width() as f32,
                    tile_size.y as f32 / image.height() as f32,
                ),
                tile_count: tile_count,
                first_gid: tileset.first_gid,
                collision_info: collision_info,
            },
            image,
        )
    }

    fn exist_collision_info(&self, gid: u32) -> bool {
        let r_gid = gid - self.first_gid;
        self.collision_info.contains_key(&r_gid)
    }

    ///
    /// キャラクターと特定のタイルとの当たり判定を行う
    ///
    fn __check_character_collision(
        &self,
        ctx: &mut ggez::Context,
        tile_col: &collision::Aabb2<f32>,
        chara: &MapObject,
    ) -> CollisionInformation {
        let area = chara.get_collision_area(ctx);

        // キャラクターが描画されている領域をaabbで表現
        let rect: collision::Aabb2<f32> = collision::Aabb2::<f32>::new(
            cgmath::Point2::<f32>::new(area.x as f32, area.y as f32),
            cgmath::Point2::<f32>::new((area.x + area.w) as f32, (area.y + area.h) as f32),
        );

        // 衝突しているか？
        if tile_col.intersects(&rect) {
            // ここでは、返す衝突情報を計算する

            // タイルが配置されている、カメラ上の相対描画位置
            let tile_pos = numeric::Vector2f::new(tile_col.min.x as f32, tile_col.min.y as f32);
            // タイルのサイズ
            let tile_size = numeric::Vector2f::new(
                (tile_col.min.x - tile_col.max.x).abs(),
                (tile_col.min.y - tile_col.max.y).abs(),
            );

            return CollisionInformation::new_collision(
                ggraphics::Rect::new(tile_pos.x, tile_pos.y, tile_size.x, tile_size.y), // タイルの位置とサイズ
                chara.get_collision_area(ctx), // キャラクターの位置
                numeric::Vector2f::new(
                    rect.center().x - tile_col.center().x,
                    rect.center().y - tile_col.center().y,
                ), // お互いの中心同士の距離（ベクタ）
            );
        } else {
            return CollisionInformation::new_not_collision();
        }
    }

    fn check_character_collision(
        &self,
        ctx: &mut ggez::Context,
        gid: u32,
        _abs_pos: numeric::Point2f,
        offset: numeric::Point2f,
        scale: numeric::Vector2f,
        chara: &MapObject,
    ) -> CollisionInformation {
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
                }
                _ => (),
            }
        }

        CollisionInformation::new_not_collision()
    }

    pub fn gid_to_location_index(&self, gid: u32) -> numeric::Vector2u {
        // 相対的なgid
        let r_gid = gid - self.first_gid;

        // 行と列を計算
        let rows = r_gid / self.tile_count.x;
        let cols = r_gid % self.tile_count.x;

        numeric::Vector2u::new(cols, rows)
    }

    pub fn is_collisionable_tile(&self, gid: u32) -> bool {
        self.contains_gid(gid) && self.collision_info.contains_key(&gid)
    }

    /// gidから、Tilesetのクロップ範囲を計算して返す
    fn gid_to_crop(&self, gid: u32) -> numeric::Rect {
        let location = self.gid_to_location_index(gid);

        numeric::Rect::new(
            location.x as f32 * self.tile_size_ratio.x, // クロップのx始点
            location.y as f32 * self.tile_size_ratio.y, // クロップのy始点
            self.tile_size_ratio.x,                     // クロップの大きさ
            self.tile_size_ratio.y,
        )
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

#[derive(Clone, Copy)]
pub struct CollisionMapNode {
    pub done: bool,
    pub cost: i32,
}

pub struct CollisionMap {
    map: Vec<Vec<CollisionMapNode>>,
    size: numeric::Vector2u,
    collision_objects: Vec<numeric::Vector2u>,
}

impl CollisionMap {
    pub fn new(size: numeric::Vector2u, collision_objects: Vec<numeric::Vector2u>) -> Self {
        CollisionMap {
            map: vec![
                vec![
                    CollisionMapNode {
                        done: false,
                        cost: -1
                    };
                    size.y as usize
                ];
                size.x as usize
            ],
            size: size,
            collision_objects: collision_objects,
        }
    }

    pub fn not_contains(&self, position: numeric::Point2i) -> bool {
        position.x < 0
            && position.x >= self.size.x as i32
            && position.y < 0
            && position.y >= self.size.y as i32
    }

    fn is_movable_position(&self, x: i32, y: i32) -> bool {
        !self.not_contains(numeric::Point2i::new(x, y))
            && !self
                .collision_objects
                .contains(&numeric::Vector2u::new(x as u32, y as u32))
    }

    pub fn get(&self, position: numeric::Point2i) -> Option<&CollisionMapNode> {
        if let Some(row) = self.map.get(position.x as usize).as_ref() {
            row.get(position.y as usize).as_ref();
        }

        None
    }

    pub fn find_path(
        &self,
        start: numeric::Point2i,
        goal: numeric::Point2i,
    ) -> Option<Vec<numeric::Point2i>> {
        let result = dijkstra(
            &start,
            |point| self.successors(point),
            |point| point.eq(&goal),
        );

        if let Some((path, _)) = result {
            Some(path)
        } else {
            None
        }
    }

    fn successors(&self, point: &numeric::Point2i) -> Vec<(numeric::Point2i, usize)> {
        let mut successors_list = Vec::new();

        let cand_point = numeric::Point2i::new(point.x as i32, point.y as i32 - 1);
        if self.is_movable_position(cand_point.x, cand_point.y) {
            successors_list.push(cand_point);
        }

        let cand_point = numeric::Point2i::new(point.x as i32, point.y as i32 + 1);
        if self.is_movable_position(cand_point.x, cand_point.y) {
            successors_list.push(cand_point);
        }

        let cand_point = numeric::Point2i::new(point.x as i32 - 1, point.y as i32);
        if self.is_movable_position(cand_point.x, cand_point.y) {
            successors_list.push(cand_point);
        }

        let cand_point = numeric::Point2i::new(point.x as i32 + 1, point.y as i32);
        if self.is_movable_position(cand_point.x, cand_point.y) {
            successors_list.push(cand_point);
        }

        successors_list.into_iter().map(|p| (p, 1)).collect()
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
    collision_map: Option<CollisionMap>,
    camera: Rc<RefCell<numeric::Rect>>,
    scale: numeric::Vector2f,
    redraw_request: bool,
    update_batch_request: bool,
    canvas: SubScreen,
}

impl StageObjectMap {
    pub fn new(
        ctx: &mut ggez::Context,
        path: &str,
        camera: Rc<RefCell<numeric::Rect>>,
        canvas_rect: numeric::Rect,
        scale: numeric::Vector2f,
    ) -> StageObjectMap {
        // マップ情報を読み込む
	println!("FIXME: map_parse.rs StageObjectMap::new");
        //let file = ggez::filesystem::open(ctx, path).unwrap();
	//let tile_map = tiled::parse(file).unwrap();
	let tile_map = tiled::parse_file(std::path::Path::new(path)).unwrap();
	
        // タイルセットを読み込み、それと同時にタイルセットの画像からSpriteBatchを生成する
        let mut batchs = HashMap::new();
        let tilesets: Vec<TileSet> = tile_map
            .tilesets
            .iter()
            .map(|ts| {
                let (ts, image) = TileSet::new(ctx, &ts);
                let mut batch = ggraphics::spritebatch::SpriteBatch::new(image);
                batch.set_filter(ggraphics::FilterMode::Nearest);

                batchs.insert(ts.first_gid, batch);
                ts
            })
            .collect();

        let mut canvas = SubScreen::new(ctx, canvas_rect, 0, ggraphics::Color::from_rgba_u32(0));
        canvas.set_filter(ggraphics::FilterMode::Nearest);

        StageObjectMap {
            tile_map: tile_map,
            tilesets: tilesets,
            tilesets_batchs: batchs,
            collision_map: None,
            camera: camera,
            scale: scale,
            canvas: canvas,
            redraw_request: true,
            update_batch_request: true,
        }
    }

    /// 引数で受け取ったタイルの情報から、そのタイルがカメラに写るか調べるメソッド
    fn tile_is_inside_of_camera(&self, dest: numeric::Point2f, size: numeric::Vector2u) -> bool {
        let rect = numeric::Rect::new(
            dest.x,
            dest.y,
            dest.x + (size.x as f32 * self.scale.x),
            dest.y + (size.y as f32 * self.scale.y),
        );
        self.camera.borrow().overlaps(&rect)
    }

    /// タイルが配置されるであろう座標を計算するメソッド
    fn calc_tile_dest_point(
        x: u32,
        y: u32,
        tile_size: numeric::Vector2u,
        scale: numeric::Vector2f,
    ) -> numeric::Point2f {
        numeric::Point2f::new(
            x as f32 * tile_size.x as f32 * scale.x,
            y as f32 * tile_size.y as f32 * scale.y,
        )
    }

    pub fn request_redraw(&mut self) {
        self.redraw_request = true;
    }

    pub fn request_updating_tile_batch(&mut self) {
        self.update_batch_request = true;
    }

    /// ある座標が、カメラに写ったときの座標を返すメソッド
    fn camera_relative_position(&self, p: numeric::Point2f) -> numeric::Point2f {
        numeric::Point2f::new(p.x - self.camera.borrow().x, p.y - self.camera.borrow().y)
    }

    pub fn check_character_collision(
        &self,
        ctx: &mut ggez::Context,
        chara: &MapObject,
    ) -> CollisionInformation {
        // 全てのレイヤーで衝突検査
        for layer in self.tile_map.layers.iter() {
            if !layer.visible {
                // レイヤーが非表示設定になっていれば、無視
                continue;
            }

            let overlap_tiles_nums = self.overlappable_tiles_nums(chara.get_collision_size(ctx));
            let map_position = chara.get_map_position_with_collision_top_offset(ctx);
            let global_tile_size = self.get_tile_drawing_size();
            let map_tile_pos = numeric::Vector2i::new(
                (map_position.x as f32 / global_tile_size.x).round() as i32,
                (map_position.y as f32 / global_tile_size.y).round() as i32,
            );

            for x_offset in -1..(overlap_tiles_nums.x - 1) as i32 {
                for y_offset in -1..(overlap_tiles_nums.y - 1) as i32 {
                    let (x, y) = (map_tile_pos.x + x_offset, map_tile_pos.y + y_offset);

                    if x < 0 || y < 0 {
                        continue;
                    }

                    let tiles = match &layer.tiles {
                        tiled::LayerData::Finite(tiles) => tiles,
                        _ => panic!(""),
                    };
                    let tile = match tiles.get(y as usize) {
                        Some(row) => match row.get(x as usize) {
                            Some(tile) => tile,
                            None => continue,
                        },
                        None => continue,
                    };

                    let gid = tile.gid;
                    // gidが0のときは、何も配置されていない状態を表すので、描画は行わない
                    if gid == 0 {
                        continue;
                    }

                    let tileset = self.get_tileset_by_gid(gid).unwrap(); // 目的のタイルセットを取り出す
                    let tile_size = tileset.tile_size; // 利用するタイルセットのタイルサイズを取得

                    let dest_pos =
                        Self::calc_tile_dest_point(x as u32, y as u32, tile_size, self.scale);

                    // カメラに入っていないマップチップは描画しない
                    if !self.tile_is_inside_of_camera(dest_pos, tile_size) {
                        continue;
                    }

                    for loop_tileset in &self.tilesets {
                        if loop_tileset.exist_collision_info(gid) {
                            let info = loop_tileset.check_character_collision(
                                ctx,
                                gid,
                                dest_pos,
                                self.camera_relative_position(dest_pos),
                                self.scale,
                                chara,
                            );
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

    pub fn get_tile_size(&self) -> numeric::Point2u {
        numeric::Point2u::new(self.tile_map.tile_width, self.tile_map.tile_height)
    }

    pub fn get_tile_scale(&self) -> numeric::Vector2f {
        self.scale
    }

    pub fn get_tile_drawing_size(&self) -> numeric::Vector2f {
        numeric::Vector2f::new(
            (self.tile_map.tile_width as f32) * self.scale.x,
            (self.tile_map.tile_height as f32) * self.scale.y,
        )
    }

    fn overlappable_tiles_nums(&self, obj_size: numeric::Vector2f) -> numeric::Vector2u {
        let tile_size = self.get_tile_drawing_size();
        numeric::Vector2u::new(
            (obj_size.x / tile_size.x as f32).ceil() as u32 + 2,
            (obj_size.y / tile_size.y as f32).ceil() as u32 + 2,
        )
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

    /// gidが指すタイルが衝突判定ありか？
    fn is_collisionable_tile(&self, gid: u32) -> bool {
        for tileset in &self.tilesets {
            let r_gid = gid - tileset.first_gid;
            if tileset.is_collisionable_tile(r_gid) {
                return true;
            }
        }

        false
    }

    /// sprite batch処理を実際に行うメソッド
    fn update_sprite_batch(&mut self) {
        if !self.update_batch_request {
            return;
        }

        self.update_batch_request = false;

        // batch処理を全てクリア
        self.clear_all_batchs();

        // 全てのレイヤーで描画を実行
        for layer in self.tile_map.layers.iter() {
            if !layer.visible {
                // レイヤーが非表示設定になっていれば、描画は行わない
                continue;
            }

            let tiles = match &layer.tiles {
                tiled::LayerData::Finite(tiles) => tiles,
                _ => panic!(""),
            };
            // 二次元のマップデータを全てbatch処理に掛ける
            for (y, row) in tiles.iter().enumerate() {
                for (x, &tile) in row.iter().enumerate() {
                    let gid = tile.gid;
                    // gidが0のときは、何も配置されていない状態を表すので、描画は行わない
                    if gid == 0 {
                        continue;
                    }

                    let tileset = self.get_tileset_by_gid(gid).unwrap(); // 目的のタイルセットを取り出す
                    let tile_size = tileset.tile_size; // 利用するタイルセットのタイルサイズを取得

                    let dest_pos =
                        Self::calc_tile_dest_point(x as u32, y as u32, tile_size, self.scale);

                    // カメラに入っていないマップチップは描画しない
                    if !self.tile_is_inside_of_camera(dest_pos, tile_size) {
                        continue;
                    }

                    let crop = tileset.gid_to_crop(gid); // クロップする部分をgidから計算
                    let first_gid = tileset.get_first_gid(); // batch処理を行うタイルセットのfirst_gidを取得

                    let batch = self.tilesets_batchs.get_mut(&first_gid).unwrap(); // sprite batchをfirst_gidから取得

		    let draw_param = ggraphics::DrawParam::default()
			.src(numeric::Rect::new(crop.x, crop.y, crop.w, crop.h))
			.scale(mintv!(self.scale))
			.dest(mintp!(dest_pos));
                    // batch処理を追加
                    batch.add(draw_param);
                }
            }
        }
    }

    fn search_collision_locations(&self) -> Vec<numeric::Vector2u> {
        let mut collision_locations = Vec::new();

        for layer in self.tile_map.layers.iter() {
            println!("name -> {}", layer.name);
            if !layer.visible {
                // レイヤーが非表示設定になっていれば、衝突オブジェクトの検索を行わない
                continue;
            }

            let tiles = match &layer.tiles {
                tiled::LayerData::Finite(tiles) => tiles,
                _ => panic!(""),
            };
            // 二次元のマップデータを全てbatch処理に掛ける
            for (y, row) in tiles.iter().enumerate() {
                for (x, &tile) in row.iter().enumerate() {
                    let gid = tile.gid;

                    // gidが0のときは、何も配置されていない状態を表すので、描画は行わない
                    if gid == 0 {
                        continue;
                    }

                    if self.is_collisionable_tile(gid) {
                        collision_locations.push(numeric::Vector2u::new(x as u32, y as u32));
                    }
                }
            }
        }

        collision_locations
    }

    pub fn build_collision_map(&mut self) {
        let collision_points = self.search_collision_locations();

        self.collision_map = Some(CollisionMap::new(
            numeric::Vector2u::new(self.tile_map.width, self.tile_map.height),
            collision_points.clone(),
        ));
    }

    pub fn map_position_to_tile_position(
        &self,
        point: numeric::Point2f,
    ) -> Option<numeric::Vector2u> {
        if point.x < 0.0 || point.y < 0.0 {
            return None;
        }

        let tile_size = self.get_tile_size();
        Some(numeric::Vector2u::new(
            ((point.x / tile_size.x as f32) / self.scale.x) as u32,
            ((point.y / tile_size.y as f32) / self.scale.y) as u32,
        ))
    }

    pub fn tile_position_to_map_position(&self, point: numeric::Vector2u) -> numeric::Point2f {
        let tile_size = self.get_tile_size();

        numeric::Point2f::new(
            (point.x * tile_size.x) as f32 * self.scale.x,
            (point.y * tile_size.y) as f32 * self.scale.y,
        )
    }

    pub fn find_shortest_route(
        &self,
        start: numeric::Vector2u,
        goal: numeric::Vector2u,
    ) -> Option<Vec<numeric::Vector2u>> {
        if let Some(collision_map) = self.collision_map.as_ref() {
            if let Some(path) = collision_map.find_path(
                numeric::Point2i::new(start.x as i32, start.y as i32),
                numeric::Point2i::new(goal.x as i32, goal.y as i32),
            ) {
                Some(
                    path.iter()
                        .map(|p| numeric::Vector2u::new(p.x as u32, p.y as u32))
                        .collect(),
                )
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_map_size(&self) -> numeric::Vector2f {
        let tile_size = self.get_tile_drawing_size();

        numeric::Vector2f::new(
            self.tile_map.width as f32 * tile_size.x,
            self.tile_map.height as f32 * tile_size.y,
        )
    }
}

impl DrawableComponent for StageObjectMap {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            if self.redraw_request {
                sub_screen::stack_screen(ctx, &self.canvas);

                // 全てのsprite batchを描画
                for (_, batch) in &self.tilesets_batchs {
		    let draw_param = ggraphics::DrawParam::default()
			.dest(mintp!(numeric::Point2f::new(
                            -self.camera.borrow().x.round(),
                            -self.camera.borrow().y.round(),
                        )));
		    
                    ggraphics::draw(
                        ctx,
                        batch,
			draw_param,
                    )?;
                }

                sub_screen::pop_screen(ctx);
                self.redraw_request = false;
            }
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

impl DrawableObject for StageObjectMap {}

impl Updatable for StageObjectMap {
    fn update(&mut self, _ctx: &mut ggez::Context, _t: Clock) {
        self.update_sprite_batch();
    }
}

pub fn map_to_display(map_pos: &numeric::Point2f, camera: &numeric::Rect) -> numeric::Point2f {
    numeric::Point2f::new(map_pos.x - camera.x, map_pos.y - camera.y)
}
