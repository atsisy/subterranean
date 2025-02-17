use torifune::core::Clock;
use torifune::graphics::object::*;

///
/// # required_time
/// アニメーションにかける時間
///
/// # start
/// アニメーションが開始する時間, 未来を指定することもできる
///
pub fn fade_in(required_time: Clock, start: Clock) -> GenericEffectFn {
    Box::new(
        move |obj: &mut dyn MovableObject, _: &ggez::Context, t: Clock| {
            if start <= t {
                let elapsed_time = t - start;
                if elapsed_time < required_time {
                    obj.set_alpha(elapsed_time as f32 / required_time as f32);
                    EffectFnStatus::EffectContinue
                } else {
                    obj.set_alpha(1.0);
                    EffectFnStatus::EffectFinish
                }
            } else {
                EffectFnStatus::EffectContinue
            }
        },
    )
}

pub fn constant_rotating(speed_rad: f32, start: Clock) -> GenericEffectFn {
    Box::new(
        move |obj: &mut dyn MovableObject, _: &ggez::Context, t: Clock| {
            if start <= t {
                let rotation_rad = obj.get_rotation();
                obj.set_rotation(rotation_rad + speed_rad);
            }
            EffectFnStatus::EffectContinue
        },
    )
}

///
/// # required_time
/// アニメーションにかける時間
///
/// # start
/// アニメーションが開始する時間, 未来を指定することもできる
///
pub fn alpha_effect(
    required_time: Clock,
    start: Clock,
    init_alpha: u8,
    fin_alpha: u8,
) -> GenericEffectFn {
    let init_ratio_alpha = init_alpha as f32 / 255.0;
    let alpha_offset = fin_alpha as i32 - init_alpha as i32;
    let diff_alpha_per_clock = alpha_offset as f32 / 255.0 / required_time as f32;

    Box::new(
        move |obj: &mut dyn MovableObject, _: &ggez::Context, t: Clock| {
            if start <= t {
                let elapsed_time = t - start;
                if elapsed_time < required_time {
                    obj.set_alpha(init_ratio_alpha + (diff_alpha_per_clock * elapsed_time as f32));
                    EffectFnStatus::EffectContinue
                } else {
                    obj.set_alpha(fin_alpha as f32 * (1.0 / 255.0));
                    EffectFnStatus::EffectFinish
                }
            } else {
                EffectFnStatus::EffectContinue
            }
        },
    )
}

///
/// # required_time
/// アニメーションにかける時間
///
/// # start
/// アニメーションが開始する時間, 未来を指定することもできる
///
pub fn fade_out(required_time: Clock, start: Clock) -> GenericEffectFn {
    Box::new(
        move |obj: &mut dyn MovableObject, _: &ggez::Context, t: Clock| {
            if start <= t {
                let elapsed_time = t - start;
                if elapsed_time <= required_time {
                    obj.set_alpha(1.0 - (elapsed_time as f32 / required_time as f32));
                    EffectFnStatus::EffectContinue
                } else {
                    obj.set_alpha(0.0);
                    EffectFnStatus::EffectFinish
                }
            } else {
                EffectFnStatus::EffectContinue
            }
        },
    )
}

pub fn appear_bale_down_from_top(required_time: Clock, called_clock: Clock) -> GenericEffectFn {
    Box::new(
        move |obj: &mut dyn MovableObject, _: &ggez::Context, t: Clock| {
            if called_clock <= t {
                let elapsed_time = t - called_clock;
                let mut current_crop = obj.get_crop();
                if elapsed_time < required_time {
                    current_crop.h = elapsed_time as f32 / required_time as f32;
                    obj.set_crop(current_crop);
                    EffectFnStatus::EffectContinue
                } else {
                    current_crop.h = 1.0;
                    obj.set_crop(current_crop);
                    EffectFnStatus::EffectFinish
                }
            } else {
                EffectFnStatus::EffectContinue
            }
        },
    )
}

pub fn appear_bale_up_from_bottom(required_time: Clock, called_clock: Clock) -> GenericEffectFn {
    Box::new(
        move |obj: &mut dyn MovableObject, _: &ggez::Context, t: Clock| {
            let elapsed_time = t - called_clock;
            if elapsed_time < required_time {
                let mut current_crop = obj.get_crop();
                current_crop.y = elapsed_time as f32 / required_time as f32;
                obj.set_crop(current_crop);
                EffectFnStatus::EffectContinue
            } else {
                EffectFnStatus::EffectFinish
            }
        },
    )
}

pub fn hide_bale_down_from_top(required_time: Clock, called_clock: Clock) -> GenericEffectFn {
    Box::new(
        move |obj: &mut dyn MovableObject, _: &ggez::Context, t: Clock| {
            if called_clock <= t {
                let elapsed_time = t - called_clock;
                let mut current_crop = obj.get_crop();
                if elapsed_time < required_time {
                    current_crop.h = 1.0 - (elapsed_time as f32 / required_time as f32);
                    obj.set_crop(current_crop);
                    EffectFnStatus::EffectContinue
                } else {
                    current_crop.h = 0.0;
                    obj.set_crop(current_crop);
                    EffectFnStatus::EffectFinish
                }
            } else {
                EffectFnStatus::EffectContinue
            }
        },
    )
}

pub fn hide_bale_up_from_bottom(required_time: Clock, called_clock: Clock) -> GenericEffectFn {
    Box::new(
        move |obj: &mut dyn MovableObject, _: &ggez::Context, t: Clock| {
            let elapsed_time = t - called_clock;
            if elapsed_time < required_time {
                let mut current_crop = obj.get_crop();
                current_crop.y = 1.0 - (elapsed_time as f32 / required_time as f32);
                obj.set_crop(current_crop);
                EffectFnStatus::EffectContinue
            } else {
                EffectFnStatus::EffectFinish
            }
        },
    )
}
