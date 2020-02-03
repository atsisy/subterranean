use torifune::device as tdev;
use torifune::core::*;
use ggez::graphics as ggraphics;
use ginput::mouse::MouseButton;
use torifune::numeric;

use torifune::graphics::*;
use torifune::graphics::object::*;

use super::*;
use crate::object::Clickable;
use crate::object::task_object;

use crate::core::{MouseInformation, MouseActionRecord, GameData, TextureID};
use crate::object::task_object::*;

use crate::object::task_result_object::*;
    
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskSceneStatus {
    Init,
    CustomerFree,
    CustomerWait,
    CustomerEvent,
    FinishDay,
}

pub struct TaskScene {
    task_table: TaskTable,
    clock: Clock,
    mouse_info: MouseInformation,
    event_list: SceneEventList<Self>,
    status: TaskSceneStatus,
    task_result: TaskResult,
}

impl TaskScene {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData) -> TaskScene {

        TaskScene {
            task_table: TaskTable::new(ctx, game_data,
                                       ggraphics::Rect::new(100.0, 20.0, 1300.0, 750.0),
                                       ggraphics::Rect::new(0.0, 0.0, 800.0, 300.0),
				       numeric::Rect::new(800.0, 0.0, 400.0, 300.0),
                                       ggraphics::Rect::new(0.0, 310.0, 1300.0, 500.0), 0),
            clock: 0,
            mouse_info: MouseInformation::new(),
	    event_list: SceneEventList::new(),
	    status: TaskSceneStatus::Init,
	    task_result: TaskResult::new(),
        }
    }

    fn dragging_handler(&mut self,
                        ctx: &mut ggez::Context,
                        point: numeric::Point2f,
                        _offset: numeric::Vector2f,
                        _game_data: &GameData) {
        let last = self.mouse_info.get_last_dragged(MouseButton::Left);
        self.task_table.dragging_handler(point, last);
        self.task_table.hand_over_check(ctx, point);
    }

    fn unselect_dragging_object(&mut self, ctx: &mut ggez::Context, t: Clock) {
        self.task_table.unselect_dragging_object(ctx, t);
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

    fn insert_customer_event(&mut self,
			     request: CustomerRequest,
			     delay_clock: Clock) {
	self.event_list.add_event(Box::new(
	    |s: &mut TaskScene, ctx: &mut ggez::Context, game_data: &GameData| {
		s.task_table.start_customer_event(
		    ctx, game_data,
		    request, s.get_current_clock());
		s.status = TaskSceneStatus::CustomerEvent;
		s.check_done_today_work();
	    }), self.get_current_clock() + delay_clock);
    }

    fn check_done_today_work(&mut self) {
	if self.task_result.add_done_works(1).get_done_works() > 5 {
	    self.status = TaskSceneStatus::FinishDay;
	    println!("done 5 works!!");
	}
    }

    fn start_borrowing_customer_event(&mut self,
				      game_data: &GameData) {
	self.insert_customer_event(
	    CustomerRequest::Borrowing(
		task_object::BorrowingInformation::new_random(
		    game_data,
		    task_object::GensoDate::new(128, 12, 20),
		    task_object::GensoDate::new(128, 12, 20))), 100);
    }

    fn start_copying_customer_event(&mut self,
				      game_data: &GameData) {
	self.insert_customer_event(
	    CustomerRequest::Copying(task_object::CopyingRequestInformation::new_random(
			game_data,
			GensoDate::new(12, 12, 12),
			GensoDate::new(12, 12, 12))), 100);
    }
    
    fn start_customer_event(&mut self,
			    game_data: &GameData) {
	match rand::random::<usize>() % 2 {
	    0 => self.start_borrowing_customer_event(game_data),
	    1 => self.start_copying_customer_event(game_data),
	    _ => (),
	}
    }

    pub fn get_task_result(&self) -> &TaskResult {
	&self.task_result
    }

    pub fn reset_task_result(&mut self) {
	self.task_result.reset();
    }

    pub fn get_task_status(&self) -> TaskSceneStatus {
	self.status
    }
}

impl SceneManager for TaskScene {

    fn key_down_event(&mut self,
                      _ctx: &mut ggez::Context,
                      _game_data: &GameData,
                      vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => {
                println!("Action1 down!");
		self.event_list.add_event(Box::new(|_, _, _| println!("aaaaaaaaaa") ), self.get_current_clock() + 2);
		self.event_list.add_event(Box::new(|_, _, _| println!("bbbbbbbbbb") ), self.get_current_clock() + 500);
            },
	    tdev::VirtualKey::Action3 => {
		self.task_table.clear_hold_data();
	    }
            _ => (),
        }
    }

    fn key_up_event(&mut self,
                    _ctx: &mut ggez::Context,
                    _game_data: &GameData,
                    vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => println!("Action1 up!"),
            tdev::VirtualKey::Action2 => {
            },
            _ => (),
        }
    }

    fn mouse_motion_event(&mut self,
                          ctx: &mut ggez::Context,
                          game_data: &GameData,
                          point: numeric::Point2f,
                          offset: numeric::Vector2f) {
        if self.mouse_info.is_dragging(MouseButton::Left) {
            let d = numeric::Vector2f::new(offset.x / 2.0, offset.y / 2.0);
            self.dragging_handler(ctx, point, d, game_data);
            self.mouse_info.set_last_dragged(MouseButton::Left, point, self.get_current_clock());
        }
	let mouse_cursor_status = self.task_table.clickable_status(ctx, point);
	ggez::input::mouse::set_cursor_type(ctx, mouse_cursor_status);
    }

    fn mouse_button_down_event(&mut self,
                               ctx: &mut ggez::Context,
                               game_data: &GameData,
                               button: MouseButton,
                               point: numeric::Point2f) {
        let info: &MouseActionRecord = &self.mouse_info.last_clicked.get(&button).unwrap();
        if info.point == point {
	    if (self.get_current_clock() - info.t) < 20 {
		self.task_table.double_click_handler(ctx, point, game_data);
            }
	}
	
        self.mouse_info.set_last_clicked(button, point, self.get_current_clock());
	self.mouse_info.set_last_down(button, point, self.get_current_clock());
        self.mouse_info.set_last_dragged(button, point, self.get_current_clock());
        self.mouse_info.update_dragging(button, true);

	self.task_table.button_down(ctx, game_data, self.get_current_clock(), button, point);
    }

    fn mouse_button_up_event(&mut self,
                             ctx: &mut ggez::Context,
                             game_data: &GameData,
                             button: MouseButton,
                             point: numeric::Point2f) {
        self.mouse_info.update_dragging(button, false);
        //self.paper.button_up(ctx, button, point);
        self.unselect_dragging_object(ctx, self.get_current_clock());
	self.task_table.button_up(ctx, game_data, self.get_current_clock(), button, point);

	let info: &MouseActionRecord = &self.mouse_info.last_down.get(&button).unwrap();
	if info.point == point {
	    self.task_table.on_click(ctx, game_data, self.get_current_clock(), button, point);
	}
	
	self.mouse_info.set_last_up(button, point, self.get_current_clock());
    }

    fn pre_process(&mut self,
                   ctx: &mut ggez::Context,
                   game_data: &GameData) {
        self.task_table.update(ctx, game_data, self.get_current_clock());

	if (self.status == TaskSceneStatus::CustomerEvent || self.status == TaskSceneStatus::Init) &&
	    self.task_table.get_remaining_customer_object_number() == 0 {
	    self.status = TaskSceneStatus::CustomerFree;
	}
	
	if self.status == TaskSceneStatus::CustomerFree {
	    self.start_customer_event(game_data);
	    self.status = TaskSceneStatus::CustomerWait;
	}
	
	self.run_scene_event(ctx, game_data, self.get_current_clock());
    }
    
    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.task_table.draw(ctx).unwrap();
    }
    
    fn post_process(&mut self, _ctx: &mut ggez::Context, _: &GameData) -> SceneTransition {
        self.update_current_clock();
        SceneTransition::Keep
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

pub struct TaskResultScene {
    clock: Clock,
    mouse_info: MouseInformation,
    event_list: SceneEventList<Self>,
    task_result: TaskResult,
    drawable_task_result: DrawableTaskResult,
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
            0);

        TaskResultScene {
            clock: 0,
            mouse_info: MouseInformation::new(),
	    event_list: SceneEventList::new(),
	    drawable_task_result: DrawableTaskResult::new(ctx, game_data,
							  numeric::Rect::new(0.0, 0.0, 1000.0, 700.0),
							  task_result.clone(),
							  SimpleObject::new(background_object, Vec::new())),
	    task_result: task_result,
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

impl SceneManager for TaskResultScene {

    fn key_down_event(&mut self,
                      _ctx: &mut ggez::Context,
                      _game_data: &GameData,
                      vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => {
                println!("Action1 down!");
		self.event_list.add_event(Box::new(|_, _, _| println!("aaaaaaaaaa") ), self.get_current_clock() + 2);
		self.event_list.add_event(Box::new(|_, _, _| println!("bbbbbbbbbb") ), self.get_current_clock() + 500);
            },
            _ => (),
        }
    }

    fn key_up_event(&mut self,
                    _ctx: &mut ggez::Context,
                    _game_data: &GameData,
                    vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => println!("Action1 up!"),
            tdev::VirtualKey::Action2 => {
            },
            _ => (),
        }
    }

    fn mouse_motion_event(&mut self,
                          ctx: &mut ggez::Context,
                          game_data: &GameData,
                          point: numeric::Point2f,
                          offset: numeric::Vector2f) {
        if self.mouse_info.is_dragging(MouseButton::Left) {
            self.mouse_info.set_last_dragged(MouseButton::Left, point, self.get_current_clock());
        }
    }

    fn mouse_button_down_event(&mut self,
                               ctx: &mut ggez::Context,
                               game_data: &GameData,
                               button: MouseButton,
                               point: numeric::Point2f) {	
        self.mouse_info.set_last_clicked(button, point, self.get_current_clock());
	self.mouse_info.set_last_down(button, point, self.get_current_clock());
        self.mouse_info.set_last_dragged(button, point, self.get_current_clock());
        self.mouse_info.update_dragging(button, true);

    }

    fn mouse_button_up_event(&mut self,
                             ctx: &mut ggez::Context,
                             game_data: &GameData,
                             button: MouseButton,
                             point: numeric::Point2f) {
        self.mouse_info.update_dragging(button, false);
	self.mouse_info.set_last_up(button, point, self.get_current_clock());
    }

    fn pre_process(&mut self,
                   ctx: &mut ggez::Context,
                   game_data: &GameData) {
	self.run_scene_event(ctx, game_data, self.get_current_clock());
    }
    
    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
	self.drawable_task_result.draw(ctx).unwrap();
    }
    
    fn post_process(&mut self, _ctx: &mut ggez::Context, _: &GameData) -> SceneTransition {
        self.update_current_clock();
        SceneTransition::Keep
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
