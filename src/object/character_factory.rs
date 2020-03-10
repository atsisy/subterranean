use torifune::numeric;

use super::map_object::*;
use super::*;
use crate::core::{GameData, TextureID};

fn create_playable_doremy1(
    game_data: &GameData,
    camera: &numeric::Rect,
    map_position: numeric::Point2f,
) -> MapObject {
    MapObject::new(
        tobj::SimpleObject::new(
            tobj::MovableUniTexture::new(
                game_data.ref_texture(TextureID::KosuzuDotFront1),
                mp::map_to_display(&map_position, camera),
                numeric::Vector2f::new(1.5, 1.5),
                0.0,
                0,
                None,
                0,
            ),
            vec![],
        ),
        vec![
            vec![
                game_data.ref_texture(TextureID::KosuzuDotFront2),
                game_data.ref_texture(TextureID::KosuzuDotFront3),
            ],
            vec![
                game_data.ref_texture(TextureID::KosuzuDotBack2),
                game_data.ref_texture(TextureID::KosuzuDotBack3),
            ],
            vec![
                game_data.ref_texture(TextureID::KosuzuDotRight2),
                game_data.ref_texture(TextureID::KosuzuDotRight3),
            ],
            vec![
                game_data.ref_texture(TextureID::KosuzuDotLeft2),
                game_data.ref_texture(TextureID::KosuzuDotLeft3),
            ],
        ],
        0,
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

fn create_customer_sample(
    game_data: &GameData,
    camera: &numeric::Rect,
    map_position: numeric::Point2f,
) -> MapObject {
    MapObject::new(
        tobj::SimpleObject::new(
            tobj::MovableUniTexture::new(
                game_data.ref_texture(TextureID::KosuzuDotFront1),
                mp::map_to_display(&map_position, camera),
                numeric::Vector2f::new(1.5, 1.5),
                0.0,
                0,
                None,
                0,
            ),
            vec![],
        ),
        vec![
            vec![
                game_data.ref_texture(TextureID::KosuzuDotFront2),
                game_data.ref_texture(TextureID::KosuzuDotFront3),
            ],
            vec![
                game_data.ref_texture(TextureID::KosuzuDotBack2),
                game_data.ref_texture(TextureID::KosuzuDotBack3),
            ],
            vec![
                game_data.ref_texture(TextureID::KosuzuDotRight2),
                game_data.ref_texture(TextureID::KosuzuDotRight3),
            ],
            vec![
                game_data.ref_texture(TextureID::KosuzuDotLeft2),
                game_data.ref_texture(TextureID::KosuzuDotLeft3),
            ],
        ],
        0,
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

pub fn create_character(
    order: CharacterFactoryOrder,
    game_data: &GameData,
    camera: &numeric::Rect,
    map_position: numeric::Point2f,
) -> MapObject {
    match order {
        CharacterFactoryOrder::PlayableDoremy1 => {
            create_playable_doremy1(game_data, camera, map_position)
        }
        CharacterFactoryOrder::CustomerSample => {
            create_customer_sample(game_data, camera, map_position)
        }
    }
}
