use torifune::numeric;

use super::map_object::*;
use super::*;
use crate::core::TextureID;
use crate::object::util_object::*;

fn create_playable_doremy1<'a>(
    ctx: &mut SuzuContext<'a>,
    camera: &numeric::Rect,
    map_position: numeric::Point2f,
) -> MapObject {
    MapObject::new(
        tobj::SimpleObject::new(
            tobj::MovableUniTexture::new(
                Box::new(UniTexture::new(
                    ctx.ref_texture(TextureID::KosuzuDotFront1),
                    mp::map_to_display(&map_position, camera),
                    numeric::Vector2f::new(1.5, 1.5),
                    0.0,
                    0,
                )),
                None,
                0,
            ),
            vec![],
        ),
        vec![
            ObjectDirection::MoveDown,
            ObjectDirection::MoveUp,
            ObjectDirection::MoveRight,
            ObjectDirection::MoveLeft,
	    ObjectDirection::StopDown,
	    ObjectDirection::StopUp,
	    ObjectDirection::StopRight,
	    ObjectDirection::StopLeft,
        ],
        vec![
            vec![
                ctx.ref_texture(TextureID::KosuzuDotFront2),
                ctx.ref_texture(TextureID::KosuzuDotFront3),
            ],
            vec![
                ctx.ref_texture(TextureID::KosuzuDotBack2),
                ctx.ref_texture(TextureID::KosuzuDotBack3),
            ],
            vec![
                ctx.ref_texture(TextureID::KosuzuDotRight1),
                ctx.ref_texture(TextureID::KosuzuDotRight2),
		ctx.ref_texture(TextureID::KosuzuDotRight1),
		ctx.ref_texture(TextureID::KosuzuDotRight3),
            ],
            vec![
                ctx.ref_texture(TextureID::KosuzuDotLeft1),
                ctx.ref_texture(TextureID::KosuzuDotLeft2),
		ctx.ref_texture(TextureID::KosuzuDotLeft1),
		ctx.ref_texture(TextureID::KosuzuDotLeft3),
            ],
	    vec![
                ctx.ref_texture(TextureID::KosuzuDotFront1),
	    ],
	    vec![
                ctx.ref_texture(TextureID::KosuzuDotBack1),
	    ],
	    vec![
                ctx.ref_texture(TextureID::KosuzuDotRight1),
	    ],
	    vec![
                ctx.ref_texture(TextureID::KosuzuDotLeft1),
	    ]
        ],
        ObjectDirection::StopDown,
        TextureSpeedInfo::new(
            numeric::Vector2f::new(0.0, 0.0),
            SpeedBorder {
                positive_x: 6.0,
                negative_x: -6.0,
                positive_y: 6.0,
                negative_y: -6.0,
            },
        ),
        map_position,
        numeric::Rect::new(0.02, 0.6, 0.98, 1.0),
        15,
    )
}

fn create_customer_sample<'a>(
    ctx: &mut SuzuContext<'a>,
    camera: &numeric::Rect,
    map_position: numeric::Point2f,
) -> MapObject {
    MapObject::new(
        tobj::SimpleObject::new(
            tobj::MovableUniTexture::new(
                Box::new(UniTexture::new(
                    ctx.ref_texture(TextureID::KosuzuDotFront1),
                    mp::map_to_display(&map_position, camera),
                    numeric::Vector2f::new(1.5, 1.5),
                    0.0,
                    0,
                )),
                None,
                0,
            ),
            vec![],
        ),
        vec![
            ObjectDirection::MoveDown,
            ObjectDirection::MoveUp,
            ObjectDirection::MoveRight,
            ObjectDirection::MoveLeft,
	    ObjectDirection::StopDown,
	    ObjectDirection::StopUp,
	    ObjectDirection::StopRight,
	    ObjectDirection::StopLeft,
        ],
        vec![
            vec![
                ctx.ref_texture(TextureID::KosuzuDotFront2),
                ctx.ref_texture(TextureID::KosuzuDotFront3),
            ],
            vec![
                ctx.ref_texture(TextureID::KosuzuDotBack2),
                ctx.ref_texture(TextureID::KosuzuDotBack3),
            ],
            vec![
                ctx.ref_texture(TextureID::KosuzuDotRight1),
                ctx.ref_texture(TextureID::KosuzuDotRight2),
		ctx.ref_texture(TextureID::KosuzuDotRight1),
		ctx.ref_texture(TextureID::KosuzuDotRight3),
            ],
            vec![
                ctx.ref_texture(TextureID::KosuzuDotLeft1),
                ctx.ref_texture(TextureID::KosuzuDotLeft2),
		ctx.ref_texture(TextureID::KosuzuDotLeft1),
		ctx.ref_texture(TextureID::KosuzuDotLeft3),
            ],
	    vec![
                ctx.ref_texture(TextureID::KosuzuDotFront1),
	    ],
	    vec![
                ctx.ref_texture(TextureID::KosuzuDotBack1),
	    ],
	    vec![
                ctx.ref_texture(TextureID::KosuzuDotRight1),
	    ],
	    vec![
                ctx.ref_texture(TextureID::KosuzuDotLeft1),
	    ]
        ],
        ObjectDirection::MoveDown,
        TextureSpeedInfo::new(
            numeric::Vector2f::new(0.0, 0.0),
            SpeedBorder {
                positive_x: 6.0,
                negative_x: -6.0,
                positive_y: 6.0,
                negative_y: -6.0,
            },
        ),
        map_position,
        numeric::Rect::new(0.02, 0.6, 0.98, 1.0),
        15,
    )
}

pub fn create_endroll_sample<'a>(
    ctx: &mut SuzuContext<'a>,
    camera: &numeric::Rect,
    map_position: numeric::Point2f,
) -> MapObject {
    MapObject::new(
        tobj::SimpleObject::new(
            tobj::MovableUniTexture::new(
                Box::new(UniTexture::new(
                    ctx.ref_texture(TextureID::KosuzuDotFront1),
                    mp::map_to_display(&map_position, camera),
                    numeric::Vector2f::new(2.0, 2.0),
                    0.0,
                    0,
                )),
                None,
                0,
            ),
            vec![],
        ),
        vec![
            ObjectDirection::MoveDown,
            ObjectDirection::MoveUp,
            ObjectDirection::MoveRight,
            ObjectDirection::MoveLeft,
	    ObjectDirection::StopDown,
	    ObjectDirection::StopUp,
	    ObjectDirection::StopRight,
	    ObjectDirection::StopLeft,
        ],
        vec![
            vec![
                ctx.ref_texture(TextureID::KosuzuDotFront2),
                ctx.ref_texture(TextureID::KosuzuDotFront3),
            ],
            vec![
                ctx.ref_texture(TextureID::KosuzuDotBack2),
                ctx.ref_texture(TextureID::KosuzuDotBack3),
            ],
            vec![
                ctx.ref_texture(TextureID::KosuzuDotRight1),
                ctx.ref_texture(TextureID::KosuzuDotRight2),
		ctx.ref_texture(TextureID::KosuzuDotRight1),
		ctx.ref_texture(TextureID::KosuzuDotRight3),
            ],
            vec![
                ctx.ref_texture(TextureID::KosuzuDotLeft1),
                ctx.ref_texture(TextureID::KosuzuDotLeft2),
		ctx.ref_texture(TextureID::KosuzuDotLeft1),
		ctx.ref_texture(TextureID::KosuzuDotLeft3),
            ],
	    vec![
                ctx.ref_texture(TextureID::KosuzuDotFront1),
	    ],
	    vec![
                ctx.ref_texture(TextureID::KosuzuDotBack1),
	    ],
	    vec![
                ctx.ref_texture(TextureID::KosuzuDotRight1),
	    ],
	    vec![
                ctx.ref_texture(TextureID::KosuzuDotLeft1),
	    ]
        ],
        ObjectDirection::MoveDown,
        TextureSpeedInfo::new(
            numeric::Vector2f::new(0.0, 0.0),
            SpeedBorder {
                positive_x: 6.0,
                negative_x: -6.0,
                positive_y: 6.0,
                negative_y: -6.0,
            },
        ),
        map_position,
        numeric::Rect::new(0.02, 0.6, 0.98, 1.0),
        15,
    )
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CharacterFactoryOrder {
    PlayableDoremy1,
    CustomerSample,
}

pub fn create_character<'a>(
    order: CharacterFactoryOrder,
    ctx: &mut SuzuContext<'a>,
    camera: &numeric::Rect,
    map_position: numeric::Point2f,
) -> MapObject {
    match order {
        CharacterFactoryOrder::PlayableDoremy1 => {
            create_playable_doremy1(ctx, camera, map_position)
        }
        CharacterFactoryOrder::CustomerSample => create_customer_sample(ctx, camera, map_position),
    }
}
