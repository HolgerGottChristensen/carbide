use crate::color::Color;
use crate::draw::stroke::StrokeDashPattern;
use crate::draw::{DrawOptions, DrawShape, DrawStyle, ImageId, ImageOptions, Rect};
use crate::environment::Environment;
use crate::render::{InnerRenderContext, Layer, LayerId};
use crate::text::{TextContext, TextId};
use crate::widget::ImageFilter;
use carbide_macro::carbide_default_builder2;
use cgmath::Matrix4;

use crate::draw::{Alignment, CompositeDrawShape, Dimension, Position};
use crate::layout::{Layout, LayoutContext};
use crate::render::{Render, RenderContext};
use crate::widget::{
    AnyShape, AnyWidget, CommonWidget, Empty, Widget, WidgetId, WidgetSync,
};
use crate::CommonWidgetImpl;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout, StateSync)]
pub struct ClipShape<C, S>
where
    C: Widget,
    S: AnyShape + AnyWidget + Clone,
{
    #[id]
    id: WidgetId,
    child: C,
    shape: S,
    position: Position,
    dimension: Dimension,
}

impl ClipShape<Empty, Empty> {
    #[carbide_default_builder2]
    pub fn new<C: Widget, S: AnyShape + AnyWidget + Clone>(child: C, shape: S) -> ClipShape<C, S> {
        ClipShape {
            id: WidgetId::new(),
            child,
            shape,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        }
    }
}

impl<C: Widget, S: AnyShape + AnyWidget + Clone> WidgetSync for ClipShape<C, S> {
    fn sync(&mut self, env: &mut Environment) {
        self.child.sync(env);
        self.shape.sync(env);
    }
}

impl<C: Widget, S: AnyShape + AnyWidget + Clone> Layout for ClipShape<C, S> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.child.calculate_size(requested_size, ctx);
        self.shape.calculate_size(requested_size, ctx);
        self.dimension = requested_size;
        requested_size
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let position = self.position;
        let dimension = self.dimension;

        self.child.set_position(Alignment::Center.position(
            position,
            dimension,
            self.child.dimension(),
        ));
        self.shape.set_position(Alignment::Center.position(
            position,
            dimension,
            self.shape.dimension(),
        ));

        self.child.position_children(ctx);
        self.shape.position_children(ctx);
    }
}

impl<C: Widget, S: AnyShape + AnyWidget + Clone> CommonWidget for ClipShape<C, S> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension, flexibility: 0);
}

impl<C: Widget, S: AnyShape + AnyWidget + Clone> Render for ClipShape<C, S> {
    fn render(&mut self, context: &mut RenderContext) {
        let mut capture = ShapeRenderCapture { shape: vec![] };

        self.shape.render(&mut RenderContext {
            render: &mut capture,
            text: context.text,
            image: context.image,
            env: context.env,
        });

        context.stencil(CompositeDrawShape::Many(capture.shape), |ctx| {
            self.child.render(ctx)
        })
    }
}

struct ShapeRenderCapture {
    shape: Vec<(DrawShape, DrawOptions)>,
}

impl InnerRenderContext for ShapeRenderCapture {
    fn shape(&mut self, shape: DrawShape, option: DrawOptions) {
        self.shape.push((shape, option));
    }

    fn transform(&mut self, _transform: Matrix4<f32>) {}

    fn pop_transform(&mut self) {}

    fn color_filter(
        &mut self,
        _hue_rotation: f32,
        _saturation_shift: f32,
        _luminance_shift: f32,
        _color_invert: bool,
    ) {
    }

    fn pop_color_filter(&mut self) {}

    fn clip(&mut self, _bounding_box: Rect) {}

    fn pop_clip(&mut self) {}

    fn filter(&mut self, _filter: &ImageFilter, _bounding_box: Rect) {}

    fn filter2d(
        &mut self,
        _filter1: &ImageFilter,
        _bounding_box1: Rect,
        _filter2: &ImageFilter,
        _bounding_box2: Rect,
    ) {
    }

    fn stencil(&mut self, _shape: CompositeDrawShape) {}

    fn pop_stencil(&mut self) {}

    fn style(&mut self, _style: DrawStyle) {}

    fn pop_style(&mut self) {}

    fn stroke_dash_pattern(&mut self, _pattern: Option<StrokeDashPattern>) {}

    fn pop_stroke_dash_pattern(&mut self) {}

    fn image(&mut self, _id: ImageId, _bounding_box: Rect, _options: ImageOptions) {}

    fn text(&mut self, _text: TextId, _ctx: &mut dyn TextContext) {}

    fn filter_new(&mut self) {}

    fn filter_new_pop(&mut self, _filter: &ImageFilter, _color: Color, _post_draw: bool) {}

    fn filter_new_pop2d(
        &mut self,
        _filter: &ImageFilter,
        _filter2: &ImageFilter,
        _color: Color,
        _post_draw: bool,
    ) {
    }

    fn mask_start(&mut self) {}

    fn mask_in(&mut self) {}

    fn mask_end(&mut self) {}

    fn layer(&mut self, _layer_id: LayerId, _dimensions: Dimension, _env: &mut Environment) -> Layer<'_> {
        todo!()
    }

    fn render_layer(&mut self, _layer_id: LayerId, _bounding_box: Rect) {}
}
