use torifune::numeric;

use crate::core::{TextureID, GameData};
use super::*;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CharacterFactoryOrder {
    PlayableDoremy1,
}

pub fn create_character(order: CharacterFactoryOrder, game_data: &GameData,
                        camera: &numeric::Rect, map_position: numeric::Point2f) -> MapObject {
    match order {
        CharacterFactoryOrder::PlayableDoremy1 => create_playable_doremy1(game_data, camera, map_position),
    }
}

fn create_playable_doremy1(game_data: &GameData, camera: &numeric::Rect,
                           map_position: numeric::Point2f) -> MapObject {
    MapObject::new(tobj::SimpleObject::new(
        tobj::MovableUniTexture::new(
            game_data.ref_texture(TextureID::LotusBlue),
            mp::map_to_display(&map_position, camera),
            numeric::Vector2f::new(0.1, 0.1),
            0.0, 0, move_fn::gravity_move(-5.0, 24.0, 600.0, 0.2),
            0), vec![]),
                   vec![vec![
                       game_data.ref_texture(TextureID::LotusPink),
                       game_data.ref_texture(TextureID::LotusBlue)
                   ]], 0,
                   TextureSpeedInfo::new(0.05, 0.08, numeric::Vector2f::new(0.0, 0.0),
                                         SpeedBorder {
                                             positive_x: 6.0,
                                             negative_x: -6.0,
                                             positive_y: 6.0,
                                             negative_y: -6.0,
                                         }), map_position,
                   5)
}
