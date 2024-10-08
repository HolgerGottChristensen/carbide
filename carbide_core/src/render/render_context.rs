use carbide::color::Color;
use carbide::draw::Dimension;
use carbide_core::draw::Rect;
use crate::color::WHITE;
use crate::draw::{InnerImageContext, Position, DrawStyle, ImageId, StrokeDashPattern};
use crate::draw::shape::stroke_vertex::StrokeVertex;
use crate::draw::shape::triangle::Triangle;

use crate::render::CarbideTransform;

use crate::text::{InnerTextContext, TextId};
use crate::widget::FilterId;
use crate::environment::Environment;
use crate::render::layer::{Layer, LayerId, NoopLayer};

pub struct RenderContext<'a> {
    pub render: &'a mut dyn InnerRenderContext,
    pub text: &'a mut dyn InnerTextContext,
    pub image: &'a mut dyn InnerImageContext,
    pub env: &'a mut Environment,
}

impl<'a> RenderContext<'a> {

    // TODO: Change BasicLayouter to something more suitable
    pub fn transform<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, transform: CarbideTransform, f: F) -> R {
        self.render.transform(transform);
        let res = f(self);
        self.render.pop_transform();
        res
    }

    /// Hue rotation given in turns.
    pub fn hue_rotation<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, rotation: f32, f: F) -> R {
        self.render.color_filter(rotation, 0.0, 0.0, false);
        let res = f(self);
        self.render.pop_color_filter();
        res
    }

    pub fn saturation_shift<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, shift: f32, f: F) -> R {
        self.render.color_filter(0.0, shift, 0.0, false);
        let res = f(self);
        self.render.pop_color_filter();
        res
    }

    pub fn luminance_shift<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, shift: f32, f: F) -> R {
        self.render.color_filter(0.0, 0.0, shift, false);
        let res = f(self);
        self.render.pop_color_filter();
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
        self.render.filter_new_pop(id, WHITE, false);
        res
    }

    pub fn filter_new2d<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, id: FilterId, id2: FilterId, f: F) -> R {
        self.render.filter_new();
        let res = f(self);
        self.render.filter_new_pop2d(id, id2, WHITE, false);
        res
    }

    pub fn shadow<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, id: FilterId, id2: FilterId, color: Color, f: F) -> R {
        self.render.filter_new();
        let res = f(self);
        self.render.filter_new_pop2d(id, id2, color, true);
        res
    }

    pub fn mask<R, F: FnOnce(&mut RenderContext) -> R, R2, F2: FnOnce(&mut RenderContext) -> R2>(&mut self, mask: F2, f: F) -> R {
        self.render.mask_start();
        let _ = mask(self);
        self.render.mask_in();
        let res = f(self);
        self.render.mask_end();
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



    pub fn stroke(&mut self, stroke: &[Triangle<StrokeVertex>]) {
        if stroke.is_empty() {
            return;
        }

        self.render.stroke(stroke);
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

    pub fn stroke_dash_pattern<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, pattern: Option<StrokeDashPattern>, f: F) -> R {
        self.render.stroke_dash_pattern(pattern);
        let res = f(self);
        self.render.pop_stroke_dash_pattern();
        res
    }

    pub fn image(&mut self, id: ImageId, bounding_box: Rect, source_rect: Rect, mode: u32) {
        self.render.image(Some(id), bounding_box, source_rect, mode);
    }

    pub fn text(&mut self, text: TextId) {
        self.render.text(text, self.text);
    }

    pub fn layer<R, F: FnOnce(Layer, &mut Environment) -> R>(&mut self, layer_id: LayerId, bounding_box: Rect, f: F) -> R {
        let layer = self.render.layer(layer_id, bounding_box.dimension);
        let res = f(layer, self.env);
        self.render.render_layer(layer_id, bounding_box);

        res
    }
}

pub trait InnerRenderContext {
    fn transform(&mut self, transform: CarbideTransform);
    fn pop_transform(&mut self);

    fn color_filter(&mut self, hue_rotation: f32, saturation_shift: f32, luminance_shift: f32, color_invert: bool);
    fn pop_color_filter(&mut self);

    fn clip(&mut self, bounding_box: Rect);
    fn pop_clip(&mut self);

    fn filter(&mut self, id: FilterId, bounding_box: Rect);
    fn filter2d(&mut self, id1: FilterId, bounding_box1: Rect, id2: FilterId, bounding_box2: Rect);

    fn stencil(&mut self, geometry: &[Triangle<Position>]);
    fn pop_stencil(&mut self);

    /// Renders the geometry with the current style
    fn geometry(&mut self, geometry: &[Triangle<Position>]);
    fn stroke(&mut self, stroke: &[Triangle<StrokeVertex>]);


    fn rect(&mut self, rect: Rect) {
        self.geometry(&[
            (rect.bottom_left(), rect.bottom_right(), rect.top_left()).into(),
            (rect.bottom_right(), rect.top_right(), rect.top_left()).into(),
        ])
    }

    // TODO: Consider making it take a reference to Style
    fn style(&mut self, style: DrawStyle);
    fn pop_style(&mut self);

    fn stroke_dash_pattern(&mut self, pattern: Option<StrokeDashPattern>);
    fn pop_stroke_dash_pattern(&mut self);

    fn image(&mut self, id: Option<ImageId>, bounding_box: Rect, source_rect: Rect, mode: u32);

    fn text(&mut self, text: TextId, ctx: &mut dyn InnerTextContext);

    fn filter_new(&mut self);
    fn filter_new_pop(&mut self, id: FilterId, color: Color, post_draw: bool);
    fn filter_new_pop2d(&mut self, id: FilterId, id2: FilterId, color: Color, post_draw: bool);

    fn mask_start(&mut self);
    fn mask_in(&mut self);
    fn mask_end(&mut self);

    fn layer(&mut self, layer_id: LayerId, dimensions: Dimension) -> Layer;
    fn render_layer(&mut self, layer_id: LayerId, bounding_box: Rect);
}