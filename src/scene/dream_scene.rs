use std::rc::Rc;
use std::cell::RefCell;
use torifune::device as tdev;
use tdev::VirtualKey;
use torifune::core::Clock;
use torifune::graphics as tgraphics;
use torifune::graphics::object::*;
use torifune::core::Updatable;
use tgraphics::object as tobj;
use ggez::input as ginput;
use ginput::mouse::MouseButton;
use torifune::numeric;
use crate::core::{TextureID, GameData};
use super::*;
use crate::object;
use crate::core::map_parser as mp;

///
/// # 夢の中のステージ
///
/// ## フィールド
/// ### player
/// プレイキャラ
///
/// ### key_listener
/// キー監視用
///
/// ### clock
/// 基準クロック
///
/// ### tile_map
/// マップ情報
///
/// ### camera
/// マップを覗くカメラ
///
pub struct DreamScene {
    player: object::PlayableCharacter,
    key_listener: tdev::KeyboardListener,
    clock: Clock,
    tile_map: mp::StageObjectMap,
    camera: Rc<RefCell<numeric::Rect>>,
}

impl DreamScene {
    
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData) -> DreamScene  {

        let key_listener = tdev::KeyboardListener::new_masked(vec![tdev::KeyInputDevice::GenericKeyboard],
                                                                  vec![]);

        let camera = Rc::new(RefCell::new(numeric::Rect::new(0.0, 0.0, 1366.0, 768.0)));

        let map_position = numeric::Point2f::new(60.0, 100.0);
        
        let player = object::PlayableCharacter::new(
            object::Character::new(tobj::SimpleObject::new(
            tobj::MovableUniTexture::new(
                game_data.ref_texture(TextureID::Ghost1),
                mp::map_to_display(&map_position, &camera.borrow()),
                numeric::Vector2f::new(0.1, 0.1),
                0.0, 0, object::move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                0), vec![]),
                                            vec![vec![
                                                game_data.ref_texture(TextureID::LotusPink),
                                                game_data.ref_texture(TextureID::LotusBlue)
                                            ]], 0,
                                            object::TextureSpeedInfo::new(0.05, 0.08, numeric::Vector2f::new(0.0, 0.0),
                                                                          object::SpeedBorder {
                                                                              positive_x: 6.0,
                                                                              negative_x: -6.0,
                                                                              positive_y: 6.0,
                                                                              negative_y: -6.0,
                                                                          }), map_position,
                                            5));
        
        DreamScene {
            player: player,
            key_listener: key_listener,
            clock: 0,
            tile_map: mp::StageObjectMap::new(ctx, "./resources/map1.tmx", camera.clone(), numeric::Vector2f::new(2.0, 2.0)),
            camera: camera,
        }
    }

    ///
    /// キー入力のイベントハンドラ
    ///
    fn check_key_event(&mut self, ctx: &ggez::Context) {
        if self.key_listener.current_key_status(ctx, &VirtualKey::Right) == tdev::KeyStatus::Pressed {
            self.right_key_handler(ctx);
        }

        if self.key_listener.current_key_status(ctx, &VirtualKey::Left) == tdev::KeyStatus::Pressed {
            self.left_key_handler(ctx);
        }

        if self.key_listener.current_key_status(ctx, &VirtualKey::Up) == tdev::KeyStatus::Pressed {
            self.up_key_handler(ctx);
        }
    }

    ///
    /// カメラを動かすメソッド
    ///
    pub fn move_camera(&mut self, offset: numeric::Vector2f) {
        self.camera.borrow_mut().x += offset.x;
        self.camera.borrow_mut().y += offset.y;
    }

    pub fn set_camera_x(&mut self, offset: f32) {
        self.camera.borrow_mut().x = offset;
    }

    pub fn set_camera_y(&mut self, offset: f32) {
        self.camera.borrow_mut().y = offset;
    }

    pub fn set_camera(&mut self, offset: numeric::Vector2f) {
        self.camera.borrow_mut().x = offset.x;
        self.camera.borrow_mut().y = offset.y;
    }
    
    fn right_key_handler(&mut self, _ctx: &ggez::Context) {
        self.player.move_right();
    }

    fn left_key_handler(&mut self, _ctx: &ggez::Context) {
        self.player.move_left();
    }

    fn up_key_handler(&mut self, _ctx: &ggez::Context) {
        let t = self.get_current_clock();
        self.player.jump(t);
    }

    fn fix_camera_position(&self) -> numeric::Point2f {
        numeric::Point2f::new(if self.player.get_map_position().x >= 650.0 { 650.0 } else { self.player.get_map_position().x },
                              if self.player.get_map_position().y >= 400.0 { 400.0 } else { self.player.get_map_position().y })
    }

    ///
    /// マップオブジェクトとの衝突を調べるためのメソッド
    ///
    fn check_collision_horizon(&mut self, ctx: &mut ggez::Context) {
        let collision_info = self.tile_map.check_character_collision(ctx, self.player.get_character_object());

        // 衝突していたか？
        if  collision_info.collision  {
            // 修正動作
            let diff = self.player.fix_collision_horizon(ctx, &collision_info, self.get_current_clock());
            self.player.move_map(numeric::Vector2f::new(diff, 0.0));
        }
    }

    ///
    /// マップオブジェクトとの衝突を調べるためのメソッド
    ///
    fn check_collision_vertical(&mut self, ctx: &mut ggez::Context) {
        let collision_info = self.tile_map.check_character_collision(ctx, self.player.get_character_object());

        // 衝突していたか？
        if  collision_info.collision  {
            // 修正動作
            let diff = self.player.fix_collision_vertical(ctx, &collision_info, self.get_current_clock());
            self.player.move_map(numeric::Vector2f::new(0.0, diff));
        }
    }

    fn playable_check_collision_horizon(&mut self, ctx: &mut ggez::Context) {
        
        self.player.move_map(numeric::Vector2f::new(self.player.get_character_object().speed_info().get_speed().x, 0.0));
        self.player.get_mut_character_object().update_display_position(&self.camera.borrow());
        
        // 衝突の検出 + 修正動作
        self.check_collision_horizon(ctx);
        self.player.get_mut_character_object().update_display_position(&self.camera.borrow());
        let a = self.player.get_mut_character_object().obj().get_position() - self.fix_camera_position();
        self.move_camera(numeric::Vector2f::new(a.x, 0.0));
        
    }

    fn playable_check_collision_vertical(&mut self, ctx: &mut ggez::Context) {
        // プレイヤーに重力の影響を受けさせる
        self.player.move_map(numeric::Vector2f::new(0.0, self.player.get_character_object().speed_info().get_speed().y));
        self.player.get_mut_character_object().update_display_position(&self.camera.borrow());
        // 衝突の検出 + 修正動作
        self.check_collision_vertical(ctx);
        self.player.get_mut_character_object().update_display_position(&self.camera.borrow());

        let a = self.player.get_mut_character_object().obj().get_position() - self.fix_camera_position();
        self.move_camera(numeric::Vector2f::new(0.0, a.y));
    }

    fn move_playable_character(&mut self, ctx: &mut ggez::Context, t: Clock) {
        // キーのチェック
        self.check_key_event(ctx);
        
        self.player.get_mut_character_object().update_texture(t);
        self.player.get_mut_character_object().apply_resistance(t);

        self.playable_check_collision_horizon(ctx);
        self.playable_check_collision_vertical(ctx);
    }
}

impl SceneManager for DreamScene {
    
    fn key_down_event(&mut self, _ctx: &mut ggez::Context, _vkey: tdev::VirtualKey) {
    }
    
    fn key_up_event(&mut self,
                    _ctx: &mut ggez::Context,
                    vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => println!("Action1 up!"),
            _ => (),
        }
    }

    fn mouse_motion_event(&mut self,
                          _ctx: &mut ggez::Context,
                          _point: numeric::Point2f,
                          _offset: numeric::Vector2f) {

    }

    fn mouse_button_down_event(&mut self,
                               _ctx: &mut ggez::Context,
                               _button: MouseButton,
                               _point: numeric::Point2f) {
    }
    
    fn mouse_button_up_event(&mut self,
                             _ctx: &mut ggez::Context,
                             _button: MouseButton,
                             _point: numeric::Point2f) {
    }

    fn pre_process(&mut self, ctx: &mut ggez::Context) {
        let t = self.get_current_clock();

        self.move_playable_character(ctx, t);
        
        // マップ描画の準備
        self.tile_map.update(ctx, t);
    }
    
    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.tile_map.draw(ctx).unwrap();
        self.player
            .get_character_object()
            .obj()
            .draw(ctx).unwrap();
    }
    
    fn post_process(&mut self, _ctx: &mut ggez::Context) -> SceneTransition {
        self.update_current_clock();
        SceneTransition::Keep
    }

    fn transition(&self) -> SceneID {
        SceneID::Dream
    }

    fn get_current_clock(&self) -> Clock {
        self.clock
    }
    
    fn update_current_clock(&mut self) {
        self.clock += 1;
    }
    
}
