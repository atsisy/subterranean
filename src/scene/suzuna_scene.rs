pub mod suzuna_sub_scene;

use ggez::input::mouse::MouseButton;

use torifune::core::*;
use torifune::debug;
use torifune::device::VirtualKey;
use torifune::numeric;

use crate::core::{GensoDate, SuzuContext};
use crate::scene::*;

use suzuna_sub_scene::*;

pub struct SuzunaScene {
    clock: Clock,
    sub_scene: SuzunaSubScene,
}

impl SuzunaScene {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, suzuna_map_id: u32) -> Self {
        SuzunaScene {
            clock: 0,
            sub_scene: SuzunaSubScene::new(ctx, suzuna_map_id),
        }
    }

    fn transition_shop_scene_to_others<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        transition_status: SceneTransition,
    ) -> SceneTransition {
        if transition_status == SceneTransition::StackingTransition {
            if self.sub_scene.get_shop_scene_mut().unwrap().transition() == SceneID::MainDesk {
                debug::debug_screen_push_text("switch shop -> work");
                self.sub_scene
                    .switch_shop_to_deskwork(ctx, transition_status);
            }

            return SceneTransition::Keep;
        }

        if self.sub_scene.get_shop_scene_mut().unwrap().transition() == SceneID::DayResult {
            debug::debug_screen_push_text("switch shop -> result");
            self.sub_scene
                .switch_shop_to_day_result(ctx, transition_status);

            return SceneTransition::Keep;
        }

        if self.sub_scene.get_shop_scene_mut().unwrap().transition() == SceneID::Title {
            debug::debug_screen_push_text("switch shop -> title");
            return SceneTransition::SwapTransition;
        }

        return SceneTransition::Keep;
    }
}

impl SceneManager for SuzunaScene {
    fn key_down_event<'a>(&mut self, ctx: &mut SuzuContext<'a>, vkey: VirtualKey) {
        self.sub_scene.key_down_event(ctx, vkey);
    }

    fn key_up_event<'a>(&mut self, ctx: &mut SuzuContext<'a>, vkey: VirtualKey) {
        self.sub_scene.key_up_event(ctx, vkey);
    }

    fn mouse_motion_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        offset: numeric::Vector2f,
    ) {
        self.sub_scene.mouse_motion_event(ctx, point, offset);
    }

    fn mouse_button_down_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        self.sub_scene.mouse_button_down_event(ctx, button, point);
    }

    fn mouse_button_up_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        self.sub_scene.mouse_button_up_event(ctx, button, point);
    }

    fn mouse_wheel_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        x: f32,
        y: f32,
    ) {
        self.sub_scene.mouse_wheel_event(ctx, point, x, y);
    }

    fn pre_process<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        self.sub_scene.pre_process(ctx)
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.sub_scene.drawing_process(ctx);
    }

    fn post_process<'a>(&mut self, ctx: &mut SuzuContext<'a>) -> SceneTransition {
        let transition_status = self.sub_scene.post_process(ctx);

        match self.sub_scene.get_scene_status() {
            SuzunaSceneStatus::Shop => {
                return self.transition_shop_scene_to_others(ctx, transition_status);
            }
            SuzunaSceneStatus::DeskWork => {
                let transition = self
                    .sub_scene
                    .get_deskwork_scene_mut()
                    .unwrap()
                    .transition();

                if transition_status == SceneTransition::PoppingTransition {
                    if transition == SceneID::SuzunaShop {
                        self.sub_scene
                            .switch_deskwork_to_shop(ctx, transition_status);
                    }
                } else if transition_status == SceneTransition::SwapTransition {
                    if transition == SceneID::Title {
                        return SceneTransition::SwapTransition;
                    }
                }
            }
            SuzunaSceneStatus::DayResult => {
                return transition_status;
            }
        }

        SceneTransition::Keep
    }

    fn transition(&self) -> SceneID {
        self.sub_scene.transition()
    }

    fn get_current_clock(&self) -> Clock {
        self.clock
    }

    fn update_current_clock(&mut self) {
        self.clock += 1;
    }
}
