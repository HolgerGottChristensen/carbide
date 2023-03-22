use image::DynamicImage;
use carbide_core::draw::Rect;
use crate::draw::{BoundingBox, Position};
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
    pub fn transform<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, transform: CarbideTransform, anchor: BasicLayouter, f: F) -> R {
        self.inner.transform(transform, anchor);
        let res = f(self);
        self.inner.de_transform();
        res
    }

    pub fn clip<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, bounding_box: BoundingBox, f: F) -> R {
        self.inner.clip(bounding_box);
        let res = f(self);
        self.inner.de_clip();
        res
    }

    pub fn filter<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, id: FilterId, f: F) -> R {
        let res = f(self);
        self.inner.filter(id);
        res
    }

    pub fn stencil<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, geometry: &Vec<Triangle<Position>>, f: F) -> R {
        self.inner.stencil(geometry);
        let res = f(self);
        self.inner.de_stencil();
        res
    }

    /// Renders the geometry with the current style
    pub fn geometry(&mut self, geometry: &Vec<Triangle<Position>>) {
        if geometry.is_empty() {
            return;
        }

        self.inner.geometry(geometry);
    }

    pub fn style<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, style: Style, f: F) -> R {
        self.inner.style(style);
        let res = f(self);
        self.inner.de_style();
        res
    }

    pub fn image(&mut self, id: ImageId, bounding_box: BoundingBox, source_rect: Rect, mode: u32) {
        self.inner.image(id, bounding_box, source_rect, mode);
    }

    pub fn text(&mut self, text: &Vec<Glyph>) {
        self.inner.text(text);
    }
}

pub trait InnerRenderContext {
    fn transform(&mut self, transform: CarbideTransform, anchor: BasicLayouter);
    fn de_transform(&mut self);

    fn clip(&mut self, bounding_box: BoundingBox);
    fn de_clip(&mut self);

    fn filter(&mut self, id: FilterId);

    fn stencil(&mut self, geometry: &Vec<Triangle<Position>>);
    fn de_stencil(&mut self);

    /// Renders the geometry with the current style
    fn geometry(&mut self, geometry: &Vec<Triangle<Position>>);

    fn style(&mut self, style: Style);

    fn de_style(&mut self);

    fn image(&mut self, id: ImageId, bounding_box: Rect, source_rect: Rect, mode: u32);

    fn text(&mut self, text: &Vec<Glyph>);
}

pub struct NoopRenderContext;

impl InnerRenderContext for NoopRenderContext {
    fn transform(&mut self, transform: CarbideTransform, anchor: BasicLayouter) {}

    fn de_transform(&mut self) {}

    fn clip(&mut self, bounding_box: BoundingBox) {}

    fn de_clip(&mut self) {}

    fn filter(&mut self, id: FilterId) {}

    fn stencil(&mut self, geometry: &Vec<Triangle<Position>>) {}

    fn de_stencil(&mut self) {}

    fn geometry(&mut self, geometry: &Vec<Triangle<Position>>) {}

    fn style(&mut self, style: Style) {}

    fn de_style(&mut self) {}

    fn image(&mut self, id: ImageId, bounding_box: Rect, source_rect: Rect, mode: u32) {}

    fn text(&mut self, text: &Vec<Glyph>) {}
}