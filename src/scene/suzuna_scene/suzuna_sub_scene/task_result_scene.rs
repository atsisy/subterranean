use ginput::mouse::MouseButton;
use torifune::core::*;
use torifune::device as tdev;
use torifune::numeric;

use torifune::graphics::drawable::*;
use torifune::graphics::object::*;

use super::super::*;

use crate::core::{GameData, MouseInformation, TextureID, TileBatchTextureID};
use crate::object::task_result_object::*;
use crate::object::util_object::SelectButton;
use crate::scene::{SceneID, SceneTransition};
use crate::object::effect_object;
use crate::flush_delay_event;

pub struct TaskResultScene {
    clock: Clock,
    mouse_info: MouseInformation,
    event_list: DelayEventList<Self>,
    drawable_task_result: DrawableTaskResult,
    ok_button: SelectButton,
    scene_transition_status: SceneTransition,
    transition_scene: SceneID,
    scene_transition_effect: Option<effect_object::ScreenTileEffect>,
}

impl TaskResultScene {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData, task_result: TaskResult, date: GensoDate) -> Self {
        let mut background_object = MovableUniTexture::new(
            game_data.ref_texture(TextureID::Paper1),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            None,
            0,
        );

        background_object.fit_scale(
            ctx,
            numeric::Vector2f::new(
                crate::core::WINDOW_SIZE_X as f32,
                crate::core::WINDOW_SIZE_Y as f32,
            ),
        );

        let ok_button = SelectButton::new(
            ctx,
            numeric::Rect::new(120.0, 608.0, 80.0, 80.0),
            Box::new(UniTexture::new(
                game_data.ref_texture(TextureID::ChoicePanel1),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
            )),
        );

	let scene_transition = Some(effect_object::ScreenTileEffect::new(
            ctx,
            game_data,
            TileBatchTextureID::Suzu1,
            numeric::Rect::new(
                0.0,
                0.0,
                crate::core::WINDOW_SIZE_X as f32,
                crate::core::WINDOW_SIZE_Y as f32,
            ),
            30,
            effect_object::SceneTransitionEffectType::Open,
            -128,
            0,
        ));

	let mut event_list = DelayEventList::new();
	event_list.add_event(Box::new(move |slf: &mut Self, _, _| { slf.scene_transition_effect = None; }), 31);
	
        TaskResultScene {
            clock: 0,
            mouse_info: MouseInformation::new(),
            event_list: event_list,
            drawable_task_result: DrawableTaskResult::new(
                ctx,
                game_data,
                numeric::Rect::new(0.0, 0.0, 1366.0, 768.0),
                task_result.clone(),
                SimpleObject::new(background_object, Vec::new()),
		date,
                0,
            ),
            ok_button: ok_button,
            scene_transition_status: SceneTransition::Keep,
            transition_scene: SceneID::DayResult,
	    scene_transition_effect: scene_transition,
        }
    }

    fn ready_to_finish_scene(&mut self, ctx: &mut ggez::Context, game_data: &GameData, t: Clock) {
        self.transition_scene = SceneID::Scenario;
        self.scene_transition_status = SceneTransition::SwapTransition;

        self.scene_transition_effect = Some(effect_object::ScreenTileEffect::new(
            ctx,
            game_data,
            TileBatchTextureID::Suzu1,
            numeric::Rect::new(
                0.0,
                0.0,
                crate::core::WINDOW_SIZE_X as f32,
                crate::core::WINDOW_SIZE_Y as f32,
            ),
            30,
            effect_object::SceneTransitionEffectType::Close,
            -128,
            t,
        ));
    }
}

impl SceneManager for TaskResultScene {
    fn key_down_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _game_data: &GameData,
        _vkey: tdev::VirtualKey,
    ) {
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _game_data: &GameData,
        vkey: tdev::VirtualKey,
    ) {
        match vkey {
            tdev::VirtualKey::Action1 => println!("Action1 up!"),
            tdev::VirtualKey::Action2 => {}
            _ => (),
        }
    }

    fn mouse_motion_event(
        &mut self,
        _: &mut ggez::Context,
        _: &GameData,
        point: numeric::Point2f,
        _: numeric::Vector2f,
    ) {
        if self.mouse_info.is_dragging(MouseButton::Left) {
            self.mouse_info
                .set_last_dragged(MouseButton::Left, point, self.get_current_clock());
        }
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        _: &GameData,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        if self.ok_button.contains(ctx, point) {
            self.ok_button.push();
        }

        self.mouse_info
            .set_last_clicked(button, point, self.get_current_clock());
        self.mouse_info
            .set_last_down(button, point, self.get_current_clock());
        self.mouse_info
            .set_last_dragged(button, point, self.get_current_clock());
        self.mouse_info.update_dragging(button, true);
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
	let t = self.get_current_clock();
	
        if self.ok_button.contains(ctx, point) {
            self.ok_button.release();
	    self.ready_to_finish_scene(ctx, game_data, t);
        }

        self.mouse_info.update_dragging(button, false);
        self.mouse_info
            .set_last_up(button, point, self.get_current_clock());
    }

    fn pre_process(&mut self, ctx: &mut ggez::Context, game_data: &GameData) {
	let t = self.get_current_clock();
	
	flush_delay_event!(self, self.event_list, ctx, game_data, t);
	
        self.drawable_task_result
            .effect(ctx, self.get_current_clock());

	if let Some(effect) = self.scene_transition_effect.as_mut() {
	    effect.effect(ctx, t);
	}
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.drawable_task_result.draw(ctx).unwrap();
        self.ok_button.draw(ctx).unwrap();

	if let Some(effect) = self.scene_transition_effect.as_mut() {
	    effect.draw(ctx).unwrap();
	}
    }

    fn post_process(&mut self, _ctx: &mut ggez::Context, _: &GameData) -> SceneTransition {
        self.update_current_clock();
        self.scene_transition_status
    }

    fn transition(&self) -> SceneID {
        self.transition_scene
    }

    fn get_current_clock(&self) -> Clock {
        self.clock
    }

    fn update_current_clock(&mut self) {
        self.clock += 1;
    }
}
