use torifune::graphics::drawable::*;
use torifune::graphics::object::*;
use torifune::numeric;
use torifune::{core::*, sound::SoundPlayFlags};

use crate::core::util::read_from_resources_as_string;
use crate::{
    core::*,
    flush_delay_event, flush_delay_event_and_redraw_check,
    scene::{DelayEventList, SceneID, SceneTransition},
};

use super::{effect, util_object::FramedButton};

pub struct EndSceneFlow {
    thanks_text: EffectableWrap<MovableWrap<UniText>>,
    resul_main_vtext: EffectableWrap<MovableWrap<VerticalText>>,
    book_collection: Vec<EffectableWrap<MovableWrap<UniTexture>>>,
    result_vtext_list: Vec<EffectableWrap<MovableWrap<VerticalText>>>,
    credit_vtext_list: Vec<EffectableWrap<MovableWrap<UniText>>>,
    ok_result_button: FramedButton,
    event_list: DelayEventList<Self>,
    scene_transition: SceneTransition,
    next_scene_id: SceneID,
    drwob_essential: DrawableObjectEssential,
    flow_done: bool,
    game_cleared: bool,
}

impl EndSceneFlow {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, t: Clock) -> Self {
        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::JpFude1),
            numeric::Vector2f::new(28.0, 28.0),
            ggez::graphics::Color::from_rgba_u32(0xff),
        );

        let mut result_vtext_list = Vec::new();
        let mut credit_vtext_list = Vec::new();
        let mut book_collection = Vec::new();
        let mut pos = numeric::Point2f::new(1000.0, 90.0);

        vec![
            format!(
                "評判\n　{}",
                number_to_jk::number_to_jk(
                    ctx.take_save_data()
                        .suzunaan_status
                        .get_current_reputation() as u64
                )
            ),
            format!(
                "総収入\n　　{}円",
                number_to_jk::number_to_jk(ctx.take_save_data().task_result.total_money as u64)
            ),
            format!(
                "接客回数\n　{}回",
                number_to_jk::number_to_jk(ctx.take_save_data().award_data.customer_count as u64)
            ),
            format!(
                "貸出回数\n　{}回",
                number_to_jk::number_to_jk(ctx.take_save_data().award_data.borrowing_count as u64)
            ),
            format!(
                "返却回数\n　{}回",
                number_to_jk::number_to_jk(ctx.take_save_data().award_data.returning_count as u64)
            ),
            format!(
                "配架冊数\n　{}冊",
                number_to_jk::number_to_jk(ctx.take_save_data().award_data.shelving_count as u64)
            ),
            format!(
                "誤評価数\n　{}回",
                number_to_jk::number_to_jk(
                    ctx.take_save_data()
                        .award_data
                        .returning_check_mistake_count as u64
                )
            ),
        ]
        .into_iter()
        .for_each(|s| {
            let mut vtext = VerticalText::new(
                s,
                pos,
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                font_info.clone(),
            );

            vtext.set_alpha(0.0);

            result_vtext_list.push(EffectableWrap::new(
                MovableWrap::new(Box::new(vtext), None, t),
                Vec::new(),
            ));

            pos.x -= 100.0;
        });

        let game_cleared = match ctx.take_save_data().game_mode {
            GameMode::Story => ctx.take_save_data().task_result.total_money > 100000,
            GameMode::TimeAttack(_) => true,
        };

        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::JpFude1),
            numeric::Vector2f::new(56.0, 56.0),
            ggez::graphics::Color::from_rgba_u32(0x150808ff),
        );
        let mut result_main_vtext = VerticalText::new(
            match ctx.take_save_data().game_mode {
                GameMode::Story => {
                    if game_cleared {
                        "目標達成"
                    } else {
                        "達成失敗"
                    }
                }
                GameMode::TimeAttack(_) => "熟練\nお疲れ様でした",
            }
            .to_string(),
            numeric::Point2f::new(pos.x - 50.0, pos.y),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            font_info,
        );
        result_main_vtext.hide();

        //let credit = read_from_resources_as_string(ctx.context, "/credit.txt");
	let credit = "令和3年3月21日
第十八回博麗神社例大祭next原作
上海アリス幻樂団 東方Projectシリーズnext二次創作作品
電氣貸本屋next企画・開発・キャラクターイラスト
空き地nextBGM
YuzuSound　様
YuzuSound - Completeより
夜のおませさん
ふわふわおはなばたけ
おしまい。nextBGM
そこからは茶畑になっている　様
東方自作アレンジ集より
希望の星は青霄に昇る：arranged by aochanextSE
YuzuSound　様
YuzuSound - Completeより
ほおずきみたいに紅い魂 - あかるいピアノ
紅より儚い永遠 - おしまい？nextSE
On-Jin ～音人～
効果音ラボnextマップチップ
ぴぽやnextスペシャルサンクス
デバッグをしてくれた3人";
        for s in credit.rsplit("next").collect::<Vec<&str>>().iter().rev() {
            let mut credit_vtext = Box::new(UniText::new(
                s.to_string(),
                numeric::Point2f::new(1100.0, 100.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                FontInformation::new(
                    ctx.resource.get_font(crate::core::FontID::Mamelon),
                    numeric::Vector2f::new(30.0, 30.0),
                    ggez::graphics::Color::from_rgba_u32(0xff),
                ),
            ));
            credit_vtext.hide();

            credit_vtext.make_center(
                ctx.context,
                numeric::Point2f::new(WINDOW_SIZE_X as f32 / 2.0, WINDOW_SIZE_Y as f32 - 150.0),
            );

            credit_vtext_list.push(EffectableWrap::new(
                MovableWrap::new(credit_vtext, None, t),
                Vec::new(),
            ));
        }

        let mut ok_result_button = FramedButton::create_design1(
            ctx,
            numeric::Point2f::new(100.0, 350.0),
            "次へ",
            numeric::Vector2f::new(28.0, 28.0),
        );
        ok_result_button.hide();

        let mut thanks_text = EffectableWrap::new(
            MovableWrap::new(
                Box::new(UniText::new(
                    "プレイしてくれてありがとう！".to_string(),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    FontInformation::new(
                        ctx.resource.get_font(crate::core::FontID::Cinema),
                        numeric::Vector2f::new(40.0, 40.0),
                        ggez::graphics::Color::from_rgba_u32(0xff),
                    ),
                )),
                None,
                t,
            ),
            Vec::new(),
        );
        thanks_text.make_center(
            ctx.context,
            numeric::Point2f::new(WINDOW_SIZE_X as f32 / 2.0, WINDOW_SIZE_Y as f32 / 2.0),
        );

        thanks_text.set_alpha(0.0);

        let mut texture_pos = numeric::Point2f::new(300.0, 100.0);
        for texture_id in vec![
            TextureID::LargeBook1,
            TextureID::LargeBook2,
            TextureID::LargeBook3,
            TextureID::MiddleBook1,
            TextureID::MiddleBook2,
            TextureID::MiddleBook3,
        ] {
            let mut texture = EffectableWrap::new(
                MovableWrap::new(
                    Box::new(UniTexture::new(
                        ctx.ref_texture(texture_id),
                        texture_pos,
                        numeric::Vector2f::new(0.15, 0.15),
                        0.0,
                        0,
                    )),
                    None,
                    t,
                ),
                Vec::new(),
            );
            texture.set_alpha(0.0);
            texture.hide();

            let area = texture.get_drawing_area(ctx.context);

            book_collection.push(texture);

            texture_pos.x += area.w + 50.0;
        }

        EndSceneFlow {
            resul_main_vtext: EffectableWrap::new(
                MovableWrap::new(Box::new(result_main_vtext), None, t),
                Vec::new(),
            ),
            result_vtext_list: result_vtext_list,
            credit_vtext_list,
            drwob_essential: DrawableObjectEssential::new(true, 0),
            ok_result_button: ok_result_button,
            scene_transition: SceneTransition::Keep,
            next_scene_id: SceneID::End,
            thanks_text: thanks_text,
            book_collection: book_collection,
            flow_done: false,
            game_cleared: game_cleared,
            event_list: DelayEventList::new(),
        }
    }

    pub fn start_result(&mut self, t: Clock) {
        let mut animation_start = t + 30;
        for vtext in self.result_vtext_list.iter_mut() {
            vtext.add_effect(vec![effect::alpha_effect(40, animation_start, 0, 255)]);
            animation_start += 60;
        }

        animation_start += 100;

        self.event_list.add_event(
            Box::new(move |slf: &mut Self, _, t| {
                slf.resul_main_vtext.appear();
                slf.resul_main_vtext
                    .set_crop(numeric::Rect::new(0.0, 0.0, 1.0, 0.0));
                slf.resul_main_vtext
                    .add_effect(vec![effect::appear_bale_down_from_top(140, t)]);
            }),
            animation_start,
        );

        animation_start += 60;

        self.event_list.add_event(
            Box::new(move |slf: &mut Self, _, _| {
                slf.ok_result_button.appear();
            }),
            animation_start,
        );

        for vtext in self.credit_vtext_list.iter_mut() {
            vtext.hide();
        }
    }

    pub fn start_credit<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        let mut animation_start = t + 30;
        for vtext in self.result_vtext_list.iter_mut() {
            vtext.hide();
        }

        self.resul_main_vtext.hide();

        for vtext in self.credit_vtext_list.iter_mut() {
            vtext.appear();
            vtext.set_alpha(0.0);
            vtext.add_effect(vec![effect::alpha_effect(40, animation_start, 0, 255)]);
            animation_start += 400;
            vtext.add_effect(vec![effect::alpha_effect(40, animation_start, 255, 0)]);
            animation_start += 100;
        }

        self.event_list.add_event(
            Box::new(move |slf: &mut Self, _, t| {
                slf.thanks_text
                    .add_effect(vec![effect::alpha_effect(40, t, 0, 255)]);
                slf.flow_done = true;
            }),
            animation_start,
        );

        let mut animation_start = t + 30;
        for texture in self.book_collection.iter_mut() {
            texture.appear();
            texture.set_alpha(0.0);
            texture.add_effect(vec![effect::alpha_effect(40, animation_start, 0, 255)]);
            animation_start += 400;
        }

        ctx.play_sound_as_bgm(
            SoundID::EndBGM,
            Some(SoundPlayFlags::new(
                3000,
                1.0,
                true,
                ctx.config.get_bgm_volume(),
            )),
        );

        ctx.permanent_save_data.story_cleared();

        match ctx.take_save_data().game_mode {
            GameMode::Story => {
                ctx.permanent_save_data
                    .add_story_mode_record(HardModeRecord::new(
                        ctx.take_save_data().task_result.total_money as i64,
                    ));
            }
            GameMode::TimeAttack(_) => {
                ctx.permanent_save_data
                    .add_hard_mode_record(HardModeRecord::new(
                        ctx.take_save_data().task_result.total_money as i64,
                    ));
            }
        }

        match ctx.permanent_save_data.save() {
            Ok(_) => (),
            Err(_) => (),
        }
    }

    pub fn update<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        for vtext in self.result_vtext_list.iter_mut() {
            vtext.effect(ctx.context, t);
            vtext.move_with_func(t);
        }

        for vtext in self.credit_vtext_list.iter_mut() {
            vtext.effect(ctx.context, t);
            vtext.move_with_func(t);
        }

        for texture in self.book_collection.iter_mut() {
            texture.effect(ctx.context, t);
            texture.move_with_func(t);
        }

        self.resul_main_vtext.effect(ctx.context, t);
        self.thanks_text.effect(ctx.context, t);

        flush_delay_event_and_redraw_check!(self, self.event_list, ctx, t, {});
    }

    pub fn click_handler<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        t: Clock,
    ) {
        if self.ok_result_button.is_visible() && self.ok_result_button.contains(point) {
            self.ok_result_button.hide();

            if self.game_cleared {
                self.start_credit(ctx, t);
            } else {
                self.scene_transition = SceneTransition::SwapTransition;
                self.next_scene_id = SceneID::Title;
		std::mem::replace(ctx.savable_data, None);
            }
        }

        if self.flow_done {
            self.scene_transition = SceneTransition::SwapTransition;
            self.next_scene_id = SceneID::Title;
        }
    }

    pub fn get_scene_transition_status(&self) -> SceneTransition {
        self.scene_transition
    }
}

impl DrawableComponent for EndSceneFlow {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            for vtext in self.result_vtext_list.iter_mut() {
                vtext.draw(ctx)?;
            }

            for vtext in self.credit_vtext_list.iter_mut() {
                vtext.draw(ctx)?;
            }

            for texture in self.book_collection.iter_mut() {
                texture.draw(ctx)?;
            }

            self.ok_result_button.draw(ctx)?;
            self.thanks_text.draw(ctx)?;
            self.resul_main_vtext.draw(ctx)?;
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

    fn set_drawing_depth(&mut self, depth: i8) {
        self.drwob_essential.drawing_depth = depth;
    }

    fn get_drawing_depth(&self) -> i8 {
        self.drwob_essential.drawing_depth
    }
}
