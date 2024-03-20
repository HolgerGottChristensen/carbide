use carbide::color::Color;
use carbide_core::draw::Rect;
use crate::color::WHITE;
use crate::draw::{InnerImageContext, Position, DrawStyle, ImageId};
use crate::draw::shape::triangle::Triangle;

use crate::render::CarbideTransform;

use crate::text::{InnerTextContext, TextId};
use crate::widget::FilterId;

pub struct RenderContext<'a> {
    pub render: &'a mut dyn InnerRenderContext,
    pub text: &'a mut dyn InnerTextContext,
    pub image: &'a mut dyn InnerImageContext,
}

impl<'a> RenderContext<'a> {

    // TODO: Change BasicLayouter to something more suitable
    pub fn transform<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, transform: CarbideTransform, f: F) -> R {
        self.render.transform(transform);
        let res = f(self);
        self.render.pop_transform();
        res
    }

    pub fn clip<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, bounding_box: Rect, f: F) -> R {
        self.render.clip(bounding_box);
        let res = f(self);
        self.render.pop_clip();
        res
    }

    pub fn filter<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, id: FilterId, bounding_box: Rect, f: F) -> R {
        let res = f(self);
        self.render.filter(id, bounding_box);
        res
    }

    pub fn filter2d<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, id1: FilterId, bounding_box1: Rect, id2: FilterId, bounding_box2: Rect, f: F) -> R {
        let res = f(self);
        self.render.filter2d(id1, bounding_box1, id2, bounding_box2);
        res
    }

    pub fn filter_new<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, id: FilterId, f: F) -> R {
        self.render.filter_new();
        let res = f(self);
        self.render.filter_new_pop(id, WHITE);
        res
    }

    pub fn shadow<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, id: FilterId, color: Color, f: F) -> R {
        self.render.filter_new();
        let res = f(self);
        self.render.filter_new_pop(id, color);
        res
    }

    pub fn stencil<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, geometry: &[Triangle<Position>], f: F) -> R {
        self.render.stencil(geometry);
        let res = f(self);
        self.render.pop_stencil();
        res
    }

    /// Renders the geometry with the current style
    pub fn geometry(&mut self, geometry: &[Triangle<Position>]) {
        if geometry.is_empty() {
            return;
        }

        self.render.geometry(geometry);
    }

    pub fn rect(&mut self, rect: Rect) {
        self.geometry(&[
            Triangle([
                rect.position,
                Position::new(rect.position.x + rect.width(), rect.position.y),
                Position::new(rect.position.x, rect.position.y + rect.height()),
            ]),
            Triangle([
                Position::new(rect.position.x + rect.width(), rect.position.y),
                Position::new(rect.position.x + rect.width(), rect.position.y + rect.height()),
                Position::new(rect.position.x, rect.position.y + rect.height()),
            ]),
        ])
    }

    pub fn style<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, style: DrawStyle, f: F) -> R {
        self.render.style(style);
        let res = f(self);
        self.render.pop_style();
        res
    }

    pub fn image(&mut self, id: ImageId, bounding_box: Rect, source_rect: Rect, mode: u32) {
        self.render.image(id, bounding_box, source_rect, mode);
    }

    pub fn text(&mut self, text: TextId) {
        self.render.text(text, self.text);
    }

    pub fn layer<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, layer: u32, f: F) -> R {
        self.render.layer(layer);
        let res = f(self);
        self.render.pop_layer();
        res
    }
}

pub trait InnerRenderContext {
    fn transform(&mut self, transform: CarbideTransform);
    fn pop_transform(&mut self);

    fn clip(&mut self, bounding_box: Rect);
    fn pop_clip(&mut self);

    fn filter(&mut self, id: FilterId, bounding_box: Rect);
    fn filter2d(&mut self, id1: FilterId, bounding_box1: Rect, id2: FilterId, bounding_box2: Rect);

    fn stencil(&mut self, geometry: &[Triangle<Position>]);
    fn pop_stencil(&mut self);

    /// Renders the geometry with the current style
    fn geometry(&mut self, geometry: &[Triangle<Position>]);

    fn rect(&mut self, rect: Rect) {
        self.geometry(&[
            (rect.bottom_left(), rect.bottom_right(), rect.top_left()).into(),
            (rect.bottom_right(), rect.top_right(), rect.top_left()).into(),
        ])
    }

    // TODO: Consider making it take a reference to Style
    fn style(&mut self, style: DrawStyle);
    fn pop_style(&mut self);

    fn image(&mut self, id: ImageId, bounding_box: Rect, source_rect: Rect, mode: u32);

    fn text(&mut self, text: TextId, ctx: &mut dyn InnerTextContext);

    fn layer(&mut self, index: u32);
    fn pop_layer(&mut self);

    fn filter_new(&mut self);
    fn filter_new_pop(&mut self, id: FilterId, color: Color);
}

pub struct NoopRenderContext;

impl InnerRenderContext for NoopRenderContext {
    fn transform(&mut self, _transform: CarbideTransform) {}

    fn pop_transform(&mut self) {}

    fn clip(&mut self, _bounding_box: Rect) {}

    fn pop_clip(&mut self) {}

    fn filter(&mut self, _id: FilterId, _bounding_box: Rect) {}

    fn filter2d(&mut self, _id1: FilterId, _bounding_box1: Rect, _id2: FilterId, _bounding_box2: Rect) {}

    fn stencil(&mut self, _geometry: &[Triangle<Position>]) {}

    fn pop_stencil(&mut self) {}

    fn geometry(&mut self, _geometry: &[Triangle<Position>]) {}

    fn style(&mut self, _style: DrawStyle) {}

    fn pop_style(&mut self) {}

    fn image(&mut self, _id: ImageId, _bounding_box: Rect, _source_rect: Rect, _mode: u32) {}

    fn text(&mut self, _text: TextId, _ctx: &mut dyn InnerTextContext) {}

    fn layer(&mut self, _index: u32) {}

    fn pop_layer(&mut self) {}

    fn filter_new(&mut self) {}

    fn filter_new_pop(&mut self, _id: FilterId, _color: Color) {}
}