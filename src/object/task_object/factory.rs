use torifune::core::Clock;

use crate::core::util;
use crate::core::*;
use crate::object::task_object::tt_sub_component::*;
use crate::object::task_object::*;

pub fn create_dobj_book<'a>(
    ctx: &mut SuzuContext<'a>,
    obj_type: DeskObjectType,
    pos: numeric::Point2f,
    book_info: BookInformation,
    t: Clock,
) -> TaskItem {
    let texture = *util::random_select(LARGE_BOOK_TEXTURE.iter()).unwrap();
    let uni_texture = UniTexture::new(
        ctx.ref_texture(texture),
        numeric::Point2f::new(0.0, 0.0),
        numeric::Vector2f::new(0.1, 0.1),
        0.0,
        0,
    );

    TaskItem::Book(TaskBook::new(
        OnDeskTexture::new(ctx.context, uni_texture, OnDeskType::Book),
        OnDeskBook::new(ctx, pos, texture, book_info),
        0,
        true,
        true,
        obj_type,
        t,
    ))
}

pub fn create_coin<'a>(
    ctx: &mut SuzuContext<'a>,
    value: u32,
    pos: numeric::Point2f,
    t: Clock,
) -> TaskItem {
    let (texture_id, scale) = match value {
        500 => (TextureID::Coin500Yen, numeric::Vector2f::new(0.11, 0.11)),
        100 => (TextureID::Coin100Yen, numeric::Vector2f::new(0.1, 0.1)),
        50 => (TextureID::Coin50Yen, numeric::Vector2f::new(0.08, 0.08)),
        _ => panic!("failed to create coin texture"),
    };

    let s_texture = UniTexture::new(
        ctx.ref_texture(texture_id),
        pos,
        scale,
        0.0,
        0,
    );

    let l_texture = UniTexture::new(
        ctx.ref_texture(texture_id),
        pos,
        scale,
        0.0,
        0,
    );

    TaskItem::Coin(TaskTexture::new(
        OnDeskTexture::new(ctx.context, s_texture, OnDeskType::Coin),
        OnDeskTexture::new(ctx.context, l_texture, OnDeskType::Coin),
        1,
        true,
        true,
        DeskObjectType::Coin,
        t,
    ))
}

pub fn create_coins<'a>(ctx: &mut SuzuContext<'a>, mut price: u32, t: Clock) -> Vec<TaskItem> {
    let mut coins = Vec::new();

    while price >= 500 {
        coins.push(create_coin(ctx, 500, numeric::Point2f::new(0.0, 0.0), t));
        price -= 500;
    }

    while price >= 100 {
        coins.push(create_coin(ctx, 100, numeric::Point2f::new(0.0, 0.0), t));
        price -= 100;
    }

    while price >= 50 {
        coins.push(create_coin(ctx, 50, numeric::Point2f::new(0.0, 0.0), t));
        price -= 50;
    }

    coins
}
