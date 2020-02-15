use std::rc::Rc;
use std::cell::RefCell;
use torifune::device as tdev;
use tdev::VirtualKey;
use torifune::core::Clock;
use torifune::graphics as tgraphics;
use torifune::core::Updatable;
use ggez::input as ginput;
use ginput::mouse::MouseButton;
use torifune::numeric;
use crate::core::GameData;
use super::*;
use crate::object::*;
use crate::object::map_object::*;
use crate::core::map_parser as mp;

struct CharacterGroup {
    group: Vec<GeneralCharacter>,
    drwob_essential: tgraphics::DrawableObjectEssential,
}

impl CharacterGroup {
    pub fn new() -> Self {
        CharacterGroup {
            group: Vec::new(),
            drwob_essential: tgraphics::DrawableObjectEssential::new(true, 0),
        }
    }

    #[inline(always)]
    pub fn add(&mut self, character: GeneralCharacter) {
        self.group.push(character);
    }

    #[inline(always)]
    pub fn remove_if<F>(&mut self, f: F)
    where F: Fn(&GeneralCharacter) -> bool {
        self.group.retain(|e| !f(e));
    }

    pub fn len(&self) -> usize {
        self.group.len()
    }

    pub fn move_and_collision_check(&mut self, ctx: &mut ggez::Context, camera: &numeric::Rect,
                           tile_map: &mp::StageObjectMap, t: Clock) {
        self.group
            .iter_mut()
            .for_each(|character| {
                character.move_map_current_speed_y();

                // 当たり判定の前に描画位置を決定しないとバグる。この仕様も直すべき
                character.get_mut_character_object().update_display_position(camera);
                
                DreamScene::check_collision_vertical(ctx, character.get_mut_character_object(), tile_map, t);
                character.get_mut_character_object().update_display_position(camera);
                

                character.move_map_current_speed_x();
                character.get_mut_character_object().update_display_position(camera);
                DreamScene::check_collision_horizon(ctx, character.get_mut_character_object(), tile_map, t);
                character.get_mut_character_object().update_display_position(camera);
            });
    }

    pub fn iter(&self) -> std::slice::Iter<GeneralCharacter> {
        self.group.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<GeneralCharacter> {
        self.group.iter_mut()
    }
}

impl DrawableComponent for CharacterGroup {
    #[inline(always)]
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        self.group.iter_mut().for_each(|e| { e.get_mut_character_object().obj_mut().draw(ctx).unwrap(); });
        Ok(())
    }

    #[inline(always)]
    fn hide(&mut self) {
        self.drwob_essential.visible = false;
    }

    #[inline(always)]
    fn appear(&mut self) {
        self.drwob_essential.visible = true;
    }

    #[inline(always)]
    fn is_visible(&self) -> bool {
        self.drwob_essential.visible
    }

    #[inline(always)]
    fn set_drawing_depth(&mut self, depth: i8) {
        self.drwob_essential.drawing_depth = depth;
    }

    #[inline(always)]
    fn get_drawing_depth(&self) -> i8 {
        self.drwob_essential.drawing_depth
    }
}

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
    player: PlayableCharacter,
    character_group: CharacterGroup,
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

        let map_position = numeric::Point2f::new(800.0, 190.0);
        
        let player = PlayableCharacter::new(
            character_factory::create_character(character_factory::CharacterFactoryOrder::PlayableDoremy1,
                                                game_data,
                                                &camera.borrow(),
                                                map_position),
        PlayerStatus { hp: 10, mp: 10.0 });
        
        let mut character_group = CharacterGroup::new();
        character_group.add(GeneralCharacter::new(
            character_factory::create_character(character_factory::CharacterFactoryOrder::PlayableDoremy1,
                                                game_data,
                                                &camera.borrow(),
                                                map_position), DamageEffect { hp_damage: 1, mp_damage: 1.0 }));
        
        DreamScene {
            player: player,
            character_group: character_group,
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

	if self.key_listener.current_key_status(ctx, &VirtualKey::Down) == tdev::KeyStatus::Pressed {
            self.down_key_handler(ctx);
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
	self.player.set_speed_x(4.0);
    }

    fn left_key_handler(&mut self, _ctx: &ggez::Context) {
	self.player.set_speed_x(-4.0);
    }

    fn up_key_handler(&mut self, _ctx: &ggez::Context) {
	self.player.set_speed_y(-4.0);
    }

    fn down_key_handler(&mut self, _ctx: &ggez::Context) {
	self.player.set_speed_y(4.0);
    }

    fn fix_camera_position(&self) -> numeric::Point2f {
        numeric::Point2f::new(if self.player.get_map_position().x >= 650.0 { 650.0 } else { self.player.get_map_position().x },
                              if self.player.get_map_position().y >= 400.0 { 400.0 } else { self.player.get_map_position().y })
    }

    ///
    /// マップオブジェクトとの衝突を調べるためのメソッド
    ///
    fn check_collision_horizon(ctx: &mut ggez::Context, character: &mut MapObject, tile_map: &mp::StageObjectMap, t: Clock) {
        let collision_info = tile_map.check_character_collision(ctx, character);

        // 衝突していたか？
        if  collision_info.collision {
            // 修正動作
            let diff = character.fix_collision_horizon(ctx, &collision_info, t);
            character.move_map(numeric::Vector2f::new(diff, 0.0));
        }
    }

    ///
    /// マップオブジェクトとの衝突を調べるためのメソッド
    ///
    fn check_collision_vertical(ctx: &mut ggez::Context, character: &mut MapObject, tile_map: &mp::StageObjectMap, t: Clock) {
        let collision_info = tile_map.check_character_collision(ctx, character);

        // 衝突していたか？
        if  collision_info.collision  {
            // 修正動作
            let diff = character.fix_collision_vertical(ctx, &collision_info, t);
            character.move_map(numeric::Vector2f::new(0.0, diff));
        }
    }

    ///
    /// PlayerCharacterのマップチップとのX方向の衝突を修正する
    ///
    fn playable_check_collision_horizon(&mut self, ctx: &mut ggez::Context) {
        
        let t = self.get_current_clock();

	// プレイヤーのマップチップに対する衝突を修正
        Self::check_collision_horizon(ctx, self.player.get_mut_character_object(), &self.tile_map, t);

	// プレイヤーのマップ座標を更新
        self.player.get_mut_character_object().update_display_position(&self.camera.borrow());

	// カメラをプレイヤーにフォーカス
	self.focus_camera_playable_character_x();
        
    }

    ///
    /// PlayerCharacterのマップチップとのY方向の衝突を修正する
    ///
    fn playable_check_collision_vertical(&mut self, ctx: &mut ggez::Context) {
        let t = self.get_current_clock();
        
	// プレイヤーのマップチップに対する衝突を修正
        Self::check_collision_vertical(ctx, self.player.get_mut_character_object(), &self.tile_map, t);

	// プレイヤーのマップ座標を更新
        self.player.get_mut_character_object().update_display_position(&self.camera.borrow());

	// カメラをプレイヤーにフォーカス
	self.focus_camera_playable_character_y();
    }

    fn focus_camera_playable_character_x(&mut self) {
	// カメラとプレイヤーキャラクターの差分を計算し、プレイヤーが中心に来るようにカメラを移動
        let player_diff = self.player.get_mut_character_object().obj().get_position() - self.fix_camera_position();
        self.move_camera(numeric::Vector2f::new(player_diff.x, 0.0));
    }

    fn focus_camera_playable_character_y(&mut self) {
	// カメラとプレイヤーキャラクターの差分を計算し、プレイヤーが中心に来るようにカメラを移動
        let player_diff = self.player.get_mut_character_object().obj().get_position() - self.fix_camera_position();
        self.move_camera(numeric::Vector2f::new(0.0, player_diff.y));
    }

    ///
    /// PlayerCharacterの他キャラクターとのX方向の衝突を修正する
    ///
    fn check_character_collision_x(&mut self, ctx: &mut ggez::Context, t: Clock) {
	// マップ座標を更新
        self.player.get_mut_character_object().update_display_position(&self.camera.borrow());

	// 他キャラクターすべてとの衝突判定を行う
        for e in self.character_group.iter_mut() {
	    // 他キャラクターのマップ座標を更新
            e.get_mut_character_object().update_display_position(&self.camera.borrow());

	    // 衝突情報を取得
            let collision_info = self.player.get_character_object().check_collision_with_character(ctx, e.get_character_object());

	    // collisionフィールドがtrueなら、衝突している
            if collision_info.collision {
		// プレイヤーと他キャラクターの衝突状況から、プレイヤーがどれだけ、突き放されればいいのか計算
                let diff = self.player.get_mut_character_object().fix_collision_horizon(ctx, &collision_info, t);
		
		// プレイヤーの突き放し距離分動かす
                self.player.get_mut_character_object().move_map(numeric::Vector2f::new(-diff, 0.0));

		// プレイヤーのマップ座標を更新
		self.player.get_mut_character_object().update_display_position(&self.camera.borrow());
            }
        }

	// カメラをプレイヤーに合わせる
        self.focus_camera_playable_character_x();
    }

    ///
    /// PlayerCharacterの他キャラクターとのY方向の衝突を修正する
    ///
    fn check_character_collision_y(&mut self, ctx: &mut ggez::Context, t: Clock) {
	// マップ座標を更新
        self.player.get_mut_character_object().update_display_position(&self.camera.borrow());

	// 他キャラクターすべてとの衝突判定を行う
        for e in self.character_group.iter_mut() {

	    // 他キャラクターのマップ座標を更新
            e.get_mut_character_object().update_display_position(&self.camera.borrow());

	    // 衝突情報を取得
            let collision_info = self.player.get_character_object().check_collision_with_character(ctx, e.get_character_object());

	    // collisionフィールドがtrueなら、衝突している
            if collision_info.collision {
		// プレイヤーと他キャラクターの衝突状況から、プレイヤーがどれだけ、突き放されればいいのか計算
                let diff = self.player.get_mut_character_object().fix_collision_vertical(ctx, &collision_info, t);

		// プレイヤーの突き放し距離分動かす
                self.player.get_mut_character_object().move_map(numeric::Vector2f::new(0.0, -diff));

		// プレイヤーのマップ座標を更新
		self.player.get_mut_character_object().update_display_position(&self.camera.borrow());
            }
        }

	// カメラをプレイヤーに合わせる
        self.focus_camera_playable_character_y();
    }

    fn move_playable_character_x(&mut self, ctx: &mut ggez::Context, t: Clock) {
	// プレイヤーのX方向の移動
        self.player.move_map_current_speed_x();
		// マップ座標を更新, これで、衝突判定を行えるようになる
        self.player.get_mut_character_object().update_display_position(&self.camera.borrow());

        // マップチップとの衝突判定（横）
        self.playable_check_collision_horizon(ctx);

	// 他キャラクターとの当たり判定
        self.check_character_collision_x(ctx, t);
    }

    fn move_playable_character_y(&mut self, ctx: &mut ggez::Context, t: Clock) {
        // プレイヤーのY方向の移動
        self.player.move_map_current_speed_y();
	// マップ座標を更新, これで、衝突判定を行えるようになる
        self.player.get_mut_character_object().update_display_position(&self.camera.borrow());

	// マップチップとの衝突判定（縦）
        self.playable_check_collision_vertical(ctx);

	// 他キャラクターとの当たり判定
        self.check_character_collision_y(ctx, t);
    }

    fn move_playable_character(&mut self, ctx: &mut ggez::Context, t: Clock) {
        // キーのチェック
        self.check_key_event(ctx);
        
        self.player.get_mut_character_object().update_texture(t);

        self.move_playable_character_x(ctx, t);
        self.move_playable_character_y(ctx, t);
    }
}

impl SceneManager for DreamScene {
    
    fn key_down_event(&mut self,
                      _ctx: &mut ggez::Context,
                      _game_data: &GameData,
                      _vkey: tdev::VirtualKey) {
    }
    
    fn key_up_event(&mut self,
                    _ctx: &mut ggez::Context,
                    _game_data: &GameData,
                    vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => println!("Action1 up!"),
            _ => (),
        }
    }

    fn mouse_motion_event(&mut self,
                          _ctx: &mut ggez::Context,
                          _game_data: &GameData,
                          _point: numeric::Point2f,
                          _offset: numeric::Vector2f) {

    }

    fn mouse_button_down_event(&mut self,
                               _ctx: &mut ggez::Context,
                               _game_data: &GameData,
                               _button: MouseButton,
                               _point: numeric::Point2f) {
    }
    
    fn mouse_button_up_event(&mut self,
                             _ctx: &mut ggez::Context,
                             _game_data: &GameData,
                             _button: MouseButton,
                             _point: numeric::Point2f) {
    }

    fn pre_process(&mut self, ctx: &mut ggez::Context, _: &GameData) {
        let t = self.get_current_clock();
	self.player.reset_speed();

        self.move_playable_character(ctx, t);
        
        self.character_group.move_and_collision_check(ctx, &self.camera.borrow(), &self.tile_map, t);
        
        // マップ描画の準備
        self.tile_map.update(ctx, t);
    }
    
    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.tile_map.draw(ctx).unwrap();
        self.player
            .get_mut_character_object()
            .obj_mut()
            .draw(ctx).unwrap();
        self.character_group.draw(ctx).unwrap();
    }
    
    fn post_process(&mut self, _ctx: &mut ggez::Context, _: &GameData) -> SceneTransition {
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
