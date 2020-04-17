use torifune::core::*;
use torifune::device as tdev;
use torifune::numeric;

use torifune::graphics::drawable::*;

use super::super::*;

use crate::core::{MouseInformation, TileBatchTextureID};
use crate::scene::{SceneID, SceneTransition};

use crate::object::copy_scene_object::*;
use crate::object::util_object::*;

use crate::flush_delay_event;

pub struct CopyingScene {
    clock: Clock,
    mouse_info: MouseInformation,
    event_list: DelayEventList<Self>,
    hangi: EffectableHangi,
    copy_data: Vec<CopyingRequestInformation>,
    scene_transition_status: SceneTransition,
    table_frame: TableFrame,
}

impl CopyingScene {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, copy_data: Vec<CopyingRequestInformation>) -> Self {
        CopyingScene {
            clock: 0,
            mouse_info: MouseInformation::new(),
            event_list: DelayEventList::new(),
            hangi: EffectableHangi::new(ctx, numeric::Rect::new(100.0, 100.0, 900.0, 550.0)),
            copy_data: copy_data,
            scene_transition_status: SceneTransition::Keep,
            table_frame: TableFrame::new(
                ctx.resource,
                numeric::Point2f::new(200.0, 200.0),
                TileBatchTextureID::OldStyleFrame,
                //FrameData::new(vec![64.0, 320.0], vec![64.0, 64.0, 64.0, 64.0, 64.0, 64.0, 64.0]),
                FrameData::new(vec![150.0, 150.0], vec![56.0; 3]),
                numeric::Vector2f::new(0.5, 0.5),
                0,
            ),
        }
    }
}

impl SceneManager for CopyingScene {
    fn key_down_event<'a>(&mut self, _ctx: &mut SuzuContext, vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => {
                println!("Action1 down!");
            }
            _ => (),
        }
    }

    fn mouse_motion_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        _: numeric::Vector2f,
    ) {
        if self.mouse_info.is_dragging(MouseButton::Left) {
            self.mouse_info
                .set_last_dragged(MouseButton::Left, point, self.get_current_clock());
            self.hangi.dragging_handler(ctx.context, point);
        }
    }

    fn mouse_button_down_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
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

        //println!("grid_position: {}", self.table_frame.get_grid_position(point));
        let pos = self
            .table_frame
            .get_grid_position(ctx.context, point)
            .unwrap();
        let dest = self.table_frame.get_position();
        println!(
            "grid_position: {}",
            self.table_frame
                .get_grid_topleft(pos, numeric::Vector2f::new(dest.x, dest.y))
        );
    }

    fn mouse_button_up_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        self.mouse_info.update_dragging(button, false);
        self.mouse_info
            .set_last_up(button, point, self.get_current_clock());
        self.hangi.release_handler(ctx.context);
    }

    fn pre_process<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        flush_delay_event!(self, self.event_list, ctx, self.get_current_clock());
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.hangi.draw(ctx).unwrap();
        self.table_frame.draw(ctx).unwrap();
    }

    fn post_process<'a>(&mut self, _ctx: &mut SuzuContext<'a>) -> SceneTransition {
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
