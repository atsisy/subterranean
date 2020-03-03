use cgmath;

use ggez::graphics as ggraphics;

use torifune::numeric;

pub enum CollisionType {
    Point(cgmath::Point2<f32>),
    Line(collision::Line2<f32>),
    Rect(collision::Aabb2<f32>),
}

#[derive(Clone, Copy)]
pub struct CollisionInformation {
    pub collision: bool,
    pub object1_position: Option<ggraphics::Rect>,
    pub object2_position: Option<numeric::Rect>,
    pub center_diff: Option<numeric::Vector2f>,
}

impl CollisionInformation {
    pub fn new_not_collision() -> CollisionInformation {
        CollisionInformation {
            collision: false,
            object1_position: None,
            object2_position: None,
            center_diff: None,
        }
    }

    pub fn new_collision(
        obj1: ggraphics::Rect,
        obj2: numeric::Rect,
        center_diff: numeric::Vector2f,
    ) -> CollisionInformation {
        CollisionInformation {
            collision: true,
            object1_position: Some(obj1),
            object2_position: Some(obj2),
            center_diff: Some(center_diff),
        }
    }
}
