use std::rc::Rc;
use std::collections::HashMap;
use std::str::FromStr;

use ggez::graphics as ggraphics;

use torifune::core::Clock;
use torifune::distance;
use torifune::numeric;
use torifune::graphics::*;
use torifune::graphics::object::*;

use crate::core::map_parser as mp;
use crate::object::collision::*;
use crate::scene::SceneID;
use crate::core::{BookShelfInformation};

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
    object: SimpleObject,
    animation_type: AnimationType,
    next_mode: usize,
    frame_speed: Clock,
}

impl TextureAnimation {
    pub fn new(obj: SimpleObject, textures: Vec<Vec<Rc<ggraphics::Image>>>, mode: usize, frame_speed: Clock) -> Self {
        TextureAnimation {
            textures: textures.iter().map(|vec| SeqTexture::new(vec.to_vec())).collect(),
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
    pub fn new(obj: SimpleObject, textures: Vec<Vec<Rc<ggraphics::Image>>>,
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

    pub fn obj(&self) -> &SimpleObject {
        self.object.get_object()
    }
    
    pub fn obj_mut(&mut self) -> &mut SimpleObject {
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
        (info.object1_position.unwrap().x + info.object1_position.unwrap().w + 0.5) - info.object2_position.unwrap().x
        
    }

    ///
    /// 垂直方向の衝突（めり込み）を修正するメソッド
    ///
    pub fn fix_collision_vertical(&mut self, ctx: &mut ggez::Context,
				  info: &CollisionInformation,
                                  t: Clock) -> f32 {
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

    pub fn move_map_current_speed_x(&mut self, border: f32) {
	let x_speed = self.get_character_object().speed_info().get_speed().x;
	let overflow = (self.get_map_position().x + x_speed) - border;
	if overflow > 0.0 {
	    self.move_map(numeric::Vector2f::new(x_speed - overflow, 0.0))
	} else {
	    self.move_map(numeric::Vector2f::new(x_speed, 0.0))
	}
    }

    pub fn move_map_current_speed_y(&mut self, border: f32) {
	let y_speed = self.get_character_object().speed_info().get_speed().y;
	let overflow = (self.get_map_position().y + y_speed) - border;
	if overflow > 0.0 {
	    self.move_map(numeric::Vector2f::new(0.0, y_speed - overflow))
	} else {
	    self.move_map(numeric::Vector2f::new(0.0, y_speed))
	}
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

pub struct GeneralCharacter {
    character: MapObject,
    collision_damage: DamageEffect,
}

impl GeneralCharacter {
    pub fn new(character: MapObject, collision_damage: DamageEffect) -> Self {
        GeneralCharacter {
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

#[derive(PartialEq, Clone, Copy)]
pub enum EventTrigger {
    Action,
    Touch,
}

impl FromStr for EventTrigger {
    type Err = ();
    
    fn from_str(trigger_str: &str) -> Result<Self, Self::Err> {
	match trigger_str {
	    "action" => Ok(Self::Action),
	    "touch" => Ok(Self::Touch),
	    _ => panic!("Error: EventTrigger::from_str"),
	}
    }
}

pub trait MapEvent {
    fn get_trigger_method(&self) -> EventTrigger;
}

pub struct MapTextEvent {
    trigger: EventTrigger,
    text: String,
}

impl MapTextEvent {
    pub fn from_toml_object(toml_script: &toml::value::Value) -> Self {
	MapTextEvent {
	    trigger: EventTrigger::from_str(toml_script.get("trigger").unwrap().as_str().unwrap()).unwrap(),
	    text: toml_script.get("text").unwrap().as_str().unwrap().to_string(),
	}
    }

    pub fn get_text(&self) -> &str {
	&self.text
    }
}

impl MapEvent for MapTextEvent {
    fn get_trigger_method(&self) -> EventTrigger {
	self.trigger
    }
}

pub struct MapEventSceneSwitch {
    trigger: EventTrigger,
    switch_scene: SceneID,
}

impl MapEventSceneSwitch {
    pub fn new(trigger: EventTrigger, switch_scene: SceneID) -> Self {
	MapEventSceneSwitch {
	    trigger: trigger,
	    switch_scene: switch_scene,
	}
    }

    pub fn from_toml_object(toml_script: &toml::value::Value) -> Self {
	MapEventSceneSwitch {
	    trigger: EventTrigger::from_str(toml_script.get("trigger").unwrap().as_str().unwrap()).unwrap(),
	    switch_scene: SceneID::from_str(toml_script.get("switch-scene-id").unwrap().as_str().unwrap()).unwrap(),
	}
    }

    pub fn get_switch_scene_id(&self) -> SceneID {
	self.switch_scene
    }
}

impl MapEvent for MapEventSceneSwitch {
    fn get_trigger_method(&self) -> EventTrigger {
	self.trigger
    }
}

pub struct BookStoreEvent {
    trigger: EventTrigger,
    book_shelf_info: BookShelfInformation,
}

impl BookStoreEvent {
    pub fn from_toml_object(toml_script: &toml::value::Value) -> Self {
	let shelf_info = toml_script.get("shelf-info").unwrap().as_table().unwrap();
	let book_shelf_info = BookShelfInformation::new(shelf_info.get("begin-number").unwrap().as_integer().unwrap() as u16,
							shelf_info.get("end-number").unwrap().as_integer().unwrap() as u16);
	
	BookStoreEvent {
	    trigger: EventTrigger::from_str(toml_script.get("trigger").unwrap().as_str().unwrap()).unwrap(),
	    book_shelf_info: book_shelf_info,
	}
    }

    pub fn get_book_shelf_info(&self) -> &BookShelfInformation {
	&self.book_shelf_info
    } 
}

impl MapEvent for BookStoreEvent {
    fn get_trigger_method(&self) -> EventTrigger {
	self.trigger
    }
}

pub enum MapEventElement {
    TextEvent(MapTextEvent),
    SwitchScene(MapEventSceneSwitch),
    BookStoreEvent(BookStoreEvent),
}

impl MapEvent for MapEventElement {
    fn get_trigger_method(&self) -> EventTrigger {
	match self {
            Self::TextEvent(text) => text.get_trigger_method(),
            Self::SwitchScene(switch_scene) => switch_scene.get_trigger_method(),
	    Self::BookStoreEvent(book_store_event) => book_store_event.get_trigger_method(),
	}
    }
}

pub struct MapEventList {
    event_table: HashMap<numeric::Point2i, MapEventElement>,
}

impl MapEventList {
    
    pub fn from_file(file_path: &str) -> Self {
	let mut table = HashMap::new();
	
	let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => panic!("Failed to read: {}", file_path),
        };
        
        let root = content.parse::<toml::Value>().unwrap();
        let array = root["event-panel"].as_array().unwrap();

	for elem in array {
	    let position_data = elem.get("position").unwrap().as_table().unwrap();
	    let position = numeric::Point2i::new(
		position_data.get("x").unwrap().as_integer().unwrap() as i32,
		position_data.get("y").unwrap().as_integer().unwrap() as i32
	    );
	    if let Some(type_info) = elem.get("type") {
		match type_info.as_str().unwrap() {
		    "text" => {
			table.insert(position, MapEventElement::TextEvent(
			    MapTextEvent::from_toml_object(elem)));
		    },
		    "switch-scene" => {
			table.insert(position, MapEventElement::SwitchScene(
			    MapEventSceneSwitch::from_toml_object(elem)));
		    },
		    "book-shelf" => {
			table.insert(position, MapEventElement::BookStoreEvent(
			    BookStoreEvent::from_toml_object(elem)));
		    },
		    _ => eprintln!("Error"),
		}
	    } else {
		eprintln!("Error");
	    }
	}
	
	MapEventList {
	    event_table: table,
	}
    }

    pub fn register_event(&mut self, point: numeric::Point2i, event: MapEventElement) -> &mut Self {
	self.event_table.insert(point, event);
	self
    }

    pub fn check_event(&self, trigger: EventTrigger, point: numeric::Point2i) -> Option<&MapEventElement> {
	if let Some(event_element) = self.event_table.get(&point) {
	    if event_element.get_trigger_method() == trigger {
		return Some(&event_element);
	    }
	}

	None
    }
}
