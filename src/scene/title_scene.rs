use torifune::core::Clock;
use torifune::graphics::drawable::*;
use torifune::graphics::object::*;
use torifune::device::*;

use crate::core::{SuzuContext, TextureID, TileBatchTextureID};
use crate::scene::*;
use crate::object::effect_object;

use crate::flush_delay_event;

pub struct TitleScene {
    background: UniTexture,
    event_list: DelayEventList<Self>,
    scene_transition_effect: Option<effect_object::ScreenTileEffect>,
    scene_transition: SceneID,
    scene_transition_type: SceneTransition,
    clock: Clock,
}

impl TitleScene {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>) -> Self {
	let background = UniTexture::new(
	    ctx.resource.ref_texture(TextureID::JpHouseTexture),
	    numeric::Point2f::new(0.0, 0.0),
	    numeric::Vector2f::new(0.7, 0.7),
	    0.0,
	    0
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
	
        TitleScene {
	    background: background,
	    event_list: event_list,
	    scene_transition_effect: scene_transition_effect,
            scene_transition: SceneID::Save,
	    scene_transition_type: SceneTransition::Keep,
            clock: 0,
        }
    }

    pub fn transition_selected_scene<'a>(&mut self, ctx: &mut SuzuContext<'a>, scene_id: SceneID, t: Clock) {
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
	    31,
        );
    }
}

impl SceneManager for TitleScene {

    fn key_up_event<'a>(&mut self, ctx: &mut SuzuContext<'a>, vkey: tdev::VirtualKey) {
	let t = self.get_current_clock();
	
	match vkey {
	    VirtualKey::Action1 => {
		self.transition_selected_scene(ctx, SceneID::Scenario, t);
	    },
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

	if let Some(transition_effect) = self.scene_transition_effect.as_mut() {
            transition_effect.draw(ctx).unwrap();
        }
    }

    fn post_process<'a>(&mut self, _ctx: &mut SuzuContext<'a>) -> SceneTransition {
        self.update_current_clock();

	self.scene_transition_type
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
