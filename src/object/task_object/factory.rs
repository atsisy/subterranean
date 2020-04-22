use torifune::core::Clock;

use crate::core::util;
use crate::core::*;
use crate::object::task_object::tt_sub_component::*;
use crate::object::task_object::*;

pub fn create_dobj_book<'a>(
    ctx: &mut SuzuContext<'a>,
    obj_type: DeskObjectType,
    book_info: BookInformation,
    t: Clock,
) -> TaskItem {
    let texture = *util::random_select(LARGE_BOOK_TEXTURE.iter()).unwrap();
    TaskItem::Book(
	TaskBook::new(
            OnDeskTexture::new(
		ctx.context,
		UniTexture::new(
                    ctx.resource.ref_texture(texture),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(0.1, 0.1),
                    0.0,
                    0,
		),
		OnDeskType::Book,
            ),
	    OnDeskBook::new(
		ctx,
		texture,
		book_info
	    ),
            0,
	    true,
            obj_type,
            t,
	)
    )
}
