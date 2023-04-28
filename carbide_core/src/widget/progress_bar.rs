
use carbide_macro::carbide_default_builder;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::environment::EnvironmentColor;
use crate::state::TState;
use crate::widget::{Capsule, CommonWidget, HSplit, Spacer, Widget, WidgetExt, WidgetId, ZStack};

#[derive(Debug, Clone, Widget)]
pub struct ProgressBar {
    id: WidgetId,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    #[state]
    progress: TState<f64>,
}

impl ProgressBar {
    #[carbide_default_builder]
    pub fn new(progress: impl Into<TState<f64>>) -> Box<Self> {}

    pub fn new(progress: impl Into<TState<f64>>) -> Box<Self> {
        let progress = progress.into();

        let child = ZStack::new(vec![
            Capsule::new().fill(EnvironmentColor::SystemFill),
            HSplit::new(Capsule::new().fill(EnvironmentColor::Accent), Spacer::new())
                .percent(progress.clone())
                .non_draggable(),
        ])
            .frame(0.0, 5)
            .expand_width();

        Box::new(ProgressBar {
            id: WidgetId::new(),
            child,
            position: Default::default(),
            dimension: Default::default(),
            progress,
        })
    }
}

impl CommonWidget for ProgressBar {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);
}

impl WidgetExt for ProgressBar {}
