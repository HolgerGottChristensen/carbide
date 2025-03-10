use std::fmt::Debug;

use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{EnvironmentColor};
use carbide_core::state::{IntoReadState, Map1, ReadState};
use carbide_core::widget::{CommonWidget, Empty, HStack, Text, AnyWidget, WidgetExt, WidgetId, Widget};
use crate::EnabledState;

/// # A plain switch widget
/// This widget contains the basic logic for a switch component, without any styling.
/// It can be styled by setting the delegate, using the delegate method.
///
/// For a styled version, use [crate::Switch] instead.
#[derive(Clone, Debug, Widget)]
pub struct Labelled<C, L> where C: Widget, L: ReadState<T=String> {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,

    child: C,
    #[state] label: L,
}

impl Labelled<Empty, String> {
    pub fn new<C: Widget, L: IntoReadState<String>>(label: L, child: C) -> Labelled<HStack<Vec<Box<dyn AnyWidget>>>, L::Output> {
        Self::new_internal(label.into_read_state(), child)
    }
}

impl<C: Widget, L: ReadState<T=String>> Labelled<C, L> {
    fn new_internal<C2: Widget, L2: ReadState<T=String>>(
        label: L2,
        child: C2,
    ) -> Labelled<HStack<Vec<Box<dyn AnyWidget>>>, L2> {

        let label_color = Map1::read_map(EnabledState::new(true), |enabled| {
            if *enabled {
                EnvironmentColor::Label
            } else {
                EnvironmentColor::TertiaryLabel
            }
        });

        let child = HStack::new(vec![
            Text::new(label.clone()).color(label_color).boxed(),
            Box::new(child.clone())
        ]);

        Labelled {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            child,
            label
        }
    }
}

impl<C: Widget, L: ReadState<T=String>> CommonWidget for Labelled<C, L> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension);
}
