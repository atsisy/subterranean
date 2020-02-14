pub mod move_fn;
pub mod effect;
pub mod collision;
pub mod character_factory;
pub mod scenario;
pub mod simulation_ui;
pub mod task_object;
pub mod task_result_object;

use std::rc::Rc;
use std::cmp::Ordering;

use ggez::graphics as ggraphics;

use torifune::core::Clock;
use torifune::numeric;
use torifune::distance;
use torifune::graphics::object as tobj;
use torifune::graphics::object::*;
use torifune::graphics::*;
use torifune::impl_texture_object_for_wrapped;
use torifune::impl_drawable_object_for_wrapped;

use crate::object::collision::*;
use crate::core::map_parser as mp;
use crate::core::{TextureID, GameData};

pub trait Clickable : TextureObject {
    fn button_down(&mut self,
                   _ctx: &mut ggez::Context,
		   _: &GameData,
		   _: Clock,
                   _button: ggez::input::mouse::MouseButton,
                   _point: numeric::Point2f) {}
    
    fn button_up(&mut self,
                 _ctx: &mut ggez::Context,
		 _: &GameData,
		 _: Clock,
                 _button: ggez::input::mouse::MouseButton,
                 _point: numeric::Point2f) {}

    fn on_click(&mut self,
                _ctx: &mut ggez::Context,
		_: &GameData,
		_: Clock,
                _button: ggez::input::mouse::MouseButton,
                _point: numeric::Point2f) {}

    fn clickable_status(&mut self,
                _ctx: &mut ggez::Context,
                    _point: numeric::Point2f) -> ggez::input::mouse::MouseCursor {
	ggez::input::mouse::MouseCursor::Default
    }
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
    pub fn new(speed: numeric::Vector2f, border: SpeedBorder)
               -> TextureSpeedInfo {
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

pub struct MapObject {
    last_position: numeric::Point2f,
    object: TextureAnimation,
    speed_info: TextureSpeedInfo,
    map_position: TwoStepPoint,
}

impl MapObject {
    pub fn new(obj: tobj::SimpleObject, textures: Vec<Vec<Rc<ggraphics::Image>>>,
               mode: usize, speed_info: TextureSpeedInfo, map_position: numeric::Point2f,
               frame_speed: Clock) -> MapObject {
        MapObject {
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
                           _: Clock) -> f32 {
        (info.object1_position.unwrap().y + info.object1_position.unwrap().h + 0.1) - info.object2_position.unwrap().y
    }

    ///
    /// キャラクタテクスチャの下側が衝突した場合
    /// どれだけ、テクスチャを移動させれば良いのかを返す
    ///
    fn fix_collision_bottom(&mut self,
                            ctx: &mut ggez::Context,
                            info: &CollisionInformation,
                            _: Clock) -> f32 {
        let area = self.object.get_object().get_drawing_size(ctx);
        info.object1_position.unwrap().y - (info.object2_position.unwrap().y + area.y) - 1.0
    }

    ///
    /// キャラクタテクスチャの右側が衝突した場合
    /// どれだけ、テクスチャを移動させれば良いのかを返す
    ///
    fn fix_collision_right(&mut self,
                           ctx: &mut ggez::Context,
                           info: &CollisionInformation,
                           _: Clock) -> f32 {
        let area = self.object.get_object().get_drawing_size(ctx);
        (info.object1_position.unwrap().x - 2.0) - (info.object2_position.unwrap().x + area.x)
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
        (info.object1_position.unwrap().x + info.object1_position.unwrap().w + 0.5) - info.object2_position.unwrap().x
        
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
	self.speed_info.set_speed_x(0.0);
        if info.center_diff.unwrap().x < 0.0 {
            return self.fix_collision_right(ctx, &info, t);
        } else if info.center_diff.unwrap().x > 0.0 {
            return self.fix_collision_left(ctx, &info, t);
        }

        0.0
    }

    pub fn update_texture(&mut self, t: Clock) {
        self.object.try_next_frame(t);
    }
    
    pub fn move_map(&mut self, offset: numeric::Vector2f) {
        self.map_position.move_diff(&offset);
    }

    pub fn update_display_position(&mut self, camera: &numeric::Rect) {
        let dp = mp::map_to_display(&self.map_position.current, camera);
        self.object.get_mut_object().set_position(dp);
    }

    pub fn check_collision_with_character(&self, ctx: &mut ggez::Context, chara: &MapObject) -> CollisionInformation {
        let a1 = self.obj().get_drawing_area(ctx);
        let a2 = chara.obj().get_drawing_area(ctx);

        if a1.overlaps(&a2) {
            CollisionInformation::new_collision(a1, a2,
                                                numeric::Vector2f::new(a2.x - a1.x, a2.y - a1.y))
        } else {
            CollisionInformation::new_not_collision()
        }
    }
    
}

pub struct DamageEffect {
    pub hp_damage: i16,
    pub mp_damage: f32,
}

pub struct AttackCore {
    center_position: numeric::Point2f,
    radius: f32,
}

impl AttackCore {
    pub fn new(center: numeric::Point2f, radius: f32) -> Self {
        AttackCore {
            center_position: center,
            radius: radius,
        }
    }
    
    pub fn distance(&self, obj: &AttackCore) -> f32 {
        distance!(self.center_position, obj.center_position)
    }

    pub fn check_collision(&self, obj: &AttackCore) -> bool {
        let d = self.distance(obj);
        d < (self.radius + obj.radius)
    }

    pub fn move_diff(&mut self, offset: numeric::Vector2f) {
        self.center_position += offset;
    }
}

pub struct PlayerStatus {
    pub hp: i16,
    pub mp: f32,
}

pub struct PlayableCharacter {
    character: MapObject,
    status: PlayerStatus,
}

impl PlayableCharacter {
    pub fn new(character: MapObject, status: PlayerStatus) -> Self {
        PlayableCharacter {
            character: character,
            status,
        }
    }

    pub fn get_map_position(&self) -> numeric::Point2f {
        self.character.get_map_position()
    }

    pub fn get_character_object(&self) -> &MapObject {
        &self.character
    }

    pub fn get_mut_character_object(&mut self) -> &mut MapObject {
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

    pub fn move_map_current_speed_x(&mut self) {
        self.move_map(numeric::Vector2f::new(self.get_character_object().speed_info().get_speed().x, 0.0))
    }

    pub fn move_map_current_speed_y(&mut self) {
        self.move_map(numeric::Vector2f::new(0.0, self.get_character_object().speed_info().get_speed().y))
    }

    pub fn attack_damage_check(&mut self, ctx: &mut ggez::Context, attack_core: &AttackCore, damage: &DamageEffect) {
        let center = self.get_map_position() + self.character.obj().get_center_offset(ctx);
        if distance!(center, attack_core.center_position) < attack_core.radius {
            self.status.hp -= damage.hp_damage;
            self.status.mp -= damage.mp_damage;
        }
    }

    pub fn set_speed(&mut self, speed: numeric::Vector2f) {
	self.character.speed_info_mut().set_speed(speed);
    }

    pub fn set_speed_x(&mut self, speed: f32) {
	self.character.speed_info_mut().set_speed_x(speed);
    }

    pub fn set_speed_y(&mut self, speed: f32) {
	self.character.speed_info_mut().set_speed_y(speed);
    }

    pub fn reset_speed(&mut self) {
	self.character.speed_info_mut().set_speed(numeric::Vector2f::new(0.0, 0.0));
    }
}

pub struct EnemyCharacter {
    character: MapObject,
    collision_damage: DamageEffect,
}

impl EnemyCharacter {
    pub fn new(character: MapObject, collision_damage: DamageEffect) -> Self {
        EnemyCharacter {
            character: character,
            collision_damage: collision_damage,
        }
    }

    pub fn get_map_position(&self) -> numeric::Point2f {
        self.character.get_map_position()
    }

    pub fn get_character_object(&self) -> &MapObject {
        &self.character
    }

    pub fn get_mut_character_object(&mut self) -> &mut MapObject {
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

    pub fn move_map_current_speed_x(&mut self) {
        self.move_map(numeric::Vector2f::new(self.get_character_object().speed_info().get_speed().x, 0.0))
    }

    pub fn move_map_current_speed_y(&mut self) {
        self.move_map(numeric::Vector2f::new(0.0, self.get_character_object().speed_info().get_speed().y))
    }

    pub fn get_collision_damage(&self) -> &DamageEffect {
        &self.collision_damage
    }

    pub fn get_attack_core(&self, ctx: &mut ggez::Context) -> AttackCore {
        AttackCore::new(self.character.get_map_position() + self.character.obj().get_center_offset(ctx), 10.0)
    }
}

pub struct BlackOutParam {
    pub black_out: Clock,
    pub black_keep: Clock,
    pub black_return: Clock,
}

impl BlackOutParam {
    pub fn new(black_out: Clock, black_keep: Clock, black_return: Clock) -> Self {
	BlackOutParam {
	    black_out: black_out,
	    black_keep: black_keep,
	    black_return: black_return,
	}
    }
}

pub struct BlackOutTexture {
    texture: EffectableWrap<MovableWrap<UniTexture>>,
}

impl BlackOutTexture {
    pub fn new(game_data: &mut GameData,
	       texture_id: TextureID,
	       pos: numeric::Point2f,
	       drawing_depth: i8,
	       now: Clock) -> Self {
	BlackOutTexture {
	    texture:
	    EffectableWrap::new(
		MovableWrap::new(
		    Box::new(
			UniTexture::new(game_data.ref_texture(texture_id),
					pos,
					numeric::Vector2f::new(1.0, 1.0),
					0.0,
					drawing_depth)), None, now),
		vec![]),
	}
    }

    pub fn run_black_out(&mut self, param: BlackOutParam, now: Clock) {
	self.texture.clear_effect();
	self.texture.add_effect(vec![
	    effect::fade_in(param.black_out, now),
	    effect::fade_out(param.black_return, now + param.black_out + param.black_keep)
	]);
    }
}

impl DrawableComponent for BlackOutTexture {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
	self.texture.draw(ctx)
    }

    #[inline(always)]
    fn hide(&mut self) {
	self.texture.hide()
    }

    #[inline(always)]
    fn appear(&mut self) {
	self.texture.appear()
    }

    #[inline(always)]
    fn is_visible(&self) -> bool {
	self.texture.is_visible()
    }

    #[inline(always)]
    fn set_drawing_depth(&mut self, depth: i8) {
	self.texture.set_drawing_depth(depth)
    }

    #[inline(always)]
    fn get_drawing_depth(&self) -> i8 {
	self.texture.get_drawing_depth()
    }
}

impl DrawableObject for BlackOutTexture {
    impl_drawable_object_for_wrapped!{texture}
}

impl TextureObject for BlackOutTexture {
    impl_texture_object_for_wrapped!{texture}
}
