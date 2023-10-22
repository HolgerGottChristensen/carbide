use std::fmt::Debug;

use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position, Scalar};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::render::Render;
use carbide_core::state::{EnvMap1, IntoReadState, Map1, ReadState};
use carbide_core::widget::{CommonWidget, Empty, HStack, Text, Widget, WidgetExt, WidgetId};
use crate::enabled_state;

const PADDING: Scalar = 8.0;

/// # A plain switch widget
/// This widget contains the basic logic for a switch component, without any styling.
/// It can be styled by setting the delegate, using the delegate method.
///
/// For a styled version, use [crate::Switch] instead.
#[derive(Clone, Debug, Widget)]
pub struct Labelled<C, L> where C: Widget + Clone, L: ReadState<T=String> {
    id: WidgetId,
    position: Position,
    dimension: Dimension,

    child: C,
    #[state] label: L,
}

impl Labelled<Empty, String> {
    pub fn new<C: Widget + Clone, L: IntoReadState<String>>(label: L, child: C) -> Labelled<HStack<Vec<Box<dyn Widget>>>, L::Output> {
        Self::new_internal(label.into_read_state(), child)
    }
}

impl<C: Widget + Clone, L: ReadState<T=String>> Labelled<C, L> {
    fn new_internal<C2: Widget + Clone, L2: ReadState<T=String>>(
        label: L2,
        child: C2,
    ) -> Labelled<HStack<Vec<Box<dyn Widget>>>, L2> {

        let label_color = Map1::read_map(enabled_state(), |enabled| {
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

impl<C: Widget + Clone, L: ReadState<T=String>> CommonWidget for Labelled<C, L> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);
}

impl<C: Widget + Clone, L: ReadState<T=String>> WidgetExt for Labelled<C, L> {}

