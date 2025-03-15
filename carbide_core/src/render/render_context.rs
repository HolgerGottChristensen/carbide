use crate::color::Color;
use crate::color::WHITE;
use crate::draw::stroke::StrokeDashPattern;
use crate::draw::Dimension;
use crate::draw::Rect;
use crate::draw::{DrawOptions, DrawStyle, ImageContext, ImageId, ImageOptions};
use crate::math::Matrix4;

use crate::environment::Environment;
use crate::render::layer::{Layer, LayerId};
use crate::text::{TextContext, TextId};
use crate::widget::{AnyShape, ImageFilter};

pub struct RenderContext<'a, 'b: 'a> {
    pub render: &'a mut dyn InnerRenderContext,
    pub text: &'a mut dyn TextContext,
    pub image: &'a mut dyn ImageContext,
    pub env: &'a mut Environment<'b>,
}

impl<'a, 'b: 'a> RenderContext<'a, 'b> {

    pub fn transform<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, transform: Matrix4<f32>, f: F) -> R {
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

    pub fn filter<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, filter: &ImageFilter, bounding_box: Rect, f: F) -> R {
        let res = f(self);
        self.render.filter(filter, bounding_box);
        res
    }

    pub fn filter2d<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, filter1: &ImageFilter, bounding_box1: Rect, filter2: &ImageFilter, bounding_box2: Rect, f: F) -> R {
        let res = f(self);
        self.render.filter2d(filter1, bounding_box1, filter2, bounding_box2);
        res
    }

    pub fn filter_new<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, filter: &ImageFilter, f: F) -> R {
        self.render.filter_new();
        let res = f(self);
        self.render.filter_new_pop(filter, WHITE, false);
        res
    }

    pub fn filter_new2d<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, filter: &ImageFilter, filter2: &ImageFilter, f: F) -> R {
        self.render.filter_new();
        let res = f(self);
        self.render.filter_new_pop2d(filter, filter2, WHITE, false);
        res
    }

    pub fn shadow<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, id: &ImageFilter, id2: &ImageFilter, color: Color, f: F) -> R {
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

    pub fn stencil<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, shape: &dyn AnyShape, options: impl Into<DrawOptions>, f: F) -> R {
        self.render.stencil(&[(shape, options.into())]);
        let res = f(self);
        self.render.pop_stencil();
        res
    }

    pub fn shape(&mut self, shape: &dyn AnyShape, options: impl Into<DrawOptions>) {
        self.render.shape(shape, options.into());
    }

    pub fn style<R, F: FnOnce(&mut RenderContext) -> R>(&mut self, style: impl Into<DrawStyle>, f: F) -> R {
        self.render.style(style.into());
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

    pub fn image(&mut self, id: ImageId, bounding_box: Rect, options: impl Into<ImageOptions>) {
        self.render.image(id, bounding_box, options.into());
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
    fn transform(&mut self, transform: Matrix4<f32>);
    fn pop_transform(&mut self);

    fn color_filter(&mut self, hue_rotation: f32, saturation_shift: f32, luminance_shift: f32, color_invert: bool);
    fn pop_color_filter(&mut self);

    fn clip(&mut self, bounding_box: Rect);
    fn pop_clip(&mut self);

    fn filter(&mut self, filter: &ImageFilter, bounding_box: Rect);
    fn filter2d(&mut self, filter1: &ImageFilter, bounding_box1: Rect, filter2: &ImageFilter, bounding_box2: Rect);

    fn stencil(&mut self, shapes: &[(&dyn AnyShape, DrawOptions)]);
    fn pop_stencil(&mut self);

    fn shape(&mut self, shape: &dyn AnyShape, options: DrawOptions);

    // TODO: Consider making it take a reference to Style
    fn style(&mut self, style: DrawStyle);
    fn pop_style(&mut self);

    fn stroke_dash_pattern(&mut self, pattern: Option<StrokeDashPattern>);
    fn pop_stroke_dash_pattern(&mut self);

    fn image(&mut self, id: ImageId, bounding_box: Rect, options: ImageOptions);

    fn text(&mut self, text: TextId, ctx: &mut dyn TextContext);

    fn filter_new(&mut self);
    fn filter_new_pop(&mut self, filter: &ImageFilter, color: Color, post_draw: bool);
    fn filter_new_pop2d(&mut self, filter: &ImageFilter, filter2: &ImageFilter, color: Color, post_draw: bool);

    fn mask_start(&mut self);
    fn mask_in(&mut self);
    fn mask_end(&mut self);

    fn layer(&mut self, layer_id: LayerId, dimensions: Dimension) -> Layer;
    fn render_layer(&mut self, layer_id: LayerId, bounding_box: Rect);
}