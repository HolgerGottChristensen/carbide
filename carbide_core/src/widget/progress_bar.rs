use instant::Duration;
use carbide_core::color::TRANSPARENT;

use crate::color::WHITE;
use crate::draw::{Dimension, Position, Rect};
use crate::prelude::*;
use crate::prelude::canvas::LineCap;
use crate::render::PrimitiveKind;
use crate::widget::canvas::Canvas;
use crate::CommonWidgetImpl;

#[derive(Debug, Clone, Widget)]
pub struct ProgressBar {
    id: Id,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    #[state] progress: F64State,
}

impl ProgressBar {
    pub fn new(progress: impl Into<F64State>) -> Box<Self> {
        ProgressBar::new_internal(progress.into())
    }

    fn new_internal(progress: F64State) -> Box<Self> {
        let child = ZStack::new(vec![
            Capsule::new().fill(EnvironmentColor::SystemFill),
            HSplit::new(
                Capsule::new().fill(EnvironmentColor::Accent),
                Spacer::new()
            ).percent(progress.clone())
                .non_draggable()
        ]).frame(SCALE, 5);

        Box::new(ProgressBar {
            id: Id::new_v4(),
            child,
            position: Default::default(),
            dimension: Default::default(),
            progress,
        })
    }
}

CommonWidgetImpl!(ProgressBar, self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);

impl WidgetExt for ProgressBar {}
