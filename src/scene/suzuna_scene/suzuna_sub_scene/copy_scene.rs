use torifune::core::*;
use torifune::device as tdev;
use torifune::numeric;

use torifune::graphics::*;

use super::super::*;

use crate::core::{GameData, MouseInformation};
use crate::scene::{SceneID, SceneTransition};

use crate::object::copy_scene_object::*;

pub struct CopyingScene {
    clock: Clock,
    mouse_info: MouseInformation,
    event_list: DelayEventList<Self>,
    hangi: EffectableHangi,
    copy_data: Vec<CopyingRequestInformation>,
    scene_transition_status: SceneTransition,
}

impl CopyingScene {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        copy_data: Vec<CopyingRequestInformation>,
    ) -> Self {
        CopyingScene {
            clock: 0,
            mouse_info: MouseInformation::new(),
            event_list: DelayEventList::new(),
            hangi: EffectableHangi::new(
                ctx,
                game_data,
                numeric::Rect::new(100.0, 100.0, 900.0, 550.0),
            ),
            copy_data: copy_data,
            scene_transition_status: SceneTransition::Keep,
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
}

impl SceneManager for CopyingScene {
    fn key_down_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _game_data: &GameData,
        vkey: tdev::VirtualKey,
    ) {
        match vkey {
            tdev::VirtualKey::Action1 => {
                println!("Action1 down!");
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
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.hangi.draw(ctx).unwrap();
    }

    fn post_process(&mut self, _ctx: &mut ggez::Context, _: &GameData) -> SceneTransition {
        self.update_current_clock();
        self.scene_transition_status
    }

    fn transition(&self) -> SceneID {
        SceneID::MainDesk
    }

    fn get_current_clock(&self) -> Clock {
        self.clock
    }

    fn update_current_clock(&mut self) {
        self.clock += 1;
    }
}
