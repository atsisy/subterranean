pub mod scenario_scene;
pub mod shop_scene;
pub mod suzuna_scene;

use std::str::FromStr;

use ggez::input as ginput;
use torifune::core::Clock;
use torifune::device as tdev;
use torifune::numeric;

use crate::core::GameData;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SceneTransition {
    Keep,
    Reset,
    SwapTransition,
    StackingTransition,
    PoppingTransition,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SceneID {
    Null,
    MainDesk,
    Scenario,
    SuzunaShop,
    DayResult,
}

impl FromStr for SceneID {
    type Err = ();

    fn from_str(scene_str: &str) -> Result<Self, Self::Err> {
        match scene_str {
            "MainDesk" => Ok(Self::MainDesk),
            "Scenario" => Ok(Self::Scenario),
            "SuzunaShop" => Ok(Self::SuzunaShop),
            "WorkResult" => Ok(Self::DayResult),
            _ => panic!("Error: EventTrigger::from_str"),
        }
    }
}

pub trait SceneManager {
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
        _vkey: tdev::VirtualKey,
    ) {
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _game_data: &GameData,
        _point: numeric::Point2f,
        _offset: numeric::Vector2f,
    ) {
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _game_data: &GameData,
        _button: ginput::mouse::MouseButton,
        _point: numeric::Point2f,
    ) {
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _game_data: &GameData,
        _button: ginput::mouse::MouseButton,
        _point: numeric::Point2f,
    ) {
    }

    fn pre_process(&mut self, ctx: &mut ggez::Context, game_data: &GameData);

    fn drawing_process(&mut self, ctx: &mut ggez::Context);
    fn post_process(&mut self, ctx: &mut ggez::Context, game_data: &GameData) -> SceneTransition;
    fn transition(&self) -> SceneID;

    fn get_current_clock(&self) -> Clock;

    fn update_current_clock(&mut self);
}

pub struct NullScene {}

impl NullScene {
    pub fn new() -> Self {
        NullScene {}
    }
}

impl SceneManager for NullScene {
    fn pre_process(&mut self, _ctx: &mut ggez::Context, _: &GameData) {}

    fn drawing_process(&mut self, _ctx: &mut ggez::Context) {}
    fn post_process(&mut self, _ctx: &mut ggez::Context, _: &GameData) -> SceneTransition {
        SceneTransition::Keep
    }

    fn transition(&self) -> SceneID {
        SceneID::Null
    }

    fn get_current_clock(&self) -> Clock {
        0
    }

    fn update_current_clock(&mut self) {}
}

///
/// # 遅延イベントを起こすための情報を保持する
///
/// ## run_time
/// 処理が走る時間
///
/// ## func
/// run_time時に実行される処理
///
pub struct DelayEvent<T> {
    pub run_time: Clock,
    pub func: Box<dyn FnOnce(&mut T, &mut ggez::Context, &GameData) -> ()>,
}

impl<T> DelayEvent<T> {
    pub fn new(f: Box<dyn FnOnce(&mut T, &mut ggez::Context, &GameData) -> ()>, t: Clock) -> Self {
        DelayEvent::<T> {
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
pub struct DelayEventList<T> {
    list: Vec<DelayEvent<T>>,
}

impl<T> DelayEventList<T> {
    pub fn new() -> Self {
        DelayEventList::<T> { list: Vec::new() }
    }

    pub fn add_event(
        &mut self,
        f: Box<dyn FnOnce(&mut T, &mut ggez::Context, &GameData) -> ()>,
        t: Clock,
    ) -> &mut Self {
        self.add(DelayEvent::new(f, t))
    }

    pub fn add(&mut self, event: DelayEvent<T>) -> &mut Self {
        self.list.push(event);
        self.list.sort_by(|o1, o2| o2.run_time.cmp(&o1.run_time));
        self
    }

    pub fn move_top(&mut self) -> Option<DelayEvent<T>> {
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
