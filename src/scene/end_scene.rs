use torifune::core::*;
use torifune::device::*;
use torifune::graphics::drawable::*;
use torifune::graphics::object::*;
use torifune::sound::*;

use crate::{core::{MouseInformation, SoundID, SuzuContext, TextureID, TileBatchTextureID, WINDOW_SIZE_X, WINDOW_SIZE_Y}, object::{effect_object::{SceneTransitionEffectType, TilingEffectType}, map_object::MapObject}};
use crate::object::character_factory;
use crate::object::effect_object;
use crate::object::title_object::*;
use crate::scene::*;
use crate::flush_delay_event;
use crate::add_delay_event;
use crate::object::end_object::*;

pub struct EndScene {
    mouse_info: MouseInformation,
    background: UniTexture,
    event_list: DelayEventList<Self>,
    end_flow: EndSceneFlow,
    scene_transition_effect: Option<effect_object::ScreenTileEffect>,
    scene_transition: SceneID,
    scene_transition_type: SceneTransition,
    bgm_handler: SoundHandler,
    walking_kosuzu: MapObject,
    clock: Clock,
}

impl EndScene {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>) -> Self {
        let background = UniTexture::new(
            ctx.ref_texture(TextureID::TextBackground),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(2.0, 2.0),
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

        let bgm_handler = ctx.play_sound_as_bgm(
            SoundID::Title,
            Some(SoundPlayFlags::new(1000, 1.0, true, 0.1)),
        );

	let mut kosuzu = character_factory::create_endroll_sample(
	    ctx,
	    &numeric::Rect::new(0.0, 0.0, 1366.0, 768.0),
	    numeric::Point2f::new(1050.0, 500.0)
	);
	kosuzu.change_animation_mode(crate::object::util_object::ObjectDirection::MoveLeft);

        EndScene {
            mouse_info: MouseInformation::new(),
            background: background,
            event_list: event_list,
	    end_flow: EndSceneFlow::new(ctx, 0),
            scene_transition_effect: scene_transition_effect,
            scene_transition: SceneID::Save,
            scene_transition_type: SceneTransition::Keep,
            bgm_handler: bgm_handler,
	    walking_kosuzu: kosuzu,
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
            Box::new(move |slf: &mut Self, ctx, _| {
                slf.scene_transition = scene_id;
                slf.scene_transition_type = SceneTransition::SwapTransition;
                ctx.resource.stop_bgm(slf.bgm_handler);
            }),
            t + 31,
        );
    }

    fn run_builtin_command(&mut self, command: TitleBuiltinCommand) {
        match command {
            TitleBuiltinCommand::Exit => std::process::exit(0),
        }
    }

    fn scene_transition_close_effect<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        self.scene_transition_effect = Some(effect_object::ScreenTileEffect::new(
            ctx,
            TileBatchTextureID::Shoji,
            numeric::Rect::new(
                0.0,
                0.0,
                crate::core::WINDOW_SIZE_X as f32,
                crate::core::WINDOW_SIZE_Y as f32,
            ),
            60,
            SceneTransitionEffectType::Close,
            TilingEffectType::WholeTile,
            -128,
            t,
        ));
    }
}

impl SceneManager for EndScene {
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

	self.walking_kosuzu.update_texture(t);
	self.walking_kosuzu.move_map(numeric::Vector2f::new(-0.3, 0.0));
	self.walking_kosuzu.update_display_position(&numeric::Rect::new(0.0, 0.0, WINDOW_SIZE_X as f32, WINDOW_SIZE_Y as f32));
	ctx.process_utility.redraw();
	
        if let Some(transition_effect) = self.scene_transition_effect.as_mut() {
            transition_effect.effect(ctx.context, t);
            ctx.process_utility.redraw();
        }

        if flush_delay_event!(self, self.event_list, ctx, self.get_current_clock()) > 0 {
            ctx.process_utility.redraw();
        }

	self.end_flow.update(ctx, t);
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.background.draw(ctx).unwrap();

	self.end_flow.draw(ctx).unwrap();
	self.walking_kosuzu.draw(ctx).unwrap();

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
        _ctx: &mut SuzuContext<'a>,
        button: ginput::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        let t = self.get_current_clock();

        self.mouse_info.set_last_clicked(button, point, t);
        self.mouse_info.set_last_down(button, point, t);
        self.mouse_info.set_last_dragged(button, point, t);
        self.mouse_info.update_dragging(button, true);

    }

    fn mouse_button_up_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: ginput::mouse::MouseButton,
        _point: numeric::Point2f,
    ) {
        let t = self.get_current_clock();

        self.mouse_info.update_dragging(button, false);

	match button {
	    ginput::mouse::MouseButton::Left => {
		self.scene_transition_close_effect(ctx, t);
		
		add_delay_event!(self.event_list, |slf, _, _| {
		    slf.scene_transition = SceneID::Title;
		    slf.scene_transition_type = SceneTransition::SwapTransition;
		}, t + 40);
	    },
	    _ => (),
	}
    }

    fn mouse_motion_event<'a>(
        &mut self,
        _ctx: &mut SuzuContext<'a>,
        _point: numeric::Point2f,
        _offset: numeric::Vector2f,
    ) {
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
