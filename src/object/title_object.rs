use std::collections::HashMap;

use ggez::graphics as ggraphics;

use torifune::graphics::drawable::*;
use torifune::graphics::object::*;
use torifune::numeric;

use crate::core::SuzuContext;

pub struct VTextList {
    vtext_list: Vec<VerticalText>,
    normal_font: FontInformation,
    large_font: FontInformation,
    drwob_essential: DrawableObjectEssential,
}

impl VTextList {
    pub fn new(
	mut position: numeric::Point2f,
	normal_font_info: FontInformation,
	large_font_info: FontInformation,
	text_list: Vec<String>,
	padding: f32,
	drawing_depth: i8
    ) -> Self {
	let mut vtext_list = Vec::new();

	for text in text_list.iter().rev() {
	    let vtext = VerticalText::new(
		text.to_string(),
		position,
		numeric::Vector2f::new(1.0, 1.0),
		0.0,
		0,
		normal_font_info.clone()
	    );

	    vtext_list.push(vtext);
	    position.x += normal_font_info.scale.x + padding;
	}
	
	VTextList {
	    vtext_list: vtext_list,
	    normal_font: normal_font_info,
	    large_font: large_font_info,
	    drwob_essential: DrawableObjectEssential::new(true, drawing_depth),
	}
    }

    pub fn update_highlight<'a>(&mut self, ctx: &mut SuzuContext<'a>, point: numeric::Point2f) {
	for vtext in self.vtext_list.iter_mut() {
	    if vtext.contains(ctx.context, point) {
		vtext.set_color(ggraphics::Color::from_rgba_u32(0xddddddff));
	    } else {
		vtext.set_color(ggraphics::Color::from_rgba_u32(0xbbbbbbff));
	    }
	}
    }
}

impl DrawableComponent for VTextList {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
	    for vtext in self.vtext_list.iter_mut() {
		vtext.draw(ctx)?;
	    }
        }

        Ok(())
    }

    fn hide(&mut self) {
        self.drwob_essential.visible = false;
    }

    fn appear(&mut self) {
        self.drwob_essential.visible = true;
    }

    fn is_visible(&self) -> bool {
        self.drwob_essential.visible
    }

    fn set_drawing_depth(&mut self, depth: i8) {
        self.drwob_essential.drawing_depth = depth;
    }

    fn get_drawing_depth(&self) -> i8 {
        self.drwob_essential.drawing_depth
    }
}

pub enum TitleContents {
    InitialMenu(VTextList),
}

impl DrawableComponent for TitleContents {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
	match self {
	    TitleContents::InitialMenu(contents) => contents.draw(ctx),
	}
    }

    fn hide(&mut self) {
	match self {
	    TitleContents::InitialMenu(contents) => contents.hide(),
	}
    }

    fn appear(&mut self) {
	match self {
	    TitleContents::InitialMenu(contents) => contents.appear(),
	}
    }

    fn is_visible(&self) -> bool {
	match self {
	    TitleContents::InitialMenu(contents) => contents.is_visible(),
	}
    }

    fn set_drawing_depth(&mut self, depth: i8) {
	match self {
	    TitleContents::InitialMenu(contents) => contents.set_drawing_depth(depth),
	}
    }

    fn get_drawing_depth(&self) -> i8 {
	match self {
	    TitleContents::InitialMenu(contents) => contents.get_drawing_depth(),
	}
    }
}

pub struct TitleContentsSet {
    contents_set: HashMap<i32, TitleContents>,
}

impl TitleContentsSet {
    pub fn new() -> Self {
	TitleContentsSet {
	    contents_set: HashMap::new(),
	}
    }

    pub fn add(&mut self, key: i32, contents: TitleContents) -> &mut TitleContentsSet {
	self.contents_set.insert(key, contents);
	self
    }

    pub fn remove_pickup(&mut self, key: i32) -> Option<TitleContents> {
	self.contents_set.remove(&key)
    }
}
