use carbide_macro::carbide_default_builder2;

use crate::{CommonWidgetImpl, impl_state_value};
use crate::draw::{Dimension, Position};
use crate::layout::{Layout, LayoutContext};
use crate::state::{IntoReadState, ReadState};
use crate::widget::{CommonWidget, Empty, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone)]
pub enum ContentMode {
    Fit,
    Fill,
}

impl_state_value!(ContentMode);

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct AspectRatio<D, M, C> where D: ReadState<T=Dimension>, M: ReadState<T=ContentMode>, C: Widget {
    id: WidgetId,
    child: C,
    position: Position,
    dimension: Dimension,
    #[state] ratio: D,
    #[state] mode: M,
}

impl AspectRatio<Dimension, ContentMode, Empty> {
    #[carbide_default_builder2]
    pub fn new<D: IntoReadState<Dimension>, C: Widget>(ratio: D, child: C) -> AspectRatio<D::Output, ContentMode, C> {
        AspectRatio {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            ratio: ratio.into_read_state(),
            mode: ContentMode::Fit,
        }
    }
}

impl<D: ReadState<T=Dimension>, M: ReadState<T=ContentMode>, C: Widget> AspectRatio<D, M, C> {
    pub fn mode<M2: IntoReadState<ContentMode>>(self, mode: M2) -> AspectRatio<D, M2::Output, C> {
        AspectRatio {
            id: self.id,
            child: self.child,
            position: self.position,
            dimension: self.dimension,
            ratio: self.ratio,
            mode: mode.into_read_state(),
        }
    }

    pub fn scale_to_fit(self) -> AspectRatio<D, ContentMode, C> {
        self.mode(ContentMode::Fit)
    }

    pub fn scale_to_fill(self) -> AspectRatio<D, ContentMode, C> {
        self.mode(ContentMode::Fill)
    }
}


impl<D: ReadState<T=Dimension>, M: ReadState<T=ContentMode>, C: Widget> Layout for AspectRatio<D, M, C> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let ratio = self.ratio.value().width / self.ratio.value().height;

        let new_requested = Dimension::new(requested_size.height * ratio, requested_size.height);

        let adjusted_requested = match *self.mode.value() {
            ContentMode::Fit => {
                if new_requested.width > requested_size.width {
                    let scale = requested_size.width / new_requested.width;
                    Dimension::new(new_requested.width * scale, new_requested.height * scale)
                } else {
                    new_requested
                }
            }
            ContentMode::Fill => {
                if new_requested.width < requested_size.width {
                    let scale = requested_size.width / new_requested.width;
                    Dimension::new(new_requested.width * scale, new_requested.height * scale)
                } else {
                    new_requested
                }
            }
        };

        self.child.calculate_size(adjusted_requested, ctx);

        self.dimension = adjusted_requested;

        self.dimension
    }
}

impl<D: ReadState<T=Dimension>, M: ReadState<T=ContentMode>, C: Widget> CommonWidget for AspectRatio<D, M, C> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);
}