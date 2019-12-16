pub mod move_fn;
pub mod collision;

use std::rc::Rc;
use std::cell::RefCell;

use ggez::graphics as ggraphics;

use torifune::core::Clock;
use torifune::core::Updatable;
use torifune::numeric;
use torifune::graphics::object as tobj;
use torifune::graphics::object::TextureObject;
use torifune::graphics::DrawableObject;
use torifune::graphics::object::MovableObject;

use crate::object::collision::*;
use crate::core::map_parser as mp;

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
    fall_begin: Clock,
    gravity_acc: f32,
    horizon_resistance: f32,
    speed: numeric::Vector2f,
    speed_border: SpeedBorder,
}

impl TextureSpeedInfo {
    pub fn new(gravity_acc: f32, horizon_res: f32, speed: numeric::Vector2f, border: SpeedBorder)
               -> TextureSpeedInfo {
        TextureSpeedInfo {
            fall_begin: 0,
            gravity_acc: gravity_acc,
            horizon_resistance: horizon_res,
            speed: speed,
            speed_border: border,
        }
    }

    pub fn fall_start(&mut self, t: Clock) {
        self.fall_begin = t;
    }

    pub fn apply_resistance(&mut self, t: Clock) {
        self.set_speed_y(self.speed.y + (self.gravity_acc * (t - self.fall_begin) as f32));
        
        if self.speed.x.abs() <= self.horizon_resistance {
            self.set_speed_x(0.0);
        }else {
            self.set_speed_x(self.speed.x + if self.speed.x > 0.0 { -self.horizon_resistance } else { self.horizon_resistance });
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

    fn set_gravity(&mut self, g: f32) {
        self.gravity_acc = g;
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
    
    pub fn next_frame(&mut self, t: AnimationType) -> Result<Rc<ggraphics::Image>, AnimationStatus> {
        self.index += 1;
        
        match t {
            AnimationType::OneShot | AnimationType::Times(_, _) => {
                if self.index == self.textures.len() {
                    return Err(AnimationStatus::OneLoopFinish);
                }
            },
            _ => (),
        }

        return Ok(self.current_frame())
    }
}

pub struct TextureAnimation {
    textures: Vec<SeqTexture>,
    current_mode: usize,
    object: tobj::SimpleObject,
    animation_type: AnimationType,
    next_mode: usize,
    frame_speed: Clock,
}

impl TextureAnimation {
    pub fn new(obj: tobj::SimpleObject, textures: Vec<Vec<Rc<ggraphics::Image>>>, mode: usize, frame_speed: Clock) -> Self {
        TextureAnimation {
            textures: textures.iter().map(|vec| SeqTexture::new(vec.to_vec())).collect(),
            current_mode: mode,
            object: obj,
            animation_type: AnimationType::Loop,
            next_mode: mode,
            frame_speed: frame_speed,
        }
    }

    pub fn get_object(&self) -> &tobj::SimpleObject {
        &self.object
    }

    pub fn get_mut_object(&mut self) -> &mut tobj::SimpleObject {
        &mut self.object
    }

    pub fn change_mode(&mut self, mode: usize, animation_type: AnimationType, next_mode: usize) {
        self.current_mode = mode;
        self.next_mode = next_mode;
        self.animation_type = animation_type;
        self.textures[self.current_mode].reset();
    }

    fn next_frame(&mut self) {
        match self.textures[self.current_mode].next_frame(self.animation_type) {
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
                            },
                            &AnimationType::Times(mut cur, lim) => {
                                // Timesの場合
                                // ループカウンタをインクリメントする
                                cur += 1;

                                // まだループする予定
                                if cur < lim {
                                    // 最初のテクスチャに戻し、アニメーションを再開
                                    self.textures[self.current_mode].reset();
                                    let texture = self.textures[self.current_mode].current_frame();
                                    self.get_mut_object().replace_texture(texture);
                                } else {
                                    // OneShotの場合と同じく、デフォルトのループに切り替える
                                    self.animation_type = AnimationType::Loop;
                                    self.current_mode = self.next_mode;
                                }
                            },
                            _ => (),
                        }
                    },
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
}

pub struct TwoStepPoint {
    pub previous: numeric::Point2f,
    pub current: numeric::Point2f,
}

impl TwoStepPoint {
    pub fn diff(&self) -> numeric::Vector2f {
        self.current - self.previous
    }

    pub fn update(&mut self, pos: numeric::Point2f) {
        self.previous = self.current;
        self.current = pos;
    }

    pub fn move_diff(&mut self, pos: &numeric::Vector2f) {
        self.previous = self.current;
        self.current += *pos;
    }
}

pub struct Character {
    last_position: numeric::Point2f,
    object: TextureAnimation,
    speed_info: TextureSpeedInfo,
    map_position: TwoStepPoint,
}

impl Character {
    pub fn new(obj: tobj::SimpleObject, textures: Vec<Vec<Rc<ggraphics::Image>>>,
               mode: usize, speed_info: TextureSpeedInfo, map_position: numeric::Point2f,
               frame_speed: Clock) -> Character {
        Character {
            last_position: obj.get_position(),
            map_position: TwoStepPoint { previous: map_position, current: map_position },
            speed_info: speed_info,
            object: TextureAnimation::new(obj, textures, mode, frame_speed),
        }
    }

    pub fn speed_info(&self) -> &TextureSpeedInfo {
        &self.speed_info
    }

    pub fn speed_info_mut(&mut self) -> &mut TextureSpeedInfo {
        &mut self.speed_info
    }

    pub fn obj(&self) -> &tobj::SimpleObject {
        self.object.get_object()
    }
    
    pub fn obj_mut(&mut self) -> &mut tobj::SimpleObject {
        self.object.get_mut_object()
    }

    pub fn get_last_position(&self) -> numeric::Point2f {
        self.last_position
    }

    pub fn undo_move(&mut self) {
        let last = self.get_last_position();
        self.object.get_mut_object().set_position(last);
    }

    fn get_last_move_distance(&self) -> numeric::Vector2f {
        let current = self.object.get_object().get_position();
        numeric::Vector2f::new(
            current.x - self.last_position.x,
            current.y - self.last_position.y,
        )
    }

    pub fn get_last_map_move_distance(&self) -> numeric::Vector2f {
        self.map_position.diff()
    }

    pub fn get_map_position(&self) -> numeric::Point2f {
        self.map_position.current
    }

    ///
    /// キャラクタテクスチャの上側が衝突した場合
    /// どれだけ、テクスチャを移動させれば良いのかを返す
    ///
    fn fix_collision_above(&mut self,
                           _ctx: &mut ggez::Context,
                           info: &CollisionInformation,
                           t: Clock) -> f32 {
        self.speed_info.fall_start(t);
        self.speed_info.set_speed_y(1.0);
        (info.tile_position.unwrap().y + info.tile_position.unwrap().h + 0.1) - info.player_position.unwrap().y
    }

    ///
    /// キャラクタテクスチャの下側が衝突した場合
    /// どれだけ、テクスチャを移動させれば良いのかを返す
    ///
    fn fix_collision_bottom(&mut self,
                            ctx: &mut ggez::Context,
                            info: &CollisionInformation,
                            t: Clock) -> f32 {
        self.speed_info.fall_start(t);
        self.speed_info.set_speed_x(0.0);
        let area = self.object.get_object().get_drawing_size(ctx);
        info.tile_position.unwrap().y - (info.player_position.unwrap().y + area.y) - 0.1
    }

    ///
    /// キャラクタテクスチャの右側が衝突した場合
    /// どれだけ、テクスチャを移動させれば良いのかを返す
    ///
    fn fix_collision_right(&mut self,
                            ctx: &mut ggez::Context,
                            info: &CollisionInformation,
                           _t: Clock) -> f32 {
        let area = self.object.get_object().get_drawing_size(ctx);
        (info.tile_position.unwrap().x - 0.1) - (info.player_position.unwrap().x + area.x)
    }

    ///
    /// キャラクタテクスチャの左側が衝突した場合
    /// どれだけ、テクスチャを移動させれば良いのかを返す
    ///
    fn fix_collision_left(&mut self,
                           _ctx: &mut ggez::Context,
                           info: &CollisionInformation,
                          _t: Clock) -> f32 {
        self.speed_info.set_speed_x(0.0);
        (info.tile_position.unwrap().x + info.tile_position.unwrap().w + 0.5) - info.player_position.unwrap().x
        
    }

    ///
    /// 垂直方向の衝突（めり込み）を修正するメソッド
    ///
    pub fn fix_collision_vertical(&mut self, ctx: &mut ggez::Context,
                         info: &CollisionInformation,
                                  t: Clock) -> f32 {
        self.speed_info.set_speed_x(0.0);
        if info.center_diff.unwrap().y < 0.0 {
            return self.fix_collision_bottom(ctx, &info, t);
        } else if info.center_diff.unwrap().y > 0.0 {
            return self.fix_collision_above(ctx, &info, t);
        }

        0.0
    }

    ///
    /// 水平方向の衝突（めり込み）を修正するメソッド
    ///
    pub fn fix_collision_horizon(&mut self, ctx: &mut ggez::Context,
                                 info: &CollisionInformation,
                                 t: Clock)  -> f32 {
        let right = info.player_position.unwrap().x + self.object.get_object().get_drawing_area(ctx).w;
        if right > info.tile_position.unwrap().x && right < info.tile_position.unwrap().x + info.tile_position.unwrap().w {
            return self.fix_collision_right(ctx, &info, t);
        } else {
            return self.fix_collision_left(ctx, &info, t);
        }
    }

    pub fn update_texture(&mut self, t: Clock) {
        self.object.try_next_frame(t);
    }
    
    pub fn apply_resistance(&mut self, t: Clock) {
        self.speed_info.apply_resistance(t);
    }
    
    pub fn move_map(&mut self, offset: numeric::Vector2f) {
        self.map_position.move_diff(&offset);
    }

    pub fn update_display_position(&mut self, camera: &numeric::Rect) {
        let dp = mp::map_to_display(&self.map_position.current, camera);
        self.object.get_mut_object().set_position(dp);
    }

}

pub struct PlayableCharacter {
    character: Character,
}

impl PlayableCharacter {
    pub fn new(character: Character) -> Self {
        PlayableCharacter {
            character: character,
        }
    }

    pub fn move_right(&mut self) {
        self.character.speed_info.set_speed_x(6.0);
    }

    pub fn move_left(&mut self) {
        self.character.speed_info.set_speed_x(-6.0);
    }

    pub fn jump(&mut self, t: Clock) {
        self.character.speed_info_mut().set_speed_y(-12.0);
        self.character.speed_info_mut().fall_start(t);
        self.character
            .obj_mut()
            .override_move_func(move_fn::gravity_move(-5.0, 24.0, 600.0, 0.2), t)
    }

    pub fn get_map_position(&self) -> numeric::Point2f {
        self.character.get_map_position()
    }

    pub fn get_character_object(&self) -> &Character {
        &self.character
    }

    pub fn get_mut_character_object(&mut self) -> &mut Character {
        &mut self.character
    }

    pub fn fix_collision_horizon(&mut self, ctx: &mut ggez::Context,
                                 info: &CollisionInformation,
                                 t: Clock)  -> f32 {
        self.character.fix_collision_horizon(ctx, info, t)
    }

    pub fn fix_collision_vertical(&mut self, ctx: &mut ggez::Context,
                                 info: &CollisionInformation,
                                 t: Clock)  -> f32 {
        self.character.fix_collision_vertical(ctx, info, t)
    }

    pub fn move_map(&mut self, offset: numeric::Vector2f) {
        self.character.move_map(offset);
    }
}

pub struct EnemyCharacter {
    character: Character,
}

impl EnemyCharacter {
    pub fn new(character: Character) -> Self {
        EnemyCharacter {
            character: character,
        }
    }
}
