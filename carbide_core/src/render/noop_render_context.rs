use crate::draw::stroke::StrokeDashPattern;
use crate::draw::DrawShape;
use crate::draw::{Color, CompositeDrawShape, Dimension, DrawOptions, DrawStyle, ImageId, ImageOptions, Rect};
use crate::render::{InnerRenderContext, Layer, LayerId, NoopLayer};
use crate::text::{TextContext, TextId};
use crate::widget::ImageFilter;
use crate::environment::Environment;
use cgmath::Matrix4;

pub struct NoopRenderContext;

impl InnerRenderContext for NoopRenderContext {
    fn transform(&mut self, _transform: Matrix4<f32>) {}

    fn pop_transform(&mut self) {}

    fn color_filter(&mut self, _hue_rotation: f32, _saturation_shift: f32, _luminance_shift: f32, _color_invert: bool) {}

    fn pop_color_filter(&mut self) {}

    fn clip(&mut self, _bounding_box: Rect) {}

    fn pop_clip(&mut self) {}

    fn filter(&mut self, _id: &ImageFilter, _bounding_box: Rect) {}

    fn filter2d(&mut self, _id1: &ImageFilter, _bounding_box1: Rect, _id2: &ImageFilter, _bounding_box2: Rect) {}

    fn stencil(&mut self, _shape: CompositeDrawShape) {}

    fn pop_stencil(&mut self) {}

    fn shape(&mut self, _shape: DrawShape, _option: DrawOptions) {}

    fn style(&mut self, _style: DrawStyle) {}
    fn pop_style(&mut self) {}

    fn stroke_dash_pattern(&mut self, _pattern: Option<StrokeDashPattern>) {}
    fn pop_stroke_dash_pattern(&mut self) {}

    fn image(&mut self, _id: ImageId, _bounding_box: Rect, _options: ImageOptions) {}

    fn text(&mut self, _text: TextId, _ctx: &mut dyn TextContext) {}

    fn filter_new(&mut self) {}

    fn filter_new_pop(&mut self, _id: &ImageFilter, _color: Color, _post_draw: bool) {}

    fn filter_new_pop2d(&mut self, _id: &ImageFilter, _id2: &ImageFilter, _color: Color, _post_draw: bool) {}

    fn mask_start(&mut self) {}
    fn mask_in(&mut self) {}
    fn mask_end(&mut self) {}

    fn layer(&mut self, _layer_id: LayerId, _dimensions: Dimension, _env: &mut Environment) -> Layer<'_> {
        static LAYER: NoopLayer = NoopLayer;
        Layer {
            inner: &LAYER,
            inner2: &LAYER
        }
    }

    fn render_layer(&mut self, _layer_id: LayerId, _bounding_box: Rect) {}
}