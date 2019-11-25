use std::collections::HashMap;
use torifune::device as tdev;
use torifune::core::Clock;
use torifune::graphics as tgraphics;
use torifune::graphics::object::*;
use tgraphics::object as tobj;
use ggez::input as ginput;
use ginput::mouse::MouseButton;
use ggez::graphics as ggraphics;
use torifune::numeric;
use torifune::hash;
use crate::core::{TextureID, GameData};
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

struct DeskObjects<'a> {
    desk_canvas: ggraphics::Canvas,
    drwob_essential: tobj::DrawableObjectEssential,
    draw_param: ggraphics::DrawParam,
    desk_objects: SimpleObjectContainer<'a>,
    dragging: Option<SimpleObject<'a>>,
}

impl<'a> DeskObjects<'a> {
    pub fn new(ctx: &mut ggez::Context, game_data: &'a GameData,
               pos: numeric::Point2f, rect: ggraphics::Rect) -> DeskObjects<'a> {

        let mut dparam = ggraphics::DrawParam::default();
        dparam.dest = pos.into();
        
        let mut desk_objects = SimpleObjectContainer::new();
        
        desk_objects.add(tobj::SimpleObject::new(
            tobj::MovableUniTexture::new(
                game_data.ref_texture(TextureID::Ghost1),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(0.1, 0.1),
                0.0, 0,  Box::new(move |p: & dyn tobj::MovableObject, t: Clock| {
                    torifune::numeric::Point2f::new(p.get_position().x + 8.0, p.get_position().y)
                }),
                0), vec![]));
        desk_objects.add(tobj::SimpleObject::new(
            tobj::MovableUniTexture::new(
                game_data.ref_texture(TextureID::LotusPink),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(0.1, 0.1),
                0.0, -1,  Box::new(move |p: & dyn tobj::MovableObject, t: Clock| {
                    torifune::numeric::Point2f::new(p.get_position().x + 8.0, p.get_position().y)
                }),
                0), vec![]));
        desk_objects.sort_with_depth();
        
        DeskObjects {
            desk_canvas: ggraphics::Canvas::new(ctx, 500, 500, ggez::conf::NumSamples::One).unwrap(),
            drwob_essential: tobj::DrawableObjectEssential::new(true, 0),
            draw_param: dparam,
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

    fn update(&mut self, ctx: &mut ggez::Context, t: Clock) {
        for p in self.desk_objects.get_raw_container_mut() {
            p.move_with_func(t);
        }
    }

}

impl<'a> tobj::DrawableObject for DeskObjects<'a> {

    fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            ggraphics::set_canvas(ctx, Some(&self.desk_canvas));
            ggraphics::clear(ctx, ggraphics::Color::from((255, 255, 255, 255)));
            ggraphics::set_screen_coordinates(ctx, ggraphics::Rect::new(0.0, 0.0, 500.0, 500.0));
            for obj in self.desk_objects.get_raw_container() {
                obj.draw(ctx);
            }
            if let Some(p) = &self.dragging {
                p.draw(ctx).unwrap();
            }
            ggraphics::set_canvas(ctx, None);
            ggraphics::set_screen_coordinates(ctx, ggraphics::Rect::new(0.0, 0.0, 1366.0, 768.0));
            ggraphics::draw(ctx, &self.desk_canvas, self.draw_param);
        }
        Ok(())
    }

    fn hide(&mut self) {
        self.drwob_essential.visible = false;
    }

    fn appear(&mut self) {
        self.drwob_essential.visible = true;
    }

    fn is_visible(&self) -> bool {
        self.drwob_essential.visible
    }

    /// 描画順序を設定する
    fn set_drawing_depth(&mut self, depth: i8) {
        self.drwob_essential.drawing_depth = depth;
    }

    /// 描画順序を返す
    fn get_drawing_depth(&self) -> i8 {
        self.drwob_essential.drawing_depth
    }

    /// 描画開始地点を設定する
    fn set_position(&mut self, pos: numeric::Point2f) {
        self.draw_param.dest = pos.into();
    }

    /// 描画開始地点を返す
    fn get_position(&self) -> numeric::Point2f {
        self.draw_param.dest.into()
    }

    /// offsetで指定しただけ描画位置を動かす
    fn move_diff(&mut self, offset: numeric::Vector2f) {
        self.draw_param.dest.x += offset.x;
        self.draw_param.dest.y += offset.y;
    }
}

pub struct TaskScene<'a> {
    desk_objects: DeskObjects<'a>,
    clock: Clock,
    mouse_info: MouseInformation,
}

impl<'a> TaskScene<'a> {
    pub fn new(ctx: &mut ggez::Context, game_data: &'a GameData) -> TaskScene<'a>  {
        
        TaskScene {
            desk_objects: DeskObjects::new(ctx, game_data,
                                           numeric::Point2f::new(150.0, 150.0),
                                           ggraphics::Rect::new(0.0, 0.0, 768.0, 768.0)),
            clock: 0,
            mouse_info: MouseInformation::new(),
        }
    }
    
    fn dragging_handler(&mut self,
                        ctx: &mut ggez::Context,
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

impl<'a> SceneManager for TaskScene<'a> {
    
    fn key_down_event(&mut self, ctx: &mut ggez::Context, vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => println!("Action1 down!"),
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
            self.mouse_info.set_last_dragged(MouseButton::Left, point, self.clock);
        }

    }

    fn mouse_button_down_event(&mut self,
                               ctx: &mut ggez::Context,
                               button: MouseButton,
                               point: numeric::Point2f) {
        let info: &MouseActionRecord = &self.mouse_info.last_clicked.get(&button).unwrap();
        if info.point == point && (self.clock - info.t) < 10 {
            println!("double clicked!!");
        }
        
        self.mouse_info.set_last_clicked(button, point, self.clock);
        self.mouse_info.set_last_dragged(button, point, self.clock);
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
        self.desk_objects.update(ctx, self.clock);
    }
    
    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.desk_objects.draw(ctx).unwrap();
    }
    
    fn post_process(&mut self, ctx: &mut ggez::Context) -> SceneTransition {
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
