pub mod copy_scene;
pub mod task_result_scene;
pub mod task_scene;

use ggez::input::mouse::MouseButton;
use torifune::core::*;
use torifune::device::VirtualKey;
use torifune::numeric;

use crate::core::*;
use crate::scene::*;

use crate::scene::shop_scene::ShopScene;

use copy_scene::*;
use task_result_scene::*;
use task_scene::*;

use crate::object::task_object::tt_main_component::*;
use crate::object::task_object::tt_sub_component::*;

#[derive(PartialEq, Clone, Copy)]
pub enum SuzunaSceneStatus {
    Shop,
    DeskWork,
    DayResult,
    Copying,
}

pub struct ReturningRequestPool {
    returning_request: Vec<ReturnBookInformation>,
}

impl ReturningRequestPool {
    pub fn new(game_data: &GameData, today: GensoDate) -> Self {
        let mut returning_request = Vec::new();
        let mut return_date = today.clone();
        return_date.add_day(14);

        for _ in 0..15 {
            returning_request.push(ReturnBookInformation::new_random(
                game_data,
                today,
                return_date,
            ));
        }

        ReturningRequestPool {
            returning_request: returning_request,
        }
    }

    pub fn add_request(&mut self, borrow_info: BorrowingInformation) {
        let returning_book_info = ReturnBookInformation::new(
            borrow_info.borrowing.clone(),
            &borrow_info.borrower,
            borrow_info.borrow_date,
            borrow_info.return_date,
        );
        self.returning_request.push(returning_book_info);
    }

    pub fn select_returning_request_random(&mut self) -> Option<ReturnBookInformation> {
        let request_len = self.returning_request.len();

        if request_len == 0 {
            return None;
        }

        Some(
            self.returning_request
                .swap_remove(rand::random::<usize>() % request_len),
        )
    }
}

pub struct SuzunaBookPool {
    books: Vec<BookInformation>,
}

impl SuzunaBookPool {
    pub fn new(game_data: &GameData) -> Self {
        SuzunaBookPool {
            books: game_data.clone_available_books(),
        }
    }

    pub fn push_book(&mut self, book_info: BookInformation) {
        self.books.push(book_info);
    }

    pub fn push_book_vec(&mut self, book_info_vec: Vec<BookInformation>) {
        self.books.extend(book_info_vec);
    }

    pub fn generate_borrowing_request(
        &mut self,
        customer_name: String,
        borrow_date: GensoDate,
        return_date: GensoDate,
    ) -> BorrowingInformation {
        let mut borrowing_books = Vec::new();
        for _ in 0..(rand::random::<u32>() % 5) {
            if self.books.is_empty() {
                break;
            }

            let book_info = self
                .books
                .swap_remove(rand::random::<usize>() % self.books.len());
            borrowing_books.push(book_info);
        }

        BorrowingInformation {
            borrowing: borrowing_books,
            borrower: customer_name,
            borrow_date: borrow_date,
            return_date: return_date,
        }
    }
}

///
/// 鈴奈庵シーンのサブシーンをまとめる構造体
///
pub struct SuzunaSubScene {
    pub shop_scene: Option<Box<ShopScene>>,
    pub desk_work_scene: Option<Box<TaskScene>>,
    pub day_result_scene: Option<Box<TaskResultScene>>,
    pub copying_scene: Option<Box<CopyingScene>>,
    scene_status: SuzunaSceneStatus,
    borrowing_record_book_data: Option<BorrowingRecordBookData>,
    returning_request_pool: ReturningRequestPool,
    suzuna_book_pool: SuzunaBookPool,
}

impl SuzunaSubScene {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData, map_id: u32) -> Self {
        SuzunaSubScene {
            shop_scene: Some(Box::new(ShopScene::new(
                ctx,
                game_data,
                map_id,
                GensoDate::new_empty(),
            ))),
            desk_work_scene: None,
            day_result_scene: None,
            copying_scene: None,
            scene_status: SuzunaSceneStatus::Shop,
            borrowing_record_book_data: None,
            returning_request_pool: ReturningRequestPool::new(
                game_data,
                GensoDate::new(12, 12, 12),
            ),
            suzuna_book_pool: SuzunaBookPool::new(game_data),
        }
    }

    pub fn get_shop_scene_mut(&mut self) -> Option<&mut Box<ShopScene>> {
        self.shop_scene.as_mut()
    }

    pub fn get_deskwork_scene_mut(&mut self) -> Option<&mut Box<TaskScene>> {
        self.desk_work_scene.as_mut()
    }

    pub fn get_dayresult_scene_mut(&mut self) -> Option<&mut Box<TaskResultScene>> {
        self.day_result_scene.as_mut()
    }

    pub fn get_scene_status(&self) -> SuzunaSceneStatus {
        self.scene_status
    }

    pub fn switch_shop_to_deskwork(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        transition: SceneTransition,
    ) {
        if transition == SceneTransition::StackingTransition {
            if let Some(shop_scene) = self.shop_scene.as_mut() {
                // CustomerRequestを構築する上で必須な要素を取得
                let customer_request_hint = shop_scene.pop_customer_request();

                if customer_request_hint.is_none() {
                    return ();
                }

                // 今回のTaskSceneで扱われるCustomerRequestを構築
                let customer_request = match customer_request_hint.as_ref().unwrap() {
                    CustomerRequest::Borrowing(raw_info) => CustomerRequest::Borrowing(
                        self.suzuna_book_pool.generate_borrowing_request(
                            raw_info.borrower.to_string(),
                            raw_info.borrow_date,
                            raw_info.return_date,
                        ),
                    ),
                    CustomerRequest::Returning(_) => CustomerRequest::Returning(
                        self.returning_request_pool
                            .select_returning_request_random()
                            .unwrap(),
                    ),
                    _ => return (),
                };

                let record_book_data =
                    std::mem::replace(&mut self.borrowing_record_book_data, None);
                let today_date = shop_scene.get_today_date();

                self.scene_status = SuzunaSceneStatus::DeskWork;
                self.desk_work_scene = Some(Box::new(TaskScene::new(
                    ctx,
                    game_data,
                    today_date,
                    Some(customer_request),
                    record_book_data,
                )));
            }
        }
    }

    pub fn switch_shop_to_day_result(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        transition: SceneTransition,
    ) {
        if transition == SceneTransition::SwapTransition {
            let task_result = self.shop_scene.as_mut().unwrap().current_task_result();
            self.scene_status = SuzunaSceneStatus::DayResult;
            self.day_result_scene =
                Some(Box::new(TaskResultScene::new(ctx, game_data, task_result)));
        }
    }

    pub fn switch_deskwork_to_shop(&mut self, transition: SceneTransition) {
        if transition == SceneTransition::PoppingTransition {
            println!("switch!!!!!!!!!, deskwork -> shop");
            self.borrowing_record_book_data = Some(
                self.desk_work_scene
                    .as_ref()
                    .unwrap()
                    .export_borrowing_record_book_data(),
            );
            self.scene_status = SuzunaSceneStatus::Shop;
            self.desk_work_scene = None;
            self.shop_scene.as_mut().unwrap().switched_and_restart();
        }
    }

    pub fn switch_shop_to_copying(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        transition: SceneTransition,
    ) {
        if transition == SceneTransition::StackingTransition {
            self.scene_status = SuzunaSceneStatus::Copying;
            self.copying_scene = Some(Box::new(CopyingScene::new(ctx, game_data, Vec::new())));
        }
    }
}

impl SceneManager for SuzunaSubScene {
    fn key_down_event(&mut self, ctx: &mut ggez::Context, game_data: &GameData, vkey: VirtualKey) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene
                    .as_mut()
                    .unwrap()
                    .key_down_event(ctx, game_data, vkey);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .key_down_event(ctx, game_data, vkey);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .key_down_event(ctx, game_data, vkey);
            }
            SuzunaSceneStatus::Copying => {
                self.copying_scene
                    .as_mut()
                    .unwrap()
                    .key_down_event(ctx, game_data, vkey);
            }
        }
    }

    fn key_up_event(&mut self, ctx: &mut ggez::Context, game_data: &GameData, vkey: VirtualKey) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene
                    .as_mut()
                    .unwrap()
                    .key_up_event(ctx, game_data, vkey);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .key_up_event(ctx, game_data, vkey);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .key_up_event(ctx, game_data, vkey);
            }
            SuzunaSceneStatus::Copying => {
                self.copying_scene
                    .as_mut()
                    .unwrap()
                    .key_up_event(ctx, game_data, vkey);
            }
        }
    }

    fn mouse_motion_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        point: numeric::Point2f,
        offset: numeric::Vector2f,
    ) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene
                    .as_mut()
                    .unwrap()
                    .mouse_motion_event(ctx, game_data, point, offset);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .mouse_motion_event(ctx, game_data, point, offset);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .mouse_motion_event(ctx, game_data, point, offset);
            }
            SuzunaSceneStatus::Copying => {
                self.copying_scene
                    .as_mut()
                    .unwrap()
                    .mouse_motion_event(ctx, game_data, point, offset);
            }
        }
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_down_event(ctx, game_data, button, point);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_down_event(ctx, game_data, button, point);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_down_event(ctx, game_data, button, point);
            }
            SuzunaSceneStatus::Copying => {
                self.copying_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_down_event(ctx, game_data, button, point);
            }
        }
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_up_event(ctx, game_data, button, point);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_up_event(ctx, game_data, button, point);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_up_event(ctx, game_data, button, point);
            }
            SuzunaSceneStatus::Copying => {
                self.copying_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_up_event(ctx, game_data, button, point);
            }
        }
    }

    fn pre_process(&mut self, ctx: &mut ggez::Context, game_data: &GameData) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene
                    .as_mut()
                    .unwrap()
                    .pre_process(ctx, game_data);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .pre_process(ctx, game_data);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .pre_process(ctx, game_data);
            }
            SuzunaSceneStatus::Copying => {
                self.copying_scene
                    .as_mut()
                    .unwrap()
                    .pre_process(ctx, game_data);
            }
        }
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene.as_mut().unwrap().drawing_process(ctx);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene.as_mut().unwrap().drawing_process(ctx);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene.as_mut().unwrap().drawing_process(ctx);
            }
            SuzunaSceneStatus::Copying => {
                self.copying_scene.as_mut().unwrap().drawing_process(ctx);
            }
        }
    }

    fn post_process(&mut self, ctx: &mut ggez::Context, game_data: &GameData) -> SceneTransition {
        match self.scene_status {
            SuzunaSceneStatus::Shop => self
                .shop_scene
                .as_mut()
                .unwrap()
                .post_process(ctx, game_data),
            SuzunaSceneStatus::DeskWork => self
                .desk_work_scene
                .as_mut()
                .unwrap()
                .post_process(ctx, game_data),
            SuzunaSceneStatus::DayResult => self
                .day_result_scene
                .as_mut()
                .unwrap()
                .post_process(ctx, game_data),
            SuzunaSceneStatus::Copying => self
                .copying_scene
                .as_mut()
                .unwrap()
                .post_process(ctx, game_data),
        }
    }

    fn transition(&self) -> SceneID {
        match self.scene_status {
            SuzunaSceneStatus::Shop => self.shop_scene.as_ref().unwrap().transition(),
            SuzunaSceneStatus::DeskWork => self.desk_work_scene.as_ref().unwrap().transition(),
            SuzunaSceneStatus::DayResult => self.day_result_scene.as_ref().unwrap().transition(),
            SuzunaSceneStatus::Copying => self.copying_scene.as_ref().unwrap().transition(),
        }
    }

    fn get_current_clock(&self) -> Clock {
        match self.scene_status {
            SuzunaSceneStatus::Shop => self.shop_scene.as_ref().unwrap().get_current_clock(),
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene.as_ref().unwrap().get_current_clock()
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene.as_ref().unwrap().get_current_clock()
            }
            SuzunaSceneStatus::Copying => self.copying_scene.as_ref().unwrap().get_current_clock(),
        }
    }

    fn update_current_clock(&mut self) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => self.shop_scene.as_mut().unwrap().update_current_clock(),
            SuzunaSceneStatus::DeskWork => self
                .desk_work_scene
                .as_mut()
                .unwrap()
                .update_current_clock(),
            SuzunaSceneStatus::DayResult => self
                .day_result_scene
                .as_mut()
                .unwrap()
                .update_current_clock(),
            SuzunaSceneStatus::Copying => {
                self.copying_scene.as_mut().unwrap().update_current_clock()
            }
        }
    }
}
