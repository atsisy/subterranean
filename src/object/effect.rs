use torifune::graphics::object::*;
use torifune::numeric;
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
