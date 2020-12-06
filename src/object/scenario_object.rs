use std::collections::HashMap;
use std::collections::VecDeque;
use std::rc::Rc;

use ggez::input::mouse::MouseButton;
use ggez::graphics as ggraphics;

use torifune::graphics::drawable::*;
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::*;
use torifune::numeric;
use torifune::roundup2f;

use torifune::graphics::object::sub_screen::SubScreen;

use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;

use crate::core::*;
use crate::core::game_system;
use crate::object::util_object::*;
use crate::set_table_frame_cell_center;
use crate::scene::scenario_scene::ScenarioContext;

use number_to_jk::number_to_jk;

use serde::{Deserialize, Serialize};

pub struct SuzunaStatusMainPage {
    table_frame: TableFrame,
    desc_text: Vec<VerticalText>,
    reputation_text: VerticalText,
    money_text: VerticalText,
    day_text: VerticalText,
    kosuzu_level_text: VerticalText,
    drwob_essential: DrawableObjectEssential,
}

impl SuzunaStatusMainPage {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>) -> Self {
        let normal_scale_font = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(24.0, 24.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );

        let large_scale_font = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(36.0, 36.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );

        let table_frame = TableFrame::new(
            ctx.resource,
            numeric::Point2f::new(150.0, 30.0),
            TileBatchTextureID::OldStyleFrame,
            FrameData::new(vec![120.0, 220.0], vec![40.0; 3]),
            numeric::Vector2f::new(0.25, 0.25),
            0,
        );

        let mut desc_text = Vec::new();

        for (index, s) in vec!["評判", "習熟度", "所持金"].iter().enumerate() {
            let mut vtext = VerticalText::new(
                s.to_string(),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                normal_scale_font,
            );

            set_table_frame_cell_center!(
                ctx.context,
                table_frame,
                vtext,
                numeric::Vector2u::new(index as u32, 0)
            );

            desc_text.push(vtext);
        }

        let mut reputation_text = VerticalText::new(
            number_to_jk(ctx.savable_data.suzunaan_status.reputation as u64),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            normal_scale_font,
        );

        set_table_frame_cell_center!(
            ctx.context,
            table_frame,
            reputation_text,
            numeric::Vector2u::new(0, 1)
        );

        let mut money_text = VerticalText::new(
            format!(
                "{}円",
                number_to_jk(ctx.savable_data.task_result.total_money as u64)
            ),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            normal_scale_font,
        );

        set_table_frame_cell_center!(
            ctx.context,
            table_frame,
            money_text,
            numeric::Vector2u::new(2, 1)
        );

        let mut kosuzu_level_text = VerticalText::new(
            format!("{}", number_to_jk(0)),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            normal_scale_font,
        );

        set_table_frame_cell_center!(
            ctx.context,
            table_frame,
            kosuzu_level_text,
            numeric::Vector2u::new(1, 1)
        );

        SuzunaStatusMainPage {
            table_frame: table_frame,
            reputation_text: reputation_text,
            desc_text: desc_text,
            day_text: VerticalText::new(
                format!(
                    "{}月{}日",
                    number_to_jk::number_to_jk(ctx.savable_data.date.month as u64),
                    number_to_jk::number_to_jk(ctx.savable_data.date.day as u64),
                ),
                numeric::Point2f::new(600.0, 50.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                large_scale_font,
            ),
            money_text: money_text,
            kosuzu_level_text: kosuzu_level_text,
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }
}

impl DrawableComponent for SuzunaStatusMainPage {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.table_frame.draw(ctx).unwrap();
            self.day_text.draw(ctx).unwrap();

            for vtext in self.desc_text.iter_mut() {
                vtext.draw(ctx).unwrap();
            }

            self.reputation_text.draw(ctx).unwrap();
            self.money_text.draw(ctx).unwrap();

            self.kosuzu_level_text.draw(ctx).unwrap();
        }

        Ok(())
    }

    #[inline(always)]
    fn hide(&mut self) {
        self.drwob_essential.visible = false;
    }

    #[inline(always)]
    fn appear(&mut self) {
        self.drwob_essential.visible = true;
    }

    #[inline(always)]
    fn is_visible(&self) -> bool {
        self.drwob_essential.visible
    }

    #[inline(always)]
    fn set_drawing_depth(&mut self, depth: i8) {
        self.drwob_essential.drawing_depth = depth;
    }

    #[inline(always)]
    fn get_drawing_depth(&self) -> i8 {
        self.drwob_essential.drawing_depth
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub enum SuzunaAdType {
    ShopNobori,
    TownNobori,
    Chindon,
    NewsPaper,
    BunBunMaruPaper,
    AdPaper,
}

impl SuzunaAdType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "ShopNobori" => Self::ShopNobori,
            "TownNobori" => Self::TownNobori,
            "Chindon" => Self::Chindon,
            "NewsPaper" => Self::NewsPaper,
            "BunBunMaruPaper" => Self::BunBunMaruPaper,
            "AdPaper" => Self::AdPaper,
            _ => panic!("Unknown SuzunaAdType => {:?}", s),
        }
    }
}

pub struct AdEntry {
    check_box: CheckBox,
    desc_text: UniText,
    drwob_essential: DrawableObjectEssential,
}

impl AdEntry {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        pos: numeric::Point2f,
        check_box_size: numeric::Vector2f,
        default_check: bool,
        desc_text: String,
        depth: i8,
    ) -> Self {
        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(18.0, 18.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let choice_box_texture = Box::new(UniTexture::new(
            ctx.ref_texture(TextureID::ChoicePanel1),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            depth,
        ));

        AdEntry {
            check_box: CheckBox::new(
                ctx,
                numeric::Rect::new(pos.x, pos.y, check_box_size.x, check_box_size.y),
                choice_box_texture,
                default_check,
                0,
            ),
            desc_text: UniText::new(
                desc_text,
                numeric::Point2f::new(pos.x + check_box_size.x + 20.0, pos.y),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                font_info,
            ),
            drwob_essential: DrawableObjectEssential::new(true, depth),
        }
    }

    pub fn is_checked(&self) -> bool {
        self.check_box.checked_now()
    }
}

impl DrawableComponent for AdEntry {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.check_box.draw(ctx)?;
            self.desc_text.draw(ctx)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn hide(&mut self) {
        self.drwob_essential.visible = false;
    }

    #[inline(always)]
    fn appear(&mut self) {
        self.drwob_essential.visible = true;
    }

    #[inline(always)]
    fn is_visible(&self) -> bool {
        self.drwob_essential.visible
    }

    #[inline(always)]
    fn set_drawing_depth(&mut self, depth: i8) {
        self.drwob_essential.drawing_depth = depth;
    }

    #[inline(always)]
    fn get_drawing_depth(&self) -> i8 {
        self.drwob_essential.drawing_depth
    }
}

pub struct ScenarioAdPage {
    header_text: UniText,
    ad_table: HashMap<SuzunaAdType, AdEntry>,
    drwob_essential: DrawableObjectEssential,
}

impl ScenarioAdPage {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        pos: numeric::Point2f,
        area_size: numeric::Vector2f,
        depth: i8,
    ) -> Self {
        let mut ad_table = HashMap::new();

        let mut entry_pos = numeric::Point2f::new(pos.x + 70.0, pos.y + 100.0);

        for (index, (ty_str, ad_type)) in vec![
            ("チラシ", SuzunaAdType::AdPaper),
            ("ちんどん屋", SuzunaAdType::Chindon),
            ("のぼり（店前）", SuzunaAdType::ShopNobori),
            ("のぼり（里）", SuzunaAdType::TownNobori),
            ("新聞", SuzunaAdType::NewsPaper),
            ("文々。新聞", SuzunaAdType::BunBunMaruPaper),
        ]
        .iter()
        .enumerate()
        {
            let entry = AdEntry::new(
                ctx,
                entry_pos,
                numeric::Vector2f::new(32.0, 32.0),
                ctx.savable_data.get_ad_status(*ad_type),
                format!(
                    "{:　<7}{:　>4}円/日",
                    ty_str,
                    ctx.resource.get_default_ad_cost(*ad_type)
                ),
                depth,
            );

            ad_table.insert(*ad_type, entry);

            if index % 2 == 0 {
                entry_pos.x = 400.0;
            } else {
                entry_pos.x = pos.x + 70.0;
                entry_pos.y += 64.0;
            }
        }

        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(30.0, 30.0),
            ggraphics::BLACK,
        );
        let mut header_text = UniText::new(
            "鈴奈庵の宣伝広告".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            font_info,
        );

        header_text.make_center(ctx.context, numeric::Point2f::new(area_size.x / 2.0, 50.0));

        ScenarioAdPage {
            header_text: header_text,
            ad_table: ad_table,
            drwob_essential: DrawableObjectEssential::new(true, depth),
        }
    }

    pub fn click_handler<'a>(&mut self, ctx: &mut SuzuContext<'a>, click_point: numeric::Point2f) {
        for (ad_type, entry) in self.ad_table.iter_mut() {
            entry.check_box.click_handler(click_point);
            ctx.savable_data
                .change_ad_status(*ad_type, entry.is_checked());
        }
    }
}

impl DrawableComponent for ScenarioAdPage {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.header_text.draw(ctx)?;
            for (_, entry) in self.ad_table.iter_mut() {
                entry.draw(ctx)?;
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn hide(&mut self) {
        self.drwob_essential.visible = false;
    }

    #[inline(always)]
    fn appear(&mut self) {
        self.drwob_essential.visible = true;
    }

    #[inline(always)]
    fn is_visible(&self) -> bool {
        self.drwob_essential.visible
    }

    #[inline(always)]
    fn set_drawing_depth(&mut self, depth: i8) {
        self.drwob_essential.drawing_depth = depth;
    }

    #[inline(always)]
    fn get_drawing_depth(&self) -> i8 {
        self.drwob_essential.drawing_depth
    }
}


pub struct SuzunaStatusPages {
    main_page: SuzunaStatusMainPage,
    ad_page: ScenarioAdPage,
    sched_page: ScenarioSchedPage,
    current_page: usize,
}

impl SuzunaStatusPages {
    pub fn new<'a>(
	ctx: &mut SuzuContext<'a>,
	scno_ctx: &ScenarioContext,
	rect: numeric::Rect
    ) -> Self {
        SuzunaStatusPages {
	    main_page: SuzunaStatusMainPage::new(ctx),
            ad_page: ScenarioAdPage::new(
                ctx,
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(rect.w, rect.h),
                0,
            ),
	    sched_page: ScenarioSchedPage::new(
		ctx,
		scno_ctx,
		numeric::Vector2f::new(rect.w, rect.h),
		0
	    ),
            current_page: 0,
        }
    }

    pub fn draw_page(&mut self, ctx: &mut ggez::Context) {
        match self.current_page {
            0 => self.main_page.draw(ctx).unwrap(),
            1 => self.ad_page.draw(ctx).unwrap(),
	    2 => self.sched_page.draw(ctx).unwrap(),
            _ => (),
        }
    }

    fn next_page(&mut self) {
        if self.current_page >= self.page_len() - 1 {
            return;
        }

        self.current_page += 1;
    }

    fn prev_page(&mut self) {
        if self.current_page <= 0 {
            return;
        }

        self.current_page -= 1;
    }

    pub fn get_current_page_num(&self) -> usize {
        self.current_page
    }

    pub fn page_len(&self) -> usize {
        3
    }

    pub fn click_handler<'a>(&mut self, ctx: &mut SuzuContext<'a>, click_point: numeric::Point2f, button: MouseButton) {
        match self.current_page {
            0 => (),
            1 => self.ad_page.click_handler(ctx, click_point),
	    2 => self.sched_page.click_handler(ctx, click_point, button),
            _ => (),
        }
    }

    pub fn mouse_button_down<'a>(&mut self, ctx: &mut SuzuContext<'a>, click_point: numeric::Point2f, button: MouseButton) {
        match self.current_page {
            0 => (),
            1 => (),
	    2 => self.sched_page.mouse_button_down(ctx, click_point, button),
            _ => (),
        }
    }

    pub fn update<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        match self.current_page {
            0 => (),
            1 => (),
	    2 => self.sched_page.update(ctx),
            _ => (),
        }
    }

    pub fn show_schedule_page(&mut self) {
	self.current_page = 2;
    }
}

pub struct SuzunaStatusScreen {
    canvas: SubScreen,
    background: UniTexture,
    appr_frame: TileBatchFrame,
    pages: SuzunaStatusPages,
    go_left_texture: UniTexture,
    go_right_texture: UniTexture,
}

impl SuzunaStatusScreen {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
	scno_ctx: &ScenarioContext,
        rect: numeric::Rect,
        depth: i8,
    ) -> SuzunaStatusScreen {
        let mut background_texture = UniTexture::new(
            ctx.ref_texture(TextureID::TextBackground),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
        );

	background_texture.fit_scale(ctx.context, numeric::Vector2f::new(rect.w, rect.h));

	let appr_frame = TileBatchFrame::new(
            ctx.resource,
            TileBatchTextureID::TaishoStyle1,
            numeric::Rect::new(0.0, 0.0, rect.w, rect.h),
            numeric::Vector2f::new(0.75, 0.75),
            0,
        );

        let mut left = UniTexture::new(
            ctx.ref_texture(TextureID::GoNextPageLeft),
            numeric::Point2f::new(0.0, rect.h - 32.0),
            numeric::Vector2f::new(0.5, 0.5),
            0.0,
            0,
        );
        left.hide();

        let right = UniTexture::new(
            ctx.ref_texture(TextureID::GoNextPageRight),
            numeric::Point2f::new(rect.w - 32.0, rect.h - 32.0),
            numeric::Vector2f::new(0.5, 0.5),
            0.0,
            0,
        );

        SuzunaStatusScreen {
	    canvas: SubScreen::new(
                ctx.context,
		numeric::Rect::new(rect.x, rect.y, rect.w + 500.0, rect.h + 500.0),
		depth,
		ggraphics::Color::from_rgba_u32(0x0),
	    ),
            background: background_texture,
	    appr_frame: appr_frame,
            pages: SuzunaStatusPages::new(
		ctx, scno_ctx, rect
            ),
            go_left_texture: left,
            go_right_texture: right,
        }
    }

    fn check_move_page_icon_visibility(&mut self) {
        self.go_right_texture.appear();
        self.go_left_texture.appear();

        if self.pages.get_current_page_num() == 0 {
            self.go_left_texture.hide();
        } else if self.pages.get_current_page_num() == self.pages.page_len() - 1 {
            self.go_right_texture.hide();
        }
    }

    pub fn click_handler<'a>(&mut self, ctx: &mut SuzuContext<'a>, click_point: numeric::Point2f, button: MouseButton) {
        if !self.canvas.contains(click_point) {
            return;
        }

        let rpoint = self.canvas.relative_point(click_point);

        if self.go_right_texture.contains(ctx.context, rpoint) {
            self.pages.next_page();
            self.check_move_page_icon_visibility();
            ctx.process_utility.redraw();
        } else if self.go_left_texture.contains(ctx.context, rpoint) {
            self.pages.prev_page();
            self.check_move_page_icon_visibility();
            ctx.process_utility.redraw();
        }

        self.pages.click_handler(ctx, rpoint, button);
    }

    pub fn mouse_down_handler<'a>(&mut self, ctx: &mut SuzuContext<'a>, click_point: numeric::Point2f, button: MouseButton) {
	if !self.canvas.contains(click_point) {
            return;
        }

        let rpoint = self.canvas.relative_point(click_point);
	self.pages.mouse_button_down(ctx, rpoint, button);
    }
    
    pub fn update<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
	self.pages.update(ctx);
    }

    pub fn show_schedule_page(&mut self) {
	self.pages.show_schedule_page();
    }
}

impl DrawableComponent for SuzunaStatusScreen {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
	    sub_screen::stack_screen(ctx, &self.canvas);
	    
            self.background.draw(ctx)?;
	    self.appr_frame.draw(ctx)?;

            self.go_right_texture.draw(ctx)?;
            self.go_left_texture.draw(ctx)?;

	    self.pages.draw_page(ctx);

	    sub_screen::pop_screen(ctx);
	    self.canvas.draw(ctx).unwrap();
        }

        Ok(())
    }

    #[inline(always)]
    fn hide(&mut self) {
        self.canvas.hide()
    }

    #[inline(always)]
    fn appear(&mut self) {
        self.canvas.appear()
    }

    #[inline(always)]
    fn is_visible(&self) -> bool {
        self.canvas.is_visible()
    }

    #[inline(always)]
    fn set_drawing_depth(&mut self, depth: i8) {
        self.canvas.set_drawing_depth(depth)
    }

    #[inline(always)]
    fn get_drawing_depth(&self) -> i8 {
        self.canvas.get_drawing_depth()
    }
}

pub trait StackableWindow: TextureObject {
    fn stacked_handler<'a>(&mut self, _ctx: &mut SuzuContext<'a>) {}

    fn close_check(&self) -> bool {
	false
    }
}

pub trait StackMessagePassingWindow<Msg>: StackableWindow {
    fn check_message(&self) -> Option<Msg> {
	None
    }

    fn apply_message<'a>(&mut self, _ctx: &mut SuzuContext<'a>, _msg: Msg) {
    }

    fn mouse_down_handler<'a>(
	&mut self,
	_ctx: &mut SuzuContext<'a>,
	_point: numeric::Point2f,
	_button: MouseButton
    ) -> Option<Box<dyn StackMessagePassingWindow<Msg>>> {
	None
    }
    
    fn mouse_click_handler<'a>(
	&mut self,
	_ctx: &mut SuzuContext<'a>,
	_point: numeric::Point2f,
	_button: MouseButton
    ) -> Option<Box<dyn StackMessagePassingWindow<Msg>>> {
	None
    }
}

pub enum WeekScheduleMessage {
    DetermineDaySchedule(game_system::DayWorkType),
}

pub struct WindowStack<Msg> {
    stack: VecDeque<Box<dyn StackMessagePassingWindow<Msg>>>,
    drwob_essential: DrawableObjectEssential,
}

impl<Msg> WindowStack<Msg> {
    pub fn new(depth: i8) -> Self {
	WindowStack {
	    stack: VecDeque::new(),
	    drwob_essential: DrawableObjectEssential::new(true, depth),
	}
    }

    pub fn push<'a>(&mut self, ctx: &mut SuzuContext<'a>, new_window: Box<dyn StackMessagePassingWindow<Msg>>) {
	if let Some(window) = self.stack.front_mut() {
	    window.stacked_handler(ctx);
	}
	self.stack.push_front(new_window);
    }

    pub fn pop(&mut self) -> Option<Box<dyn StackMessagePassingWindow<Msg>>> {
	self.stack.pop_front()
    }

    pub fn mouse_down_handler<'a>(&mut self, ctx: &mut SuzuContext<'a>, point: numeric::Point2f, button: MouseButton) {
	if let Some(window) = self.stack.front_mut() {
	    let new_window = window.mouse_down_handler(ctx, point, button);
	    if new_window.is_none() {
		return;
	    }

	    self.push(ctx, new_window.unwrap());
	}
    }
    
    pub fn mouse_click_handler<'a>(&mut self, ctx: &mut SuzuContext<'a>, point: numeric::Point2f, button: MouseButton) {
	if let Some(window) = self.stack.front_mut() {
	    let new_window = window.mouse_click_handler(ctx, point, button);
	    if new_window.is_none() {
		return;
	    }

	    self.push(ctx, new_window.unwrap());
	}
    }

    pub fn check_outofclick_hide<'a>(&mut self, ctx: &mut SuzuContext<'a>, p: numeric::Point2f, protect_index: usize) {
	let len = self.stack.len();
	let check_nums = len - protect_index;

	for _ in 0..check_nums {
	    if self.stack.front().as_ref().unwrap().contains(ctx.context, p) {
		break;
	    }

	    self.stack.pop_front();
	}
    }

    pub fn message_passing<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
	let mut msg = None;
	for window in self.stack.iter_mut() {
	    if let Some(msg) = msg {
		window.apply_message(ctx, msg);
	    }
	    msg = window.check_message();
	}

	self.stack.retain(|win| !win.close_check());
    }
}

impl<Msg> DrawableComponent for WindowStack<Msg> {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
	    for window in self.stack.iter_mut().rev() {
		window.draw(ctx)?;
	    }
        }

        Ok(())
    }

    #[inline(always)]
    fn hide(&mut self) {
        self.drwob_essential.visible = false;
    }

    #[inline(always)]
    fn appear(&mut self) {
        self.drwob_essential.visible = true;
    }

    #[inline(always)]
    fn is_visible(&self) -> bool {
        self.drwob_essential.visible
    }

    #[inline(always)]
    fn set_drawing_depth(&mut self, depth: i8) {
        self.drwob_essential.drawing_depth = depth;
    }

    #[inline(always)]
    fn get_drawing_depth(&self) -> i8 {
        self.drwob_essential.drawing_depth
    }
}

pub struct WeekScheduleWindow {
    canvas: SubScreen,
    frame: TableFrame,
    current_mark: TileBatchFrame,
    background: UniTexture,
    desc_vtext: Vec<VerticalText>,
    sched_vtext: [Option<VerticalText>; 7],
    week_sched: [Option<game_system::DayWorkType>; 7],
    last_clicked: u32,
    ok_button: SelectButton,
}

impl WeekScheduleWindow {
    pub fn new<'a>(
	ctx: &mut SuzuContext<'a>,
	scno_ctx: &ScenarioContext,
	pos: numeric::Point2f,
	depth: i8
    ) -> Self {
	let frame = TableFrame::new(
            ctx.resource,
            numeric::Point2f::new(25.0, 25.0),
            TileBatchTextureID::OldStyleFrame,
            FrameData::new(vec![110.0, 110.0], vec![56.0; 7]),
            numeric::Vector2f::new(0.3, 0.3),
            0,
	);
	let frame_area = frame.get_area();

	let font_info = FontInformation::new(
	    ctx.resource.get_font(FontID::Cinema),
	    numeric::Vector2f::new(24.0, 24.0),
	    ggraphics::Color::from_rgba_u32(0xff),
	);

	let mut desc_text = Vec::new();
	
	for (index, s) in vec!["日", "月", "火", "水", "木", "金", "土"].iter().enumerate() {
	    let mut vtext = VerticalText::new(
		s.to_string(),
		numeric::Point2f::new(0.0, 0.0),
		numeric::Vector2f::new(1.0, 1.0),
		0.0,
		0,
		font_info.clone()
	    );

            set_table_frame_cell_center!(
		ctx.context,
		frame,
		vtext,
		numeric::Vector2u::new(index as u32, 0)
            );

	    desc_text.push(vtext);
	}
	
	let mut sched_vtext = [None, None, None, None, None, None, None];
	let mut week_sched = [None, None, None, None, None, None, None];
	for i in 0..7 {
	    let day_work_type = ctx.savable_data.week_schedule.get_schedule_at(i);
	    if day_work_type.is_none() {
		continue;
	    }
	    
	    week_sched[i] = day_work_type;

	    let mut vtext = VerticalText::new(
		day_work_type.unwrap().to_string_jp(),
		numeric::Point2f::new(0.0, 0.0),
		numeric::Vector2f::new(1.0, 1.0),
		0.0,
		0,
		font_info.clone()
	    );
	    
            set_table_frame_cell_center!(
		ctx.context,
		frame,
		vtext,
		numeric::Vector2u::new(i as u32, 1)
            );
	    
	    sched_vtext[i] = Some(vtext);
	}

	let background = UniTexture::new(
	    ctx.ref_texture(TextureID::TextBackground),
	    numeric::Point2f::new(0.0, 0.0),
	    numeric::Vector2f::new(1.0, 1.0),
	    0.0,
	    0
	);

	let button_texture = Box::new(TextButtonTexture::new(
            ctx,
            numeric::Point2f::new(0.0, 0.0),
            "決定".to_string(),
            FontInformation::new(
                ctx.resource.get_font(FontID::Cinema),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0xf6e1d5ff),
            ),
            5.0,
            ggraphics::Color::from_rgba_u32(0x5a4f3fff),
            0,
        ));
	let mut ok_button = SelectButton::new(ctx, numeric::Rect::new(350.0, 280.0, 120.0, 60.0), button_texture);
	if !scno_ctx.schedule_redefine {
	    ok_button.hide();
	}

	let date_diff = ctx.savable_data.week_schedule.get_first_day().diff_day(&ctx.savable_data.date);
	let p = frame.get_grid_topleft(numeric::Vector2u::new(date_diff.abs() as u32, 0), numeric::Vector2f::new(0.0, 0.0));
	let cell_size = frame.get_cell_size(numeric::Vector2u::new(date_diff.abs() as u32, 0));
	let frame_height = frame.get_area().h;
	
	let current_mark = TileBatchFrame::new(
	    ctx.resource,
	    TileBatchTextureID::RedOldStyleFrame,
	    numeric::Rect::new(p.x + 8.0, p.y + 8.0, cell_size.x + 32.0, frame_height + 32.0),
	    numeric::Vector2f::new(0.5, 0.5),
	    0
	);
	
	WeekScheduleWindow {
	    canvas: SubScreen::new(
		ctx.context,
		numeric::Rect::new(pos.x, pos.y, frame_area.w + 50.0, frame_area.h + 100.0),
		depth,
		ggraphics::Color::from_rgba_u32(0)
	    ),
	    frame: frame,
	    current_mark: current_mark,
	    background: background,
	    desc_vtext: desc_text,
	    sched_vtext: sched_vtext,
	    week_sched: week_sched,
	    last_clicked: 0,
	    ok_button: ok_button,
	}
    }

    pub fn all_day_sched_determined(&self) -> bool {
	for ty in self.week_sched.iter() {
	    if ty.is_none() {
		return false;
	    }
	}

	true
    }

    pub fn export_week_sched(&self, first_day: GensoDate) -> Option<game_system::WeekWorkSchedule> {
	if !self.all_day_sched_determined() {
	    return None;
	}
	
	let schedule = [
	    self.week_sched[0].as_ref().unwrap().clone(),
	    self.week_sched[1].as_ref().unwrap().clone(),
	    self.week_sched[2].as_ref().unwrap().clone(),
	    self.week_sched[3].as_ref().unwrap().clone(),
	    self.week_sched[4].as_ref().unwrap().clone(),
	    self.week_sched[5].as_ref().unwrap().clone(),
	    self.week_sched[6].as_ref().unwrap().clone(),
	];
	
	Some(game_system::WeekWorkSchedule::new(first_day, schedule))
    }
}


impl DrawableComponent for WeekScheduleWindow {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
	if self.is_visible() {
	    sub_screen::stack_screen(ctx, &self.canvas);

	    self.background.draw(ctx)?;
	    self.frame.draw(ctx)?;
	    self.current_mark.draw(ctx)?;

	    for vtext in self.desc_vtext.iter_mut() {
		vtext.draw(ctx)?;
	    }

	    for maybe_vtext in self.sched_vtext.iter_mut() {
		if let Some(vtext) = maybe_vtext {
		    vtext.draw(ctx)?;
		}
	    }

	    self.ok_button.draw(ctx)?;

            sub_screen::pop_screen(ctx);
            self.canvas.draw(ctx).unwrap();
	}

	Ok(())
    }

    #[inline(always)]
    fn hide(&mut self) {
        self.canvas.hide()
    }

    #[inline(always)]
    fn appear(&mut self) {
        self.canvas.appear()
    }

    #[inline(always)]
    fn is_visible(&self) -> bool {
        self.canvas.is_visible()
    }

    #[inline(always)]
    fn set_drawing_depth(&mut self, depth: i8) {
        self.canvas.set_drawing_depth(depth)
    }

    #[inline(always)]
    fn get_drawing_depth(&self) -> i8 {
        self.canvas.get_drawing_depth()
    }
}


impl DrawableObject for WeekScheduleWindow {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for WeekScheduleWindow {
    impl_texture_object_for_wrapped! {canvas}
}

impl StackableWindow for WeekScheduleWindow {
    fn stacked_handler<'a>(&mut self, _ctx: &mut SuzuContext<'a>) {
    }
}

impl StackMessagePassingWindow<WeekScheduleMessage> for WeekScheduleWindow {
    fn check_message(&self) -> Option<WeekScheduleMessage> {
	None
    }

    fn apply_message<'a>(&mut self, ctx: &mut SuzuContext<'a>, msg: WeekScheduleMessage) {
	match msg {
	    WeekScheduleMessage::DetermineDaySchedule(work_type) => {
		let font_info = FontInformation::new(
		    ctx.resource.get_font(FontID::Cinema),
		    numeric::Vector2f::new(24.0, 24.0),
		    ggraphics::Color::from_rgba_u32(0xff),
		);

		let mut vtext = VerticalText::new(
		    work_type.to_string_jp(),
		    numeric::Point2f::new(0.0, 0.0),
		    numeric::Vector2f::new(1.0, 1.0),
		    0.0,
		    0,
		    font_info
		);

		set_table_frame_cell_center!(
		    ctx.context,
		    self.frame,
		    vtext,
		    numeric::Vector2u::new(self.last_clicked, 1)
		);
		
		self.sched_vtext[self.last_clicked as usize] = Some(vtext);
		self.week_sched[self.last_clicked as usize] = Some(work_type);
	    }
	}
    }

    fn mouse_down_handler<'a>(
	&mut self,
	_ctx: &mut SuzuContext<'a>,
	_point: numeric::Point2f,
	_button: MouseButton
    ) -> Option<Box<dyn StackMessagePassingWindow<WeekScheduleMessage>>> {
	None
    }
    
    fn mouse_click_handler<'a>(
	&mut self,
	ctx: &mut SuzuContext<'a>,
	point: numeric::Point2f,
	_button: MouseButton
    ) -> Option<Box<dyn StackMessagePassingWindow<WeekScheduleMessage>>> {
	if ctx.holding_week_schedule_is_available() {
	    return None;
	}
	
	let rpoint = self.canvas.relative_point(point);
	let maybe_grid_position = self.frame.get_grid_position(ctx.context, rpoint);
        if let Some(grid_position) = maybe_grid_position {
	    if grid_position.x >= 7 {
		return None;
	    }

	    self.last_clicked = grid_position.x;

	    return Some(
		Box::new(
		    ScheduleSelectWindow::new(
			ctx,
			point,
			self.canvas.get_drawing_depth(),
		    )
		)
	    );
	}

	if self.ok_button.contains(ctx.context, rpoint) {
	    let date = ctx.savable_data.date.clone();
	    if let Some(sched) = self.export_week_sched(date) {
		ctx.savable_data.update_week_schedule(sched);
		self.ok_button.hide();
	    }
	}

	None
    }
}

pub struct ScheduleSelectWindow {
    canvas: SubScreen,
    frame: TableFrame,
    background: UniTexture,
    candidate_vtext: Vec<VerticalText>,
    selected_schedule: Option<game_system::DayWorkType>,
}

impl ScheduleSelectWindow {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, pos: numeric::Point2f, depth: i8) -> Self {
	let frame = TableFrame::new(
            ctx.resource,
            numeric::Point2f::new(25.0, 25.0),
            TileBatchTextureID::OldStyleFrame,
            FrameData::new(vec![220.0], vec![56.0; 3]),
            numeric::Vector2f::new(0.3, 0.3),
            0,
	);
	let frame_area = frame.get_area();

	let font_info = FontInformation::new(
	    ctx.resource.get_font(FontID::Cinema),
	    numeric::Vector2f::new(24.0, 24.0),
	    ggraphics::Color::from_rgba_u32(0xff),
	);

	let mut candidate_vtext = Vec::new();
	for (index, s) in vec!["店番", "外出", "家で休む"].iter().enumerate() {
	    let mut vtext = VerticalText::new(
		s.to_string(),
		numeric::Point2f::new(0.0, 0.0),
		numeric::Vector2f::new(1.0, 1.0),
		0.0,
		0,
		font_info.clone()
	    );

            set_table_frame_cell_center!(
		ctx.context,
		frame,
		vtext,
		numeric::Vector2u::new(index as u32, 0)
            );

	    candidate_vtext.push(vtext);
	}

	let background = UniTexture::new(
	    ctx.ref_texture(TextureID::TextBackground),
	    numeric::Point2f::new(0.0, 0.0),
	    numeric::Vector2f::new(1.0, 1.0),
	    0.0,
	    0
	);

	let canvas = SubScreen::new(
	    ctx.context,
	    numeric::Rect::new(pos.x, pos.y, frame_area.w + 50.0, frame_area.h + 50.0),
	    depth,
	    ggraphics::Color::from_rgba_u32(0),
	);
	
	ScheduleSelectWindow {
	    canvas: canvas,
	    frame: frame,
	    background: background,
	    candidate_vtext: candidate_vtext,
	    selected_schedule: None,
	}
    }
}

impl DrawableComponent for ScheduleSelectWindow {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
	if self.is_visible() {
	    sub_screen::stack_screen(ctx, &self.canvas);

	    self.background.draw(ctx)?;
	    self.frame.draw(ctx)?;

	    for vtext in self.candidate_vtext.iter_mut() {
		vtext.draw(ctx)?;
	    }
	    
            sub_screen::pop_screen(ctx);
            self.canvas.draw(ctx).unwrap();
	}

	Ok(())
    }

    #[inline(always)]
    fn hide(&mut self) {
        self.canvas.hide()
    }

    #[inline(always)]
    fn appear(&mut self) {
        self.canvas.appear()
    }

    #[inline(always)]
    fn is_visible(&self) -> bool {
        self.canvas.is_visible()
    }

    #[inline(always)]
    fn set_drawing_depth(&mut self, depth: i8) {
        self.canvas.set_drawing_depth(depth)
    }

    #[inline(always)]
    fn get_drawing_depth(&self) -> i8 {
        self.canvas.get_drawing_depth()
    }
}

impl DrawableObject for ScheduleSelectWindow {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for ScheduleSelectWindow {
    impl_texture_object_for_wrapped! {canvas}
}

impl StackableWindow for ScheduleSelectWindow {
    fn stacked_handler<'a>(&mut self, _ctx: &mut SuzuContext<'a>) {}

    fn close_check(&self) -> bool {
	self.selected_schedule.is_some()
    }
}

impl StackMessagePassingWindow<WeekScheduleMessage> for ScheduleSelectWindow {
    fn check_message(&self) -> Option<WeekScheduleMessage> {
	if let Some(work_type) = self.selected_schedule.as_ref() {
	    Some(WeekScheduleMessage::DetermineDaySchedule(work_type.clone()))
	} else {
	    None
	}
    }
    
    fn mouse_down_handler<'a>(
	&mut self,
	_ctx: &mut SuzuContext<'a>,
	_point: numeric::Point2f,
	_button: MouseButton
    ) -> Option<Box<dyn StackMessagePassingWindow<WeekScheduleMessage>>> {
	None
    }

    fn mouse_click_handler<'a>(
	&mut self,
	ctx: &mut SuzuContext<'a>,
	point: numeric::Point2f,
	_button: MouseButton
    ) -> Option<Box<dyn StackMessagePassingWindow<WeekScheduleMessage>>> {
	let rpoint = self.canvas.relative_point(point);
	
	let maybe_grid_position = self.frame.get_grid_position(ctx.context, rpoint);
        if let Some(grid_position) = maybe_grid_position {
	    self.selected_schedule = match grid_position.x {
		0 => Some(game_system::DayWorkType::ShopWork),
		1 => Some(game_system::DayWorkType::GoingOut(game_system::GoingOutEvent::AkyuTei)),
		2 => Some(game_system::DayWorkType::TakingRest),
		_ => None,
	    }
        }
	
	None
    }
}

pub struct ScenarioSchedPage {
    header_text: UniText,
    window_stack: WindowStack<WeekScheduleMessage>,
    drwob_essential: DrawableObjectEssential,
}

impl ScenarioSchedPage {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
	scno_ctx: &ScenarioContext,
        area_size: numeric::Vector2f,
        depth: i8,
    ) -> Self {
        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(30.0, 30.0),
            ggraphics::BLACK,
        );
        let mut header_text = UniText::new(
            "鈴奈庵店番計画表".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            font_info,
        );

        header_text.make_center(ctx.context, numeric::Point2f::new(area_size.x / 2.0, 50.0));

	let mut window_stack = WindowStack::new(0);
	let window = Box::new(
	    WeekScheduleWindow::new(
		ctx,
		scno_ctx,
		numeric::Point2f::new(100.0, 100.0),
		0
	    )
	);
	window_stack.push(
	    ctx,
	    window,
	);
	
        ScenarioSchedPage {
            header_text: header_text,
	    window_stack: window_stack,
            drwob_essential: DrawableObjectEssential::new(true, depth),
        }
    }

    pub fn click_handler<'a>(&mut self, ctx: &mut SuzuContext<'a>, click_point: numeric::Point2f, button: MouseButton) {
	self.window_stack.mouse_click_handler(ctx, click_point, button);
	self.window_stack.check_outofclick_hide(ctx, click_point, 1);
    }

    pub fn mouse_button_down<'a>(&mut self, ctx: &mut SuzuContext<'a>, click_point: numeric::Point2f, button: MouseButton) {
	self.window_stack.mouse_down_handler(ctx, click_point, button);
    }

    pub fn update<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
	self.window_stack.message_passing(ctx);
    }
}

impl DrawableComponent for ScenarioSchedPage {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.header_text.draw(ctx)?;
	    self.window_stack.draw(ctx)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn hide(&mut self) {
        self.drwob_essential.visible = false;
    }

    #[inline(always)]
    fn appear(&mut self) {
        self.drwob_essential.visible = true;
    }

    #[inline(always)]
    fn is_visible(&self) -> bool {
        self.drwob_essential.visible
    }

    #[inline(always)]
    fn set_drawing_depth(&mut self, depth: i8) {
        self.drwob_essential.drawing_depth = depth;
    }

    #[inline(always)]
    fn get_drawing_depth(&self) -> i8 {
        self.drwob_essential.drawing_depth
    }
}
