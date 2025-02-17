use torifune::core::Clock;
use torifune::distance;
use torifune::graphics as tg;
use torifune::graphics::object::GenericMoveFn;
use torifune::numeric;

pub fn stop() -> Option<GenericMoveFn> {
    None
}

pub fn halt(pos: numeric::Point2f) -> Option<GenericMoveFn> {
    Some(Box::new(
        move |_: &dyn tg::object::MovableObject, _: Clock| Some(pos),
    ))
}

pub fn gravity_move(
    init_speed: f32,
    max_speed: f32,
    border_y: f32,
    a: f32,
) -> Option<GenericMoveFn> {
    Some(Box::new(
        move |p: &dyn tg::object::MovableObject, t: Clock| {
            let p = p.get_position();
            let next_spped = ((t as f32) * a) + init_speed;

            let speed = if next_spped < max_speed {
                next_spped
            } else {
                max_speed
            };

            let mut next = numeric::Point2f::new(p.x, p.y + (speed));
            if next.y > border_y {
                next.y = border_y;
            }

            Some(next)
        },
    ))
}

pub fn devide_distance(dest: numeric::Point2f, divide_c: f32) -> Option<GenericMoveFn> {
    Some(Box::new(
        move |p: &dyn tg::object::MovableObject, _t: Clock| {
            let current_pos = p.get_position();

            if distance!(current_pos, dest) < 1.0 {
                return Some(dest);
            }

            if dest == current_pos {
                return None;
            }

            let offset = numeric::Vector2f::new(dest.x - current_pos.x, dest.y - current_pos.y);
            Some(numeric::Point2f::new(
                current_pos.x + (offset.x * divide_c),
                current_pos.y + (offset.y * divide_c),
            ))
        },
    ))
}

pub fn move_constant(speed: numeric::Vector2f) -> Option<GenericMoveFn> {
    Some(Box::new(
        move |p: &dyn tg::object::MovableObject, _t: Clock| {
            let current_pos = p.get_position();
            Some(numeric::Point2f::new(
                current_pos.x + speed.x,
                current_pos.y + speed.y,
            ))
        },
    ))
}

pub fn move_constant_dest(
    dest: numeric::Point2f,
    speed: numeric::Vector2f,
) -> Option<GenericMoveFn> {
    Some(Box::new(
        move |p: &dyn tg::object::MovableObject, _t: Clock| {
            let current_pos = p.get_position();

            if current_pos == dest {
                return None;
            }

            let next_pos = numeric::Point2f::new(current_pos.x + speed.x, current_pos.y + speed.y);

            let current_distance = distance!(dest, current_pos);
            let next_distance = distance!(dest, next_pos);

            if next_distance < current_distance {
                Some(next_pos)
            } else {
                Some(dest)
            }
        },
    ))
}
