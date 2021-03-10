use ggez::input::mouse::MouseButton;

use torifune::core::*;
use torifune::device::*;
use torifune::graphics::drawable::*;
use torifune::graphics::object::*;
use torifune::sound::*;

use crate::core::{
    GameMode, MouseInformation, SoundID, SuzuContext, TextureID, TileBatchTextureID,
};
use crate::object::effect_object;
use crate::object::title_object::*;
use crate::scene::*;

use crate::flush_delay_event;

pub struct TitleScene {
    mouse_info: MouseInformation,
    background: UniTexture,
    logo: UniTexture,
    event_list: DelayEventList<Self>,
    scene_transition_effect: Option<effect_object::ScreenTileEffect>,
    scene_transition: SceneID,
    scene_transition_type: SceneTransition,
    current_title_contents: Option<TitleContents>,
    title_contents_set: TitleContentsSet,
    scene_transition_lock: bool,
    clock: Clock,
}

impl TitleScene {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>) -> Self {
        let background = UniTexture::new(
            ctx.ref_texture(TextureID::JpHouseTexture),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(0.7, 0.7),
            0.0,
            0,
        );

        let logo = UniTexture::new(
            ctx.ref_texture(TextureID::SuzuLogo),
            numeric::Point2f::new(950.0, 50.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
        );

        let scene_transition_effect = Some(effect_object::ScreenTileEffect::new(
            ctx,
            TileBatchTextureID::Shoji,
            numeric::Rect::new(
                0.0,
                0.0,
                crate::core::WINDOW_SIZE_X as f32,
                crate::core::WINDOW_SIZE_Y as f32,
            ),
            30,
            effect_object::SceneTransitionEffectType::Open,
            effect_object::TilingEffectType::WholeTile,
            -128,
            0,
        ));

        let mut event_list = DelayEventList::new();
        event_list.add_event(
            Box::new(move |slf: &mut Self, _, _| {
                slf.scene_transition_effect = None;
            }),
            31,
        );

        let mut title_contents_set = TitleContentsSet::from_file(
            ctx,
            "./resources/title_contents/title_contents_list.toml",
            0,
        );

        ctx.play_sound_as_bgm(
            SoundID::Title,
            Some(SoundPlayFlags::new(10000, 1.0, true, 0.1)),
        );

        TitleScene {
            mouse_info: MouseInformation::new(),
            background: background,
            event_list: event_list,
            scene_transition_effect: scene_transition_effect,
            scene_transition: SceneID::Title,
            scene_transition_type: SceneTransition::Keep,
            current_title_contents: title_contents_set.remove_pickup("init-menu"),
            title_contents_set: title_contents_set,
            logo: logo,
            scene_transition_lock: false,
            clock: 0,
        }
    }

    fn lock_scene_transition(&mut self) {
        self.scene_transition_lock = true;
    }

    fn unlock_scene_transition(&mut self) {
        self.scene_transition_lock = false;
    }

    fn is_scene_transition_locked(&self) -> bool {
        self.scene_transition_lock
    }

    pub fn transition_selected_scene<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        scene_id: SceneID,
        trans: SceneTransition,
        game_mode: Option<GameMode>,
        t: Clock,
    ) {
        if self.is_scene_transition_locked() {
            return;
        }

        self.lock_scene_transition();

        self.scene_transition_effect = Some(effect_object::ScreenTileEffect::new(
            ctx,
            TileBatchTextureID::Shoji,
            numeric::Rect::new(
                0.0,
                0.0,
                crate::core::WINDOW_SIZE_X as f32,
                crate::core::WINDOW_SIZE_Y as f32,
            ),
            30,
            effect_object::SceneTransitionEffectType::Close,
            effect_object::TilingEffectType::WholeTile,
            -128,
            t,
        ));

        // 新規開始ならセーブデータを初期化
        if scene_id == SceneID::Scenario {
            ctx.reset_save_data(if let Some(game_mode) = game_mode {
                game_mode
            } else {
                GameMode::story()
            });
        }

        self.event_list.add_event(
            Box::new(move |slf: &mut Self, ctx, _| {
                slf.scene_transition = scene_id;
                slf.scene_transition_type = trans;
                slf.unlock_scene_transition();
                ctx.resource.stop_bgm(ctx.context, SoundID::Title);
            }),
            t + 31,
        );
    }

    pub fn contents_mouse_motion_handler<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        offset: numeric::Vector2f,
    ) {
        if self.current_title_contents.is_none() {
            return;
        }

        match &mut self.current_title_contents.as_mut().unwrap() {
            TitleContents::InitialMenu(contents) => {
                contents.update_highlight(ctx, point);
            }
            TitleContents::TitleSoundPlayer(contents) => {
                println!("sound-player");
                contents.dragging_handler(ctx, point, offset);
            }
            TitleContents::ConfigPanel(_) => (),
            TitleContents::UpdatePanel(_) => (),
            TitleContents::Gallery(_) => (),
            TitleContents::RecordRoom(_) => (),
        }
    }

    pub fn contents_mouse_dragging_handler<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        _offset: numeric::Vector2f,
    ) {
        if self.current_title_contents.is_none() {
            return;
        }

        let t = self.get_current_clock();

        match &mut self.current_title_contents.as_mut().unwrap() {
            TitleContents::ConfigPanel(panel) => {
                panel.mouse_dragging_handler(ctx, MouseButton::Left, point, t);
            }
            _ => (),
        }
    }

    fn switch_current_content<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        content_name: String,
        t: Clock,
    ) {
        let mut next_content = self.title_contents_set.remove_pickup(&content_name);
        if next_content.is_none() {
            print!("{}?", content_name);
            panic!("target title contents not found.");
        }

        // Galleryはクリア後特典なので、クリアしていなければ
        // listに戻して切り替え中断
        match next_content.as_ref().unwrap() {
            TitleContents::Gallery(_) => {
                if !ctx.permanent_save_data.is_cleared() {
                    self.title_contents_set.add(
                        next_content.as_ref().unwrap().get_content_name(),
                        next_content.unwrap(),
                    );
                    return;
                }
            }
            _ => (),
        }

        next_content.as_mut().unwrap().notify_switched(ctx, t);

        // contentの切り替え
        let old = std::mem::replace(&mut self.current_title_contents, next_content);
        self.title_contents_set
            .add(old.as_ref().unwrap().get_content_name(), old.unwrap());
    }

    fn run_builtin_command(&mut self, command: TitleBuiltinCommand) {
        match command {
            TitleBuiltinCommand::Exit => std::process::exit(0),
        }
    }

    pub fn contents_mouse_click_handler<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        t: Clock,
    ) {
        if self.current_title_contents.is_none() {
            return;
        }

        ctx.process_utility.redraw();

        match &mut self.current_title_contents.as_mut().unwrap() {
            TitleContents::InitialMenu(contents) => {
                let maybe_event = contents.click_handler(ctx, point);
                if let Some(event) = maybe_event {
                    match event {
                        TitleContentsEvent::NextContents(content_name) => {
                            self.switch_current_content(ctx, content_name, t);
                        }
                        TitleContentsEvent::SceneTransition((scene_id, trans, game_mode)) => {
                            self.transition_selected_scene(ctx, scene_id, trans, game_mode, t);
                        }
                        TitleContentsEvent::BuiltinEvent(command) => {
                            self.run_builtin_command(command);
                        }
                    }
                }
            }
            TitleContents::TitleSoundPlayer(contents) => {
                contents.mouse_button_up_handler();
            }
            TitleContents::ConfigPanel(panel) => {
                let maybe_event = panel.mouse_button_up(ctx, point, t);
                if let Some(event) = maybe_event {
                    match event {
                        TitleContentsEvent::NextContents(content_name) => {
                            self.switch_current_content(ctx, content_name, t);
                        }
                        _ => (),
                    }
                }
            }
            TitleContents::UpdatePanel(panel) => {
                let maybe_event = panel.mouse_button_up(ctx, point, t);
                if let Some(event) = maybe_event {
                    match event {
                        TitleContentsEvent::NextContents(content_name) => {
                            self.switch_current_content(ctx, content_name, t);
                        }
                        _ => (),
                    }
                }
            }
            TitleContents::Gallery(gallery) => {
                let maybe_event = gallery.mouse_button_up(ctx, point, t);
                if let Some(event) = maybe_event {
                    match event {
                        TitleContentsEvent::NextContents(content_name) => {
                            self.switch_current_content(ctx, content_name, t);
                        }
                        _ => (),
                    }
                }
            }
            TitleContents::RecordRoom(rr) => {
                let maybe_event = rr.mouse_button_up(ctx, point, t);
                if let Some(event) = maybe_event {
                    match event {
                        TitleContentsEvent::NextContents(content_name) => {
                            self.switch_current_content(ctx, content_name, t);
                        }
                        _ => (),
                    }
                }
            }
        }
    }
}

impl SceneManager for TitleScene {
    fn key_up_event<'a>(&mut self, _ctx: &mut SuzuContext<'a>, vkey: tdev::VirtualKey) {
        match vkey {
            VirtualKey::Action2 => {
                if let Some(content) = self.current_title_contents.as_mut() {
                    match content {
                        TitleContents::TitleSoundPlayer(_) => {}
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }

    fn pre_process<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        let t = self.get_current_clock();

        if let Some(transition_effect) = self.scene_transition_effect.as_mut() {
            transition_effect.effect(ctx.context, t);
            ctx.process_utility.redraw();
        }

        if let Some(content) = self.current_title_contents.as_mut() {
            match content {
                TitleContents::TitleSoundPlayer(player) => {
                    player.move_with_func(t);
                    ctx.process_utility.redraw();
                }
                _ => (),
            }
        }

        if flush_delay_event!(self, self.event_list, ctx, self.get_current_clock()) > 0 {
            ctx.process_utility.redraw();
        }

        if let Some(contents) = self.current_title_contents.as_mut() {
            contents.update(ctx, t);
        }
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.background.draw(ctx).unwrap();
        self.logo.draw(ctx).unwrap();

        if let Some(contetns) = self.current_title_contents.as_mut() {
            contetns.draw(ctx).unwrap();
        }

        if let Some(transition_effect) = self.scene_transition_effect.as_mut() {
            transition_effect.draw(ctx).unwrap();
        }
    }

    fn post_process<'a>(&mut self, _ctx: &mut SuzuContext<'a>) -> SceneTransition {
        self.update_current_clock();

        self.scene_transition_type
    }

    fn mouse_button_down_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: ginput::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        let t = self.get_current_clock();

        self.mouse_info.set_last_clicked(button, point, t);
        self.mouse_info.set_last_down(button, point, t);
        self.mouse_info.set_last_dragged(button, point, t);
        self.mouse_info.update_dragging(button, true);

        match self.current_title_contents.as_mut().unwrap() {
            TitleContents::TitleSoundPlayer(contents) => {
                contents.mouse_button_down_handler(ctx, point)
            }
            TitleContents::ConfigPanel(panel) => panel.mouse_button_down(ctx, button, point, t),
            _ => (),
        }
    }

    fn mouse_button_up_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: ginput::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        let t = self.get_current_clock();

        self.mouse_info.update_dragging(button, false);

        self.contents_mouse_click_handler(ctx, point, t);
    }

    fn mouse_motion_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        offset: numeric::Vector2f,
    ) {
        if self.mouse_info.is_dragging(ggez::event::MouseButton::Left) {
            self.contents_mouse_dragging_handler(ctx, point, offset);
        } else {
            self.contents_mouse_motion_handler(ctx, point, offset);
        }
    }
    fn scene_popping_return_handler<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        self.scene_transition_type = SceneTransition::Keep;
        self.scene_transition = SceneID::Title;

        self.scene_transition_effect = Some(effect_object::ScreenTileEffect::new(
            ctx,
            TileBatchTextureID::Shoji,
            numeric::Rect::new(
                0.0,
                0.0,
                crate::core::WINDOW_SIZE_X as f32,
                crate::core::WINDOW_SIZE_Y as f32,
            ),
            30,
            effect_object::SceneTransitionEffectType::Open,
            effect_object::TilingEffectType::WholeTile,
            -128,
            self.get_current_clock(),
        ));
    }

    fn transition(&self) -> SceneID {
        self.scene_transition
    }

    fn get_current_clock(&self) -> Clock {
        self.clock
    }

    fn update_current_clock(&mut self) {
        self.clock += 1;
    }
}
