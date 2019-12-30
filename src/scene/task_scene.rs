use std::collections::HashMap;
use torifune::device as tdev;
use torifune::core::Clock;
use torifune::graphics as tgraphics;
use tgraphics::object as tobj;
use ggez::input as ginput;
use ginput::mouse::MouseButton;
use ggez::graphics as ggraphics;
use torifune::numeric;
use torifune::hash;

use crate::core::{TextureID, GameData};

use crate::object::scenario::*;
use crate::object::simulation_ui as sui;
use torifune::graphics::*;
use torifune::graphics::object::TextureObject;
use torifune::graphics::object::MovableObject;

use super::*;


#[derive(PartialEq, Debug, Clone, Copy)]
struct MouseActionRecord {
    point: numeric::Point2f,
    t: Clock,
}

impl MouseActionRecord {
    fn new(point: numeric::Point2f, t: Clock) -> MouseActionRecord {
        MouseActionRecord {
            point: point,
            t: t
        }
    }

    fn new_empty() -> MouseActionRecord {
        MouseActionRecord {
            point: numeric::Point2f::new(0.0, 0.0),
            t: 0
        }
    }
}

struct MouseInformation {
    last_clicked: HashMap<MouseButton, MouseActionRecord>,
    last_dragged: HashMap<MouseButton, MouseActionRecord>,
    dragging: HashMap<MouseButton, bool>,
}

impl MouseInformation {

    fn new() -> MouseInformation {
        MouseInformation {
            last_clicked: hash![(MouseButton::Left, MouseActionRecord::new_empty()),
                                (MouseButton::Right, MouseActionRecord::new_empty()),
                                (MouseButton::Middle, MouseActionRecord::new_empty())],
            last_dragged: hash![(MouseButton::Left, MouseActionRecord::new_empty()),
                                (MouseButton::Right, MouseActionRecord::new_empty()),
                                (MouseButton::Middle, MouseActionRecord::new_empty())],
            dragging: hash![(MouseButton::Left, false),
                            (MouseButton::Right, false),
                            (MouseButton::Middle, false)]
        }
    }

    fn get_last_clicked(&self, button: MouseButton) -> numeric::Point2f {
        match self.last_clicked.get(&button) {
            Some(x) => x.point,
            None => panic!("No such a mouse button"),
        }
    }

    fn set_last_clicked(&mut self, button: MouseButton, point: numeric::Point2f, t: Clock) {
        if self.last_clicked.insert(button, MouseActionRecord::new(point, t)) == None {
            panic!("No such a mouse button")
        }
    }

    fn get_last_dragged(&self, button: MouseButton) -> numeric::Point2f {
        match self.last_dragged.get(&button) {
            Some(x) => x.point,
            None => panic!("No such a mouse button"),
        }
    }

    fn set_last_dragged(&mut self, button: MouseButton, point: numeric::Point2f, t: Clock) {
        if self.last_dragged.insert(button, MouseActionRecord::new(point, t)) == None {
            panic!("No such a mouse button")
        }
    }

    fn is_dragging(&self, button: ginput::mouse::MouseButton) -> bool {
        match self.dragging.get(&button) {
            Some(x) => *x,
            None => panic!("No such a mouse button"),
        }
    }

    fn update_dragging(&mut self, button: MouseButton, drag: bool) {
        if self.dragging.insert(button, drag) == None {
            panic!("No such a mouse button")
        }
    }
    
}

struct DeskObjects {
    desk_canvas: tgraphics::SubScreen,
    desk_objects: SimpleObjectContainer,
    dragging: Option<tobj::SimpleObject>,
}

impl DeskObjects {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData,
               rect: ggraphics::Rect) -> DeskObjects {

        let mut dparam = ggraphics::DrawParam::default();
        dparam.dest = numeric::Point2f::new(rect.x, rect.y).into();
        
        let mut desk_objects = SimpleObjectContainer::new();
        
        desk_objects.add(tobj::SimpleObject::new(
            tobj::MovableUniTexture::new(
                game_data.ref_texture(TextureID::Ghost1),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(0.1, 0.1),
                0.0, 0,  Box::new(move |p: & dyn tobj::MovableObject, _t: Clock| {
                    torifune::numeric::Point2f::new(p.get_position().x + 8.0, p.get_position().y)
                }),
                0), vec![]));
        desk_objects.add(tobj::SimpleObject::new(
            tobj::MovableUniTexture::new(
                game_data.ref_texture(TextureID::LotusPink),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(0.1, 0.1),
                0.0, -1,  Box::new(move |p: & dyn tobj::MovableObject, _t: Clock| {
                    torifune::numeric::Point2f::new(p.get_position().x + 8.0, p.get_position().y)
                }),
                0), vec![]));
        desk_objects.sort_with_depth();
        
        DeskObjects {
            desk_canvas: tgraphics::SubScreen::new(ctx, rect, 0, ggraphics::Color::new(1.0, 1.0, 1.0, 1.0)),
            desk_objects: desk_objects,
            dragging: None,
        }
    }

    
    fn dragging_handler(&mut self,
                        point: numeric::Point2f,
                        last: numeric::Point2f) {
        if let Some(obj) = &mut self.dragging {
            obj.move_diff(numeric::Vector2f::new(point.x - last.x, point.y - last.y));
        }
    }

    fn select_dragging_object(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {

        let mut dragging_object_index = 0;
        let mut drag_start = false;
        
        // オブジェクトは深度が深い順にソートされているので、
        // 逆順から検索していくことで、最も手前に表示されているオブジェクトを
        // 取り出すことができる
        for (index, obj) in self.desk_objects.get_raw_container_mut().iter_mut().rev().enumerate() {
            if obj.get_drawing_area(ctx).contains(point) {
                dragging_object_index = self.desk_objects.len() - index - 1;
                drag_start = true;
                break;
            }
        }
        if drag_start {
            // 元々、最前面に表示されていたオブジェクトのdepthに設定する
            self.dragging = Some(self.desk_objects.get_raw_container_mut()
                                 .swap_remove(dragging_object_index));
        }
    }

    fn unselect_dragging_object(&mut self) {
        if let Some(obj) = &mut self.dragging {
            let min = self.desk_objects.get_minimum_depth();
            obj.set_drawing_depth(min);
            self.desk_objects.change_depth_equally(1);
        }
        match self.dragging {
            None =>  (),
            _ => {
                self.desk_objects.add(std::mem::replace(&mut self.dragging, None).unwrap());
                self.desk_objects.sort_with_depth();
            }
        }
    }

    fn update(&mut self, _ctx: &mut ggez::Context, t: Clock) {
        for p in self.desk_objects.get_raw_container_mut() {
            p.move_with_func(t);
        }
    }

}

impl tgraphics::DrawableComponent for DeskObjects {
    fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.desk_canvas.begin_drawing(ctx);
            for obj in self.desk_objects.get_raw_container() {
                obj.draw(ctx)?;
            }
            if let Some(p) = &self.dragging {
                p.draw(ctx).unwrap();
            }
            self.desk_canvas.end_drawing(ctx);
            self.desk_canvas.draw(ctx).unwrap();
            
        }
        Ok(())
    }

    fn hide(&mut self) {
        self.desk_canvas.hide();
    }

    fn appear(&mut self) {
        self.desk_canvas.appear();
    }

    fn is_visible(&self) -> bool {
        self.desk_canvas.is_visible()
    }

    /// 描画順序を設定する
    fn set_drawing_depth(&mut self, depth: i8) {
        self.desk_canvas.set_drawing_depth(depth);
    }

    /// 描画順序を返す
    fn get_drawing_depth(&self) -> i8 {
        self.desk_canvas.get_drawing_depth()
    }

}

impl tgraphics::DrawableObject for DeskObjects {

    /// 描画開始地点を設定する
    fn set_position(&mut self, pos: numeric::Point2f) {
        self.desk_canvas.set_position(pos);
    }

    /// 描画開始地点を返す
    fn get_position(&self) -> numeric::Point2f {
        self.desk_canvas.get_position()
    }

    /// offsetで指定しただけ描画位置を動かす
    fn move_diff(&mut self, offset: numeric::Vector2f) {
        self.desk_canvas.move_diff(offset)
    }
}

pub struct TaskScene {
    desk_objects: DeskObjects,
    scenario_event: ScenarioEvent,
    simulation_status: sui::SimulationStatus,
    clock: Clock,
    mouse_info: MouseInformation,
}

impl TaskScene {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData) -> TaskScene  {
        let scenario = ScenarioEvent::new(ctx, numeric::Rect::new(100.0, 520.0, 1200.0, 280.0),
                                          "./resources/scenario_parsing_test.toml",
                                          game_data, 0);
        
        TaskScene {
            desk_objects: DeskObjects::new(ctx, game_data,
                                           ggraphics::Rect::new(150.0, 150.0, 500.0, 500.0)),
            simulation_status: sui::SimulationStatus::new(ctx, numeric::Rect::new(0.0, 0.0, 1366.0, 200.0), game_data),
            scenario_event: scenario,
            clock: 0,
            mouse_info: MouseInformation::new(),
        }
    }
    
    fn dragging_handler(&mut self,
                        _ctx: &mut ggez::Context,
                        point: numeric::Point2f,
                        _offset: numeric::Vector2f) {
        let last = self.mouse_info.get_last_dragged(MouseButton::Left);
        self.desk_objects.dragging_handler(point, last);
    }

    fn select_dragging_object(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        self.desk_objects.select_dragging_object(ctx, point);
    }

    fn unselect_dragging_object(&mut self) {
        self.desk_objects.unselect_dragging_object();
    }
}

impl SceneManager for TaskScene {
    
    fn key_down_event(&mut self, _ctx: &mut ggez::Context, vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => {
                println!("Action1 down!");
                self.scenario_event.next_page();
            },
            _ => (),
        }
    }
    
    fn key_up_event(&mut self,
                    _ctx: &mut ggez::Context,
                    vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => println!("Action1 up!"),
            _ => (),
        }
    }

    fn mouse_motion_event(&mut self,
                          ctx: &mut ggez::Context,
                          point: numeric::Point2f,
                          offset: numeric::Vector2f) {
        if self.mouse_info.is_dragging(MouseButton::Left) {
            //println!("x: {}, y: {} ::: offset_x: {}, offset_y: {}", point.x, point.y, offset.x, offset.y);
            let d = numeric::Vector2f::new(offset.x / 2.0, offset.y / 2.0);
            self.dragging_handler(ctx, point, d);
            self.mouse_info.set_last_dragged(MouseButton::Left, point, self.get_current_clock());
        }

    }

    fn mouse_button_down_event(&mut self,
                               ctx: &mut ggez::Context,
                               button: MouseButton,
                               point: numeric::Point2f) {
        let info: &MouseActionRecord = &self.mouse_info.last_clicked.get(&button).unwrap();
        if info.point == point && (self.get_current_clock() - info.t) < 10 {
            println!("double clicked!!");
        }
        
        self.mouse_info.set_last_clicked(button, point, self.get_current_clock());
        self.mouse_info.set_last_dragged(button, point, self.get_current_clock());
        self.mouse_info.update_dragging(button, true);

        self.select_dragging_object(ctx, point);
    }
    
    fn mouse_button_up_event(&mut self,
                             _ctx: &mut ggez::Context,
                             button: MouseButton,
                             _point: numeric::Point2f) {
        self.mouse_info.update_dragging(button, false);
        self.unselect_dragging_object();
    }

    fn pre_process(&mut self, ctx: &mut ggez::Context) {
        self.desk_objects.update(ctx, self.get_current_clock());
        self.scenario_event.update_text();
        self.simulation_status.update();
    }
    
    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.desk_objects.draw(ctx).unwrap();
        self.scenario_event.draw(ctx).unwrap();
        self.simulation_status.draw(ctx).unwrap();
    }
    
    fn post_process(&mut self, _ctx: &mut ggez::Context) -> SceneTransition {
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
