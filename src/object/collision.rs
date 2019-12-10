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
    pub tile_position: Option<ggraphics::Rect>,
    pub player_position: Option<numeric::Point2f>,
    pub center_diff: Option<numeric::Vector2f>,
}

impl CollisionInformation {
    pub fn new_not_collision() -> CollisionInformation {
        CollisionInformation {
            collision: false,
            tile_position: None,
            player_position: None,
            center_diff: None,
        }
    }

    pub fn new_collision(tile_pos: ggraphics::Rect,
                         player_pos: numeric::Point2f,
                         center_diff: numeric::Vector2f) -> CollisionInformation {
        CollisionInformation {
            collision: true,
            tile_position: Some(tile_pos),
            player_position: Some(player_pos),
            center_diff: Some(center_diff),
        }
    }
}
