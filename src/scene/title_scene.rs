use torifune::core::Clock;
use torifune::device::*;
use torifune::graphics::drawable::*;
use torifune::graphics::object::*;

use crate::core::{SuzuContext, TextureID, TileBatchTextureID};
use crate::object::effect_object;
use crate::object::title_object::*;
use crate::scene::*;

use crate::flush_delay_event;

pub struct TitleScene {
    background: UniTexture,
    event_list: DelayEventList<Self>,
    scene_transition_effect: Option<effect_object::ScreenTileEffect>,
    scene_transition: SceneID,
    scene_transition_type: SceneTransition,
    current_title_contents: Option<TitleContents>,
    title_contents_set: TitleContentsSet,
    clock: Clock,
}

impl TitleScene {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>) -> Self {
        let background = UniTexture::new(
            ctx.resource.ref_texture(TextureID::JpHouseTexture),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(0.7, 0.7),
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

        let mut title_contents_set =
            TitleContentsSet::from_file(ctx, "./resources/title_contents/title_contents_list.toml");

        TitleScene {
            background: background,
            event_list: event_list,
            scene_transition_effect: scene_transition_effect,
            scene_transition: SceneID::Save,
            scene_transition_type: SceneTransition::Keep,
            current_title_contents: title_contents_set.remove_pickup("init-menu"),
            title_contents_set: title_contents_set,
            clock: 0,
        }
    }

    pub fn transition_selected_scene<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        scene_id: SceneID,
        t: Clock,
    ) {
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

        self.event_list.add_event(
            Box::new(move |slf: &mut Self, _, _| {
                slf.scene_transition = scene_id;
                slf.scene_transition_type = SceneTransition::SwapTransition;
            }),
            t + 31,
        );
    }

    pub fn contents_mouse_motion_handler<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
    ) {
        if self.current_title_contents.is_none() {
            return;
        }

        match &mut self.current_title_contents.as_mut().unwrap() {
            TitleContents::InitialMenu(contents) => contents.update_highlight(ctx, point),
	    TitleContents::TitleSoundPlayer(_) => (),
        }
    }

    fn switch_current_content(&mut self, content_name: String) {
        let next_content = self.title_contents_set.remove_pickup(&content_name);
        if next_content.is_none() {
            print!("{}?", content_name);
            panic!("target title contents not found.");
        }

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

        match &mut self.current_title_contents.as_mut().unwrap() {
            TitleContents::InitialMenu(contents) => {
                let maybe_event = contents.click_handler(ctx, point);
                if let Some(event) = maybe_event {
                    match event {
                        TitleContentsEvent::NextContents(content_name) => {
                            self.switch_current_content(content_name);
                        },
                        TitleContentsEvent::SceneTransition(scene_id) => {
                            self.transition_selected_scene(ctx, scene_id, t);
                        },
			TitleContentsEvent::BuiltinEvent(command) => {
			    self.run_builtin_command(command);
			},
                    }
                }
            },
	    TitleContents::TitleSoundPlayer(_) => (),
        }
    }
}

impl SceneManager for TitleScene {
    fn key_up_event<'a>(&mut self, ctx: &mut SuzuContext<'a>, vkey: tdev::VirtualKey) {
        let t = self.get_current_clock();

        match vkey {
            VirtualKey::Action1 => {
                self.transition_selected_scene(ctx, SceneID::Scenario, t);
            }
            _ => (),
        }
    }

    fn pre_process<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        let t = self.get_current_clock();

        if let Some(transition_effect) = self.scene_transition_effect.as_mut() {
            transition_effect.effect(ctx.context, t);
        }

        flush_delay_event!(self, self.event_list, ctx, self.get_current_clock());
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.background.draw(ctx).unwrap();

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

    fn mouse_button_up_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        _button: ginput::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        let t = self.get_current_clock();
        self.contents_mouse_click_handler(ctx, point, t);
    }

    fn mouse_motion_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        _offset: numeric::Vector2f,
    ) {
        self.contents_mouse_motion_handler(ctx, point);
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
