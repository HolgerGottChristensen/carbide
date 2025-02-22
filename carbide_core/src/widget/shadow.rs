use crate::color::BLACK;
use crate::CommonWidgetImpl;
use crate::draw::{Color, Dimension, Position};
use crate::render::{Render, RenderContext};
use crate::state::{IntoReadState, ReadState};
use crate::widget::{CommonWidget, Empty, FilterId, ImageFilter, IntoWidget, Widget, WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Shadow<W, C, S, X, Y> where W: Widget, C: ReadState<T=Color>, S: ReadState<T=f64>, X: ReadState<T=i32>, Y: ReadState<T=i32> {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,
    child: W,
    filter_id: Option<(FilterId, FilterId)>,
    #[state] color: C,
    #[state] sigma: S,
    #[state] offset_x: X,
    #[state] offset_y: Y,
}

impl Shadow<Empty, Color, f64, i32, i32> {
    pub fn new<S: IntoReadState<f64>, W: IntoWidget>(sigma: S, child: W) -> Shadow<W::Output, Color, S::Output, i32, i32> {
        Shadow {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            child: child.into_widget(),
            filter_id: None,
            color: BLACK,
            sigma: sigma.into_read_state(),
            offset_x: 0,
            offset_y: 0,
        }
    }
}

impl<W: Widget, C: ReadState<T=Color>, S: ReadState<T=f64>, X: ReadState<T=i32>, Y: ReadState<T=i32>> Shadow<W, C, S, X, Y> {
    pub fn shadow_color<C2: IntoReadState<Color>>(self, color: C2) -> Shadow<W, C2::Output, S, X, Y> {
        Shadow {
            id: self.id,
            position: self.position,
            dimension: self.dimension,
            child: self.child,
            filter_id: self.filter_id,
            color: color.into_read_state(),
            sigma: self.sigma,
            offset_x: self.offset_x,
            offset_y: self.offset_y,
        }
    }

    pub fn shadow_offset<X2: IntoReadState<i32>, Y2: IntoReadState<i32>>(self, x: X2, y: Y2) -> Shadow<W, C, S, X2::Output, Y2::Output> {
        Shadow {
            id: self.id,
            position: self.position,
            dimension: self.dimension,
            child: self.child,
            filter_id: self.filter_id,
            color: self.color,
            sigma: self.sigma,
            offset_x: x.into_read_state(),
            offset_y: y.into_read_state(),
        }
    }
}

impl<W: Widget, C: ReadState<T=Color>, S: ReadState<T=f64>, X: ReadState<T=i32>, Y: ReadState<T=i32>> CommonWidget for Shadow<W, C, S, X, Y> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension, flexibility: 0);
}

impl<W: Widget, C: ReadState<T=Color>, S: ReadState<T=f64>, X: ReadState<T=i32>, Y: ReadState<T=i32>> Render for Shadow<W, C, S, X, Y> {
    fn render(&mut self, context: &mut RenderContext) {
        let filter1 = ImageFilter::gaussian_blur_1d(*self.sigma.value() as f32).offset(-*self.offset_x.value(), -*self.offset_y.value());
        let filter2 = ImageFilter::gaussian_blur_1d(*self.sigma.value() as f32).flipped().offset(-*self.offset_x.value(), -*self.offset_y.value());

        context.shadow(&filter1, &filter2, *self.color.value(), |new_context| {
            self.child.render(new_context)
        })
    }
}