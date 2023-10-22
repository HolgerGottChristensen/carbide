use carbide_core::state::{IntoReadState, ReadState};
use carbide_macro::{carbide_default_builder2};

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::environment::EnvironmentColor;

use crate::state::ReadStateExtNew;
use crate::widget::{Capsule, CommonWidget, Empty, Frame, HSplit, Spacer, Widget, WidgetExt, WidgetId, ZStack};

#[derive(Debug, Clone, Widget)]
pub struct ProgressBar<P> where P: ReadState<T=f64> + Clone {
    id: WidgetId,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    #[state]
    progress: P,
}

impl ProgressBar<f64> {
    #[carbide_default_builder2]
    pub fn new<P: IntoReadState<f64>>(progress: P) -> Box<ProgressBar<P::Output>> {
        let progress = progress.into_read_state();

        let child = ZStack::new(vec![
            Capsule::new().fill(EnvironmentColor::SystemFill).boxed(),
            HSplit::new(Capsule::new().fill(EnvironmentColor::Accent).boxed(), Spacer::new())
                .percent(progress.ignore_writes())
                .non_draggable()
                .boxed(),
        ])
            .frame(0.0, 5.0)
            .expand_width()
            .boxed();

        Box::new(ProgressBar {
            id: WidgetId::new(),
            child,
            position: Default::default(),
            dimension: Default::default(),
            progress,
        })
    }
}

impl<P: ReadState<T=f64> + Clone> CommonWidget for ProgressBar<P> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);
}

impl<P: ReadState<T=f64> + Clone> WidgetExt for ProgressBar<P> {}
