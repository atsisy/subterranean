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
    pub fn new(game_data: &GameResource, today: GensoDate) -> Self {
        let mut returning_request = Vec::new();
	
        for _ in 1..=5 {
	    let mut return_date = today.clone();
	    
	    match rand::random::<u32>() % 2 {
		0 => return_date.add_day(14),
		1 => return_date.add_day(7),
		_ => (),
	    }
	    
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

    pub fn iter(&self) -> std::slice::Iter<ReturnBookInformation> {
        self.returning_request.iter()
    }
}

pub struct SuzunaBookPool {
    books: Vec<BookInformation>,
}

impl SuzunaBookPool {
    pub fn new(game_data: &GameResource) -> Self {
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
        customer_name: &str,
        borrow_date: GensoDate,
        rental_limit: RentalLimit,
    ) -> BorrowingInformation {
        let mut borrowing_books = Vec::new();
        for _ in 1..(rand::random::<u32>() % 6) {
            if self.books.is_empty() {
                break;
            }

            let book_info = self
                .books
                .swap_remove(rand::random::<usize>() % self.books.len());
            borrowing_books.push(book_info);
        }

	println!("generated books count: {}", borrowing_books.len());

        BorrowingInformation::new(borrowing_books, customer_name, borrow_date, rental_limit)
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
    date: GensoDate,
}

impl SuzunaSubScene {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, map_id: u32, game_status: SavableData) -> Self {
        let returning_pool = ReturningRequestPool::new(ctx.resource, game_status.date.clone());

        let borrowing_record_book_data = BorrowingRecordBookData {
            pages_data: returning_pool
                .iter()
                .map(|ret_info| {
                    println!("{:?}", ret_info);
                    BorrowingRecordBookPageData::from(ret_info)
                })
                .collect(),
        };

        SuzunaSubScene {
            shop_scene: Some(Box::new(ShopScene::new(ctx, map_id))),
            desk_work_scene: None,
            day_result_scene: None,
            copying_scene: None,
            scene_status: SuzunaSceneStatus::Shop,
            borrowing_record_book_data: Some(borrowing_record_book_data),
            returning_request_pool: returning_pool,
            suzuna_book_pool: SuzunaBookPool::new(ctx.resource),
            date: game_status.date,
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

    pub fn switch_shop_to_deskwork<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
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
                    CustomerRequest::Borrowing(raw_info) => {
                        let borrowing_info = self.suzuna_book_pool.generate_borrowing_request(
                            &raw_info.borrower,
                            raw_info.borrow_date,
                            raw_info.rental_limit.clone(),
                        );

                        self.returning_request_pool
                            .add_request(borrowing_info.clone());

                        CustomerRequest::Borrowing(borrowing_info)
                    }
                    CustomerRequest::Returning(_) => {
                        let request = self
                            .returning_request_pool
                            .select_returning_request_random()
                            .unwrap();
                        println!("{:?}", request);
                        CustomerRequest::Returning(request)
                    }
                    _ => return (),
                };

                let record_book_data =
                    std::mem::replace(&mut self.borrowing_record_book_data, None);

                self.scene_status = SuzunaSceneStatus::DeskWork;
                self.desk_work_scene = Some(Box::new(TaskScene::new(
                    ctx,
                    Some(customer_request),
                    record_book_data,
                )));
            }
        }
    }

    pub fn switch_shop_to_day_result<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        transition: SceneTransition,
    ) {
        if transition == SceneTransition::SwapTransition {
            self.scene_status = SuzunaSceneStatus::DayResult;
            self.day_result_scene = Some(Box::new(TaskResultScene::new(
                ctx,
                self.date.clone(),
            )));
        }
    }

    pub fn switch_deskwork_to_shop<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        transition: SceneTransition,
    ) {
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
            self.shop_scene.as_mut().unwrap().switched_and_restart(ctx);
        }
    }

    pub fn switch_shop_to_copying<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        transition: SceneTransition,
    ) {
        if transition == SceneTransition::StackingTransition {
            self.scene_status = SuzunaSceneStatus::Copying;
            self.copying_scene = Some(Box::new(CopyingScene::new(ctx, Vec::new())));
        }
    }
}

impl SceneManager for SuzunaSubScene {
    fn key_down_event<'a>(&mut self, ctx: &mut SuzuContext<'a>, vkey: VirtualKey) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene.as_mut().unwrap().key_down_event(ctx, vkey);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .key_down_event(ctx, vkey);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .key_down_event(ctx, vkey);
            }
            SuzunaSceneStatus::Copying => {
                self.copying_scene
                    .as_mut()
                    .unwrap()
                    .key_down_event(ctx, vkey);
            }
        }
    }

    fn key_up_event<'a>(&mut self, ctx: &mut SuzuContext<'a>, vkey: VirtualKey) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene.as_mut().unwrap().key_up_event(ctx, vkey);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .key_up_event(ctx, vkey);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .key_up_event(ctx, vkey);
            }
            SuzunaSceneStatus::Copying => {
                self.copying_scene.as_mut().unwrap().key_up_event(ctx, vkey);
            }
        }
    }

    fn mouse_motion_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        offset: numeric::Vector2f,
    ) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene
                    .as_mut()
                    .unwrap()
                    .mouse_motion_event(ctx, point, offset);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .mouse_motion_event(ctx, point, offset);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .mouse_motion_event(ctx, point, offset);
            }
            SuzunaSceneStatus::Copying => {
                self.copying_scene
                    .as_mut()
                    .unwrap()
                    .mouse_motion_event(ctx, point, offset);
            }
        }
    }

    fn mouse_button_down_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_down_event(ctx, button, point);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_down_event(ctx, button, point);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_down_event(ctx, button, point);
            }
            SuzunaSceneStatus::Copying => {
                self.copying_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_down_event(ctx, button, point);
            }
        }
    }

    fn mouse_button_up_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_up_event(ctx, button, point);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_up_event(ctx, button, point);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_up_event(ctx, button, point);
            }
            SuzunaSceneStatus::Copying => {
                self.copying_scene
                    .as_mut()
                    .unwrap()
                    .mouse_button_up_event(ctx, button, point);
            }
        }
    }

    fn mouse_wheel_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        x: f32,
        y: f32,
    ) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene
                    .as_mut()
                    .unwrap()
                    .mouse_wheel_event(ctx, point, x, y);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene
                    .as_mut()
                    .unwrap()
                    .mouse_wheel_event(ctx, point, x, y);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene
                    .as_mut()
                    .unwrap()
                    .mouse_wheel_event(ctx, point, x, y);
            }
            SuzunaSceneStatus::Copying => {
                self.copying_scene
                    .as_mut()
                    .unwrap()
                    .mouse_wheel_event(ctx, point, x, y);
            }
        }
    }

    fn pre_process<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        match self.scene_status {
            SuzunaSceneStatus::Shop => {
                self.shop_scene.as_mut().unwrap().pre_process(ctx);
            }
            SuzunaSceneStatus::DeskWork => {
                self.desk_work_scene.as_mut().unwrap().pre_process(ctx);
            }
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene.as_mut().unwrap().pre_process(ctx);
            }
            SuzunaSceneStatus::Copying => {
                self.copying_scene.as_mut().unwrap().pre_process(ctx);
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

    fn post_process<'a>(&mut self, ctx: &mut SuzuContext<'a>) -> SceneTransition {
        match self.scene_status {
            SuzunaSceneStatus::Shop => self.shop_scene.as_mut().unwrap().post_process(ctx),
            SuzunaSceneStatus::DeskWork => self.desk_work_scene.as_mut().unwrap().post_process(ctx),
            SuzunaSceneStatus::DayResult => {
                self.day_result_scene.as_mut().unwrap().post_process(ctx)
            }
            SuzunaSceneStatus::Copying => self.copying_scene.as_mut().unwrap().post_process(ctx),
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
