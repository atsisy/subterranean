use std::collections::VecDeque;

use ggez::graphics as ggraphics;

use torifune::distance;
use torifune::graphics::drawable::*;
use torifune::graphics::object::shape;
use torifune::graphics::object::shape::*;
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::graphics::object::*;
use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;
use torifune::numeric;

use crate::core::{SuzuContext, TextureID};

#[derive(Clone)]
struct DragDistanceCalculator {
    distance: f32,
    last: Option<numeric::Point2f>,
}

impl DragDistanceCalculator {
    pub fn new() -> Self {
        DragDistanceCalculator {
            distance: 0.0,
            last: None,
        }
    }

    pub fn add_point(&mut self, point: numeric::Point2f) {
        if self.last.is_some() {
            self.distance += distance!(self.last.unwrap(), point);
        }

        self.last = Some(point);
    }

    pub fn release(&mut self) {
        self.last = None;
    }

    pub fn get_distance(&self) -> f32 {
        self.distance
    }
}

pub struct PointerLag {
    lag_buffer: VecDeque<numeric::Point2f>,
    mesh: Option<ggraphics::Mesh>,
    drwob_essential: DrawableObjectEssential,
}

impl PointerLag {
    pub fn new() -> Self {
        PointerLag {
            lag_buffer: VecDeque::new(),
            mesh: None,
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }

    pub fn push_point(&mut self, point: numeric::Point2f) {
        self.lag_buffer.push_front(point);
        if self.lag_buffer.len() > 4 {
            self.lag_buffer.pop_back();
        }
    }

    pub fn update_mesh(&mut self, ctx: &mut ggez::Context) {
        self.mesh = if self.lag_buffer.len() > 0 {
            let mut builder = ggraphics::MeshBuilder::new();
            for point in &self.lag_buffer {
                let circle = shape::Circle::new(
                    *point,
                    60.0,
                    0.1,
                    ggraphics::DrawMode::fill(),
                    ggraphics::Color::from_rgba_u32(0x2020ff80),
                );
                circle.add_to_builder(&mut builder);
            }
            Some(builder.build(ctx).unwrap())
        } else {
            None
        }
    }
}

impl DrawableComponent for PointerLag {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            if let Some(mesh) = self.mesh.as_mut() {
                ggraphics::draw(
                    ctx,
                    mesh,
                    ggraphics::DrawParam::default()
                        .color(ggraphics::Color::from_rgba_u32(0xffffff80)),
                )
                .unwrap();
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

pub struct EffectableHangi {
    hangi_texture: UniTexture,
    canvas: SubScreen,
    drag_distance: DragDistanceCalculator,
    lag_effect: PointerLag,
}

impl EffectableHangi {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, rect: numeric::Rect) -> Self {
        EffectableHangi {
            hangi_texture: UniTexture::new(
                ctx.ref_texture(TextureID::Wood1),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                1,
            ),
            canvas: SubScreen::new(
                ctx.context,
                rect,
                0,
                ggraphics::Color::from_rgba_u32(0xffffffff),
            ),
            drag_distance: DragDistanceCalculator::new(),
            lag_effect: PointerLag::new(),
        }
    }

    pub fn dragging_handler(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        let rpoint = self.canvas.relative_point(point);
        self.drag_distance.add_point(rpoint);
        self.lag_effect.push_point(rpoint);
        self.lag_effect.update_mesh(ctx);
    }

    pub fn release_handler(&mut self, ctx: &mut ggez::Context) {
        self.drag_distance.release();
        self.lag_effect.update_mesh(ctx);
    }
}

impl DrawableComponent for EffectableHangi {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.hangi_texture.draw(ctx).unwrap();
            self.lag_effect.draw(ctx).unwrap();

            sub_screen::pop_screen(ctx);
            self.canvas.draw(ctx).unwrap();
        }
        Ok(())
    }

    fn hide(&mut self) {
        self.canvas.hide()
    }

    fn appear(&mut self) {
        self.canvas.appear();
    }

    fn is_visible(&self) -> bool {
        self.canvas.is_visible()
    }

    fn set_drawing_depth(&mut self, depth: i8) {
        self.canvas.set_drawing_depth(depth);
    }

    fn get_drawing_depth(&self) -> i8 {
        self.canvas.get_drawing_depth()
    }
}

impl DrawableObject for EffectableHangi {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for EffectableHangi {
    impl_texture_object_for_wrapped! {canvas}
}
