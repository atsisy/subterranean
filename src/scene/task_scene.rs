use torifune::device as tdev;
use torifune::core::*;
use ggez::graphics as ggraphics;
use ginput::mouse::MouseButton;
use torifune::numeric;

use torifune::graphics::*;

use super::*;
use crate::object::Clickable;

use crate::object::task_object;

use crate::core::GameData;
use crate::object::task_object::*;
use crate::object::simulation_ui as sui;

///
/// # 遅延イベントを起こすための情報を保持する
///
/// ## run_time
/// 処理が走る時間
///
/// ## func
/// run_time時に実行される処理
///
struct SceneEvent<T> {
    run_time: Clock,
    func: Box<dyn Fn(&mut T, &mut ggez::Context, &GameData) -> ()>,
}

impl<T> SceneEvent<T> {
    pub fn new(f: Box<dyn Fn(&mut T, &mut ggez::Context, &GameData) -> ()>, t: Clock) -> Self {
	SceneEvent::<T> {
	    run_time: t,
	    func: f,
	}
    }
}

///
/// # 遅延イベントを保持しておく構造体
///
/// ## list
/// 遅延イベントのリスト, run_timeでソートされている
///
struct SceneEventList<T> {
    list: Vec<SceneEvent<T>>,
}

impl<T> SceneEventList<T> {
    pub fn new() -> Self {
	SceneEventList::<T> {
	    list: Vec::new(),
	}
    }

    pub fn add_event(&mut self, f: Box<dyn Fn(&mut T, &mut ggez::Context, &GameData) -> ()>,
		     t: Clock) -> &mut Self {
	self.add(SceneEvent::new(f, t))
    }

    pub fn add(&mut self, event: SceneEvent<T>) -> &mut Self {
	self.list.push(event);
	self.list.sort_by(|o1, o2| { o2.run_time.cmp(&o1.run_time) });
	self
    }

    pub fn move_top(&mut self) -> Option<SceneEvent<T>> {
	if self.list.len() > 0 {
	    self.list.pop()
	} else {
	    None
	}
    }

    pub fn len(&self) -> usize {
	self.list.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TaskSceneStatus {
    Init,
    CustomerFree,
    CustomerWait,
    CustomerEvent,
}

pub struct TaskScene {
    task_table: TaskTable,
    simulation_status: sui::SimulationStatus,
    clock: Clock,
    mouse_info: MouseInformation,
    event_list: SceneEventList<Self>,
    status: TaskSceneStatus,
}

impl TaskScene {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData) -> TaskScene  {

        TaskScene {
            task_table: TaskTable::new(ctx, game_data,
                                       ggraphics::Rect::new(10.0, 230.0, 1300.0, 500.0),
                                       ggraphics::Rect::new(10.0, 0.0, 520.0, 500.0),
                                       ggraphics::Rect::new(540.0, 0.0, 900.0, 500.0), 0),
	    simulation_status: sui::SimulationStatus::new(ctx, numeric::Rect::new(0.0, 0.0, 1366.0, 180.0), game_data),
            clock: 0,
            mouse_info: MouseInformation::new(),
	    event_list: SceneEventList::new(),
	    status: TaskSceneStatus::Init,
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
                    ctx: &mut ggez::Context,
                    game_data: &GameData,
                    vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => println!("Action1 up!"),
            tdev::VirtualKey::Action2 => {
                self.task_table.start_customer_event(
                    ctx, game_data,
                    task_object::BorrowingInformation::new_random(
                        game_data,
                        task_object::GensoDate::new(128, 12, 20),
                        task_object::GensoDate::new(128, 12, 20)
                    ), self.get_current_clock());
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
    }

    fn mouse_button_down_event(&mut self,
                               ctx: &mut ggez::Context,
                               game_data: &GameData,
                               button: MouseButton,
                               point: numeric::Point2f) {
        let info: &MouseActionRecord = &self.mouse_info.last_clicked.get(&button).unwrap();
        if info.point == point && (self.get_current_clock() - info.t) < 20 {
            self.task_table.double_click_handler(ctx, point, game_data);
        }

        self.mouse_info.set_last_clicked(button, point, self.get_current_clock());
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
	    self.event_list.add_event(Box::new(
		|s: &mut TaskScene, ctx: &mut ggez::Context, game_data: &GameData| {
		    s.task_table.start_customer_event(
			ctx, game_data,
			task_object::BorrowingInformation::new_random(
			    game_data,
			    task_object::GensoDate::new(128, 12, 20),
			    task_object::GensoDate::new(128, 12, 20)
			), s.get_current_clock());
		    s.status = TaskSceneStatus::CustomerEvent;
		}), self.get_current_clock() + 100);
	    self.status = TaskSceneStatus::CustomerWait;
	}
	
	self.run_scene_event(ctx, game_data, self.get_current_clock());
    }
    
    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.task_table.draw(ctx).unwrap();
	self.simulation_status.draw(ctx).unwrap();
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
