use ggez::input::mouse::MouseButton;
use torifune::core::*;
use torifune::numeric;

use torifune::graphics::drawable::*;
use torifune::graphics::object::*;

use super::super::*;

use crate::flush_delay_event_and_redraw_check;
use crate::object::effect_object;
use crate::object::task_result_object::*;
use crate::object::util_object;
use crate::scene::{SceneID, SceneTransition};
use crate::{core::SoundID, flush_delay_event};

#[allow(unused_imports)]
use crate::{
    core::{MouseInformation, ResultReport, SavableData, TextureID, TileBatchTextureID},
    perf_measure,
};
use effect_object::TilingEffectType;

pub struct TaskResultScene {
    clock: Clock,
    mouse_info: MouseInformation,
    event_list: DelayEventList<Self>,
    drawable_task_result: DrawableTaskResult,
    ok_button: util_object::FramedButton,
    scene_transition_status: SceneTransition,
    transition_scene: SceneID,
    scene_transition_effect: Option<effect_object::ScreenTileEffect>,
}

impl TaskResultScene {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        initial_save_data: SavableData,
        result_report: ResultReport,
        date: GensoDate,
    ) -> Self {
        let mut background_object = MovableUniTexture::new(
            Box::new(UniTexture::new(
                ctx.ref_texture(TextureID::Paper1),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
            )),
            None,
            0,
        );

        background_object.fit_scale(
            ctx.context,
            numeric::Vector2f::new(
                crate::core::WINDOW_SIZE_X as f32,
                crate::core::WINDOW_SIZE_Y as f32,
            ),
        );

        let mut ok_button = util_object::FramedButton::create_design1(
            ctx,
            numeric::Point2f::new(60.0, 550.0),
            "戸締まり",
            numeric::Vector2f::new(28.0, 28.0),
        );

	ok_button.hide();

        let scene_transition = Some(effect_object::ScreenTileEffect::new(
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

        ctx.play_sound_as_se(SoundID::ResultSE, None);

        TaskResultScene {
            clock: 0,
            mouse_info: MouseInformation::new(),
            event_list: event_list,
            drawable_task_result: DrawableTaskResult::new(
                ctx,
                numeric::Rect::new(0.0, 0.0, 1366.0, 768.0),
                SimpleObject::new(background_object, Vec::new()),
                initial_save_data,
                result_report,
                date,
                0,
            ),
            ok_button: ok_button,
            scene_transition_status: SceneTransition::Keep,
            transition_scene: SceneID::DayResult,
            scene_transition_effect: scene_transition,
        }
    }

    fn ready_to_finish_scene<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        self.transition_scene = SceneID::Scenario;
        self.scene_transition_status = SceneTransition::SwapTransition;
        ctx.go_next_day();

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
            TilingEffectType::WholeTile,
            -128,
            t,
        ));
    }
}

impl SceneManager for TaskResultScene {
    fn mouse_motion_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        _: numeric::Vector2f,
    ) {
        self.ok_button.mouse_motion_handler(ctx, point);
        if self.mouse_info.is_dragging(MouseButton::Left) {
            self.mouse_info
                .set_last_dragged(MouseButton::Left, point, self.get_current_clock());
        }
    }

    fn mouse_button_down_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        if self.ok_button.contains(point) {
            self.ok_button.make_this_pressed_status(ctx);
        }

        self.mouse_info
            .set_last_clicked(button, point, self.get_current_clock());
        self.mouse_info
            .set_last_down(button, point, self.get_current_clock());
        self.mouse_info
            .set_last_dragged(button, point, self.get_current_clock());
        self.mouse_info.update_dragging(button, true);
    }

    fn mouse_button_up_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        let t = self.get_current_clock();

        if self.ok_button.is_visible() && self.ok_button.contains(point) {
            self.ok_button.make_this_hovered_status(ctx);
            self.ready_to_finish_scene(ctx, t);
        }

        self.drawable_task_result.click_handler(ctx, t);

        self.mouse_info.update_dragging(button, false);
        self.mouse_info
            .set_last_up(button, point, self.get_current_clock());
    }

    fn pre_process<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        let t = self.get_current_clock();

        //println!("ResultScene::pre_process {}", perf_measure!({
        flush_delay_event_and_redraw_check!(self, self.event_list, ctx, t, {});

        self.drawable_task_result
            .run_effect(ctx, self.get_current_clock());

	if self.drawable_task_result.evaluation_flow_is_done() {
	    self.ok_button.appear();
	}

        if let Some(effect) = self.scene_transition_effect.as_mut() {
            effect.effect(ctx.context, t);
            ctx.process_utility.redraw();
        }
        //}));
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        //println!("ResultScene::drawing_process {}", perf_measure!({
        self.drawable_task_result.draw(ctx).unwrap();
        self.ok_button.draw(ctx).unwrap();

        if let Some(effect) = self.scene_transition_effect.as_mut() {
            effect.draw(ctx).unwrap();
        }
        //}));
    }

    fn post_process<'a>(&mut self, _ctx: &mut SuzuContext<'a>) -> SceneTransition {
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
