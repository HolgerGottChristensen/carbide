use carbide::widget::AnyWidget;
use carbide_core::state::{IntoReadState, ReadState};
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::environment::EnvironmentColor;
use crate::state::ReadStateExtNew;
use crate::widget::{Capsule, CommonWidget, Empty, HSplit, Spacer, Widget, WidgetExt, WidgetId, ZStack};

#[derive(Debug, Clone, Widget)]
pub struct ProgressBar<P, W> where P: ReadState<T=f64>, W: Widget {
    #[id] id: WidgetId,
    child: W,
    position: Position,
    dimension: Dimension,
    #[state] progress: P,
}

impl ProgressBar<f64, Empty> {
    #[carbide_default_builder2]
    pub fn new<P: IntoReadState<f64>>(progress: P) -> ProgressBar<P::Output, impl Widget> {
        let progress = progress.into_read_state();

        let child = ZStack::new((
            Capsule::new().fill(EnvironmentColor::SystemFill),
            HSplit::new(Capsule::new().fill(EnvironmentColor::Accent), Spacer::new())
                .percent(progress.ignore_writes())
                .non_draggable()
                .boxed(),
        ))
            .frame(0.0, 5.0)
            .expand_width();

        ProgressBar {
            id: WidgetId::new(),
            child,
            position: Default::default(),
            dimension: Default::default(),
            progress,
        }
    }
}

impl<P: ReadState<T=f64>, W: Widget> CommonWidget for ProgressBar<P, W> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension);
}