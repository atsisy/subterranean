use super::*;
use crate::core::*;
use crate::core::util;

pub fn create_dobj_random(ctx: &mut ggez::Context, game_data: &GameData) -> DeskObject {
    let paper_info = CopyingRequestInformation::new_random(game_data,
                                                           GensoDate::new(128, 12, 8),
                                                           GensoDate::new(128, 12, 8));

    DeskObject::new(
        Box::new(tobj::SimpleObject::new(
            tobj::MovableUniTexture::new(
                game_data.ref_texture(TextureID::select_random()),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(0.1, 0.1),
                0.0, 0, move_fn::stop(),
                0), vec![])),
        Box::new(CopyingRequestPaper::new(ctx, ggraphics::Rect::new(0.0, 0.0, 420.0, 350.0), TextureID::Paper1,
                                          &paper_info,
                                          game_data, 0)), 0)
}

pub fn create_dobj_book_random(_ctx: &mut ggez::Context, game_data: &GameData) -> DeskObject {
    let texture = *util::random_select(LARGE_BOOK_TEXTURE.iter()).unwrap();
    DeskObject::new(
        Box::new(tobj::MovableUniTexture::new(
            game_data.ref_texture(texture),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(0.1, 0.1),
            0.0, 0, move_fn::stop(),
            0)),
        Box::new(tobj::MovableUniTexture::new(
            game_data.ref_texture(texture),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(0.3, 0.3),
            0.0, 0, move_fn::stop(),
            0)), 0)
}
