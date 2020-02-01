use torifune::graphics::object::*;
use torifune::core::Clock;

pub fn fade_in(required_time: Clock, called_clock: Clock) -> GenericEffectFn {
    Box::new(move |obj: &mut dyn MovableObject, _: &ggez::Context, t: Clock| {
	let elapsed_time = t - called_clock;
	if elapsed_time < required_time {
	    obj.set_alpha(elapsed_time as f32 / required_time as f32);
	    EffectFnStatus::EffectContinue
	} else {
	    EffectFnStatus::EffectFinish
	}
    })
}

pub fn fade_out(required_time: Clock, called_clock: Clock) -> GenericEffectFn {
    Box::new(move |obj: &mut dyn MovableObject, _: &ggez::Context, t: Clock| {
	let elapsed_time = t - called_clock;
	if elapsed_time < required_time {
	    obj.set_alpha(1.0 - (elapsed_time as f32 / required_time as f32));
	    EffectFnStatus::EffectContinue
	} else {
	    EffectFnStatus::EffectFinish
	}
    })
}

pub fn appear_bale_down_from_top(required_time: Clock, called_clock: Clock) -> GenericEffectFn {
    Box::new(move |obj: &mut dyn MovableObject, _: &ggez::Context, t: Clock| {
	let elapsed_time = t - called_clock;
	if elapsed_time < required_time {
	    let mut current_crop = obj.get_crop();
	    current_crop.h = elapsed_time as f32 / required_time as f32;
	    obj.set_crop(current_crop);
	    EffectFnStatus::EffectContinue
	} else {
	    EffectFnStatus::EffectFinish
	}
    })
}

pub fn appear_bale_up_from_bottom(required_time: Clock, called_clock: Clock) -> GenericEffectFn {
    Box::new(move |obj: &mut dyn MovableObject, _: &ggez::Context, t: Clock| {
	let elapsed_time = t - called_clock;
	if elapsed_time < required_time {
	    let mut current_crop = obj.get_crop();
	    current_crop.y = elapsed_time as f32 / required_time as f32;
	    obj.set_crop(current_crop);
	    EffectFnStatus::EffectContinue
	} else {
	    EffectFnStatus::EffectFinish
	}
    })
}
