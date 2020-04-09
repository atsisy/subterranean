use ginput::mouse::MouseButton;
use torifune::core::*;
use torifune::device as tdev;
use torifune::numeric;

use torifune::graphics::object::*;
use torifune::graphics::drawable::*;

use super::super::*;

use crate::core::{GameData, MouseInformation, TextureID};
use crate::object::task_result_object::*;
use crate::scene::{SceneID, SceneTransition};

pub struct TaskResultScene {
    clock: Clock,
    mouse_info: MouseInformation,
    event_list: DelayEventList<Self>,
    drawable_task_result: DrawableTaskResult,
    scene_transition_status: SceneTransition,
    transition_scene: SceneID,
}

impl TaskResultScene {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData, task_result: TaskResult) -> Self {
        let background_object = MovableUniTexture::new(
            game_data.ref_texture(TextureID::Paper1),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            None,
            0,
        );

        TaskResultScene {
            clock: 0,
            mouse_info: MouseInformation::new(),
            event_list: DelayEventList::new(),
            drawable_task_result: DrawableTaskResult::new(
                ctx,
                game_data,
                numeric::Rect::new(0.0, 0.0, 1000.0, 700.0),
                task_result.clone(),
                SimpleObject::new(background_object, Vec::new()),
                0,
            ),
            scene_transition_status: SceneTransition::Keep,
            transition_scene: SceneID::DayResult,
        }
    }

    ///
    /// 遅延処理を走らせるメソッド
    ///
    fn run_scene_event(&mut self, ctx: &mut ggez::Context, game_data: &GameData, t: Clock) {
        // 最後の要素の所有権を移動
        while let Some(event) = self.event_list.move_top() {
            // 時間が来ていない場合は、取り出した要素をリストに戻して処理ループを抜ける
            if event.run_time > t {
                self.event_list.add(event);
                break;
            }

            // 所有権を移動しているため、selfを渡してもエラーにならない
            (event.func)(self, ctx, game_data);
        }
    }

    fn ready_to_finish_scene(&mut self) {
        self.transition_scene = SceneID::Scenario;
        self.scene_transition_status = SceneTransition::SwapTransition;
    }
}

impl SceneManager for TaskResultScene {
    fn key_down_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _game_data: &GameData,
        vkey: tdev::VirtualKey,
    ) {
        match vkey {
            tdev::VirtualKey::Action1 => {
                self.ready_to_finish_scene();
            }
            _ => (),
        }
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
        _: &mut ggez::Context,
        _: &GameData,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
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
        _: &mut ggez::Context,
        _: &GameData,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        self.mouse_info.update_dragging(button, false);
        self.mouse_info
            .set_last_up(button, point, self.get_current_clock());
    }

    fn pre_process(&mut self, ctx: &mut ggez::Context, game_data: &GameData) {
        self.run_scene_event(ctx, game_data, self.get_current_clock());
        self.drawable_task_result
            .effect(ctx, self.get_current_clock());
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.drawable_task_result.draw(ctx).unwrap();
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
