use ggez::graphics as ggraphics;

use torifune::core::Clock;
use torifune::graphics::drawable::*;
use torifune::graphics::object::*;

use crate::core::{FontID, SavableData, SuzuContext, TextureID, TileBatchTextureID};
use crate::object::effect_object;
use crate::object::save_scene_object::*;
use crate::object::util_object::*;
use crate::scene::*;

use crate::flush_delay_event;

pub struct SaveScene {
    background: UniTexture,
    exit_button: SelectButton,
    event_list: DelayEventList<Self>,
    scene_transition_effect: Option<effect_object::ScreenTileEffect>,
    save_entry_table: SaveEntryTable,
    scene_transition: SceneID,
    scene_transition_type: SceneTransition,
    clock: Clock,
}

impl SaveScene {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>) -> Self {
        let save_data_list = (1..=4)
            .map(|slot_index| match SavableData::new_load(slot_index) {
                Ok(savable_data) => Some(savable_data),
                Err(_) => None,
            })
            .collect();

        let save_entry_table = SaveEntryTable::new(
            ctx,
            numeric::Rect::new(50.0, 50.0, 1248.0, 672.0),
            save_data_list,
            0,
        );

        let background = UniTexture::new(
            ctx.ref_texture(TextureID::JpHouseTexture),
            numeric::Point2f::new(0.0, 0.0),
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

        let texture = Box::new(TextButtonTexture::new(
            ctx,
            numeric::Point2f::new(0.0, 0.0),
            "戻る".to_string(),
            FontInformation::new(
                ctx.resource.get_font(FontID::Cinema),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0xf6e1d5ff),
            ),
            10.0,
            ggraphics::Color::from_rgba_u32(0x5a4f3fff),
            0,
        ));

        let exit_button = SelectButton::new(
            ctx,
            numeric::Rect::new(
                1050.0,
                (crate::core::WINDOW_SIZE_Y as f32) - 120.0,
                100.0,
                50.0,
            ),
            texture,
        );

        let mut event_list = DelayEventList::new();
        event_list.add_event(
            Box::new(move |slf: &mut Self, _, _| {
                slf.scene_transition_effect = None;
            }),
            31,
        );

        SaveScene {
            background: background,
            event_list: event_list,
            exit_button: exit_button,
            scene_transition_effect: scene_transition_effect,
            save_entry_table: save_entry_table,
            scene_transition: SceneID::Save,
            scene_transition_type: SceneTransition::Keep,
            clock: 0,
        }
    }

    fn exit_scene_poping<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
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
                slf.scene_transition = SceneID::Scenario;
                slf.scene_transition_type = SceneTransition::PoppingTransition;
            }),
            31,
        );

	if let Some(save_data) = ctx.savable_data.as_mut() {
	    let _ = save_data.get_scenario_save_data();
	}
    }

    fn load_and_scene_swap<'a>(&mut self, ctx: &mut SuzuContext<'a>, slot: u8, t: Clock) {
        match SavableData::new_load(slot) {
            Ok(data) => {
                ctx.savable_data.replace(data);
            }
            Err(_) => return,
        }

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
                slf.scene_transition = SceneID::Scenario;
                slf.scene_transition_type = SceneTransition::SwapTransition;
            }),
            31,
        );
    }
}

impl SceneManager for SaveScene {
    fn mouse_button_up_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        _button: ginput::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        let t = self.get_current_clock();

        match self.save_entry_table.click_handler(ctx, point) {
            SaveDataOperation::Loading(slot) => {
                self.load_and_scene_swap(ctx, slot, t);
            }
            _ => (),
        }

        if self.exit_button.contains(ctx.context, point) {
            self.exit_scene_poping(ctx, t);
        }
    }

    fn pre_process<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        let t = self.get_current_clock();

        if let Some(transition_effect) = self.scene_transition_effect.as_mut() {
            transition_effect.effect(ctx.context, t);
            ctx.process_utility.redraw();
        }

        if flush_delay_event!(self, self.event_list, ctx, self.get_current_clock()) > 0 {
            ctx.process_utility.redraw();
        }
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.background.draw(ctx).unwrap();
        self.save_entry_table.draw(ctx).unwrap();

        self.exit_button.draw(ctx).unwrap();

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
