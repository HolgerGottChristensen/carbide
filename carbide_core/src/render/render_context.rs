use image::DynamicImage;
use carbide_core::draw::Rect;
use crate::draw::{BoundingBox, Position};
use crate::draw::draw_style::DrawStyle;
use crate::draw::image::ImageId;
use crate::draw::shape::triangle::Triangle;
use crate::layout::BasicLayouter;
use crate::render::CarbideTransform;
use crate::render::style::Style;
use crate::text::Glyph;
use crate::widget::FilterId;

pub struct RenderContext<'a> {
    inner: &'a mut dyn InnerRenderContext,
}

impl<'a> RenderContext<'a> {

    pub fn new(inner: &'a mut dyn InnerRenderContext) -> RenderContext<'a> {
        RenderContext {
            inner,
        }
    }

    // TODO: Change BasicLayouter to something more suitable
    pub fn transform<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, transform: CarbideTransform, f: F) -> R {
        self.inner.transform(transform);
        let res = f(self);
        self.inner.pop_transform();
        res
    }

    pub fn clip<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, bounding_box: BoundingBox, f: F) -> R {
        self.inner.clip(bounding_box);
        let res = f(self);
        self.inner.pop_clip();
        res
    }

    pub fn filter<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, id: FilterId, bounding_box: BoundingBox, f: F) -> R {
        let res = f(self);
        self.inner.filter(id, bounding_box);
        res
    }

    pub fn filter2d<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, id1: FilterId, bounding_box1: BoundingBox, id2: FilterId, bounding_box2: BoundingBox, f: F) -> R {
        let res = f(self);
        self.inner.filter2d(id1, bounding_box1, id2, bounding_box2);
        res
    }


    pub fn stencil<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, geometry: &[Triangle<Position>], f: F) -> R {
        self.inner.stencil(geometry);
        let res = f(self);
        self.inner.pop_stencil();
        res
    }

    /// Renders the geometry with the current style
    pub fn geometry(&mut self, geometry: &[Triangle<Position>]) {
        if geometry.is_empty() {
            return;
        }

        self.inner.geometry(geometry);
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
        self.inner.style(style);
        let res = f(self);
        self.inner.pop_style();
        res
    }

    pub fn image(&mut self, id: ImageId, bounding_box: BoundingBox, source_rect: Rect, mode: u32) {
        self.inner.image(id, bounding_box, source_rect, mode);
    }

    pub fn text(&mut self, text: &[Glyph]) {
        self.inner.text(text);
    }
}

pub trait InnerRenderContext {
    fn transform(&mut self, transform: CarbideTransform);
    fn pop_transform(&mut self);

    fn clip(&mut self, bounding_box: BoundingBox);
    fn pop_clip(&mut self);

    fn filter(&mut self, id: FilterId, bounding_box: BoundingBox);
    fn filter2d(&mut self, id1: FilterId, bounding_box1: BoundingBox, id2: FilterId, bounding_box2: BoundingBox);

    fn stencil(&mut self, geometry: &[Triangle<Position>]);
    fn pop_stencil(&mut self);

    /// Renders the geometry with the current style
    fn geometry(&mut self, geometry: &[Triangle<Position>]);

    // TODO: Consider making it take a reference to Style
    fn style(&mut self, style: DrawStyle);
    fn pop_style(&mut self);

    fn image(&mut self, id: ImageId, bounding_box: Rect, source_rect: Rect, mode: u32);

    fn text(&mut self, text: &[Glyph]);
}

pub struct NoopRenderContext;

impl InnerRenderContext for NoopRenderContext {
    fn transform(&mut self, transform: CarbideTransform) {}

    fn pop_transform(&mut self) {}

    fn clip(&mut self, bounding_box: BoundingBox) {}

    fn pop_clip(&mut self) {}

    fn filter(&mut self, id: FilterId, bounding_box: BoundingBox) {}

    fn filter2d(&mut self, id1: FilterId, bounding_box1: BoundingBox, id2: FilterId, bounding_box2: BoundingBox) {}

    fn stencil(&mut self, geometry: &[Triangle<Position>]) {}

    fn pop_stencil(&mut self) {}

    fn geometry(&mut self, geometry: &[Triangle<Position>]) {}

    fn style(&mut self, style: DrawStyle) {}

    fn pop_style(&mut self) {}

    fn image(&mut self, id: ImageId, bounding_box: Rect, source_rect: Rect, mode: u32) {}

    fn text(&mut self, text: &[Glyph]) {}
}