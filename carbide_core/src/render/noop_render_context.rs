use carbide::widget::ImageFilter;
use crate::draw::{Color, Dimension, DrawStyle, ImageId, Position, Rect, StrokeDashPattern};
use crate::draw::shape::stroke_vertex::StrokeVertex;
use crate::draw::shape::triangle::Triangle;
use crate::render::{CarbideTransform, InnerRenderContext, Layer, LayerId, NoopLayer};
use crate::text::{InnerTextContext, TextId};

pub struct NoopRenderContext;

impl InnerRenderContext for NoopRenderContext {
    fn transform(&mut self, _transform: CarbideTransform) {}

    fn pop_transform(&mut self) {}

    fn color_filter(&mut self, _hue_rotation: f32, _saturation_shift: f32, _luminance_shift: f32, _color_invert: bool) {}

    fn pop_color_filter(&mut self) {}

    fn clip(&mut self, _bounding_box: Rect) {}

    fn pop_clip(&mut self) {}

    fn filter(&mut self, _id: &ImageFilter, _bounding_box: Rect) {}

    fn filter2d(&mut self, _id1: &ImageFilter, _bounding_box1: Rect, _id2: &ImageFilter, _bounding_box2: Rect) {}

    fn stencil(&mut self, _geometry: &[Triangle<Position>]) {}

    fn pop_stencil(&mut self) {}

    fn geometry(&mut self, _geometry: &[Triangle<Position>]) {}

    fn stroke(&mut self, _stroke: &[Triangle<StrokeVertex>]) {}

    fn style(&mut self, _style: DrawStyle) {}
    fn pop_style(&mut self) {}

    fn stroke_dash_pattern(&mut self, _pattern: Option<StrokeDashPattern>) {}
    fn pop_stroke_dash_pattern(&mut self) {}

    fn image(&mut self, _id: Option<ImageId>, _bounding_box: Rect, _source_rect: Rect, _mode: u32) {}

    fn text(&mut self, _text: TextId, _ctx: &mut dyn InnerTextContext) {}

    fn filter_new(&mut self) {}

    fn filter_new_pop(&mut self, _id: &ImageFilter, _color: Color, _post_draw: bool) {}

    fn filter_new_pop2d(&mut self, _id: &ImageFilter, _id2: &ImageFilter, _color: Color, _post_draw: bool) {}

    fn mask_start(&mut self) {}
    fn mask_in(&mut self) {}
    fn mask_end(&mut self) {}

    fn layer(&mut self, _layer_id: LayerId, _dimensions: Dimension) -> Layer {
        static LAYER: NoopLayer = NoopLayer;
        Layer {
            inner: &LAYER,
            inner2: &LAYER
        }
    }

    fn render_layer(&mut self, _layer_id: LayerId, _bounding_box: Rect) {}
}