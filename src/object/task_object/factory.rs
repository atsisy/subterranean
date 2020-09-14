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
        OnDeskTexture::new(
            ctx.context,
            uni_texture,
            OnDeskType::Book,
        ),
        OnDeskBook::new(ctx, pos, texture, book_info),
        0,
        true,
        true,
        obj_type,
        t,
    ))
}
