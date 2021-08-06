use std::fmt::Debug;

use carbide_core::{DeserializeOwned, Serialize};
use carbide_core::widget::*;

use crate::PlainRadioButton;

#[derive(Clone, Widget)]
pub struct RadioButton<T, GS> where GS: GlobalStateContract, T: Serialize + Clone + Debug + Default + DeserializeOwned + PartialEq + 'static {
    id: Id,
    child: PlainRadioButton<T, GS>,
    position: Point,
    dimension: Dimensions,
}

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + PartialEq + 'static, GS: GlobalStateContract> RadioButton<T, GS> {
    pub fn new<S: Into<StringState<GS>>, L: Into<TState<T, GS>>>(label: S, reference: T, local_state: L) -> Box<Self> {
        let mut child = *PlainRadioButton::new(label, reference, local_state);

        child = *child.delegate(|focus_state, selected_state, button: Box<dyn Widget<GS>>| {
            let focus_color = TupleState3::new(
                focus_state,
                EnvironmentColor::OpaqueSeparator,
                EnvironmentColor::Accent,
            ).mapped(|(focus, primary_color, focus_color)| {
                if focus == &Focus::Focused {
                    *focus_color
                } else {
                    *primary_color
                }
            });

            let selected_color = TupleState3::new(
                selected_state.clone(),
                EnvironmentColor::SecondarySystemBackground,
                EnvironmentColor::Accent,
            ).mapped(|(selected, primary_color, selected_color)| {
                if *selected {
                    *selected_color
                } else {
                    *primary_color
                }
            });

            ZStack::initialize(vec![
                Ellipse::new()
                    .fill(selected_color)
                    .stroke(focus_color)
                    .stroke_style(1.0),
                IfElse::new(selected_state)
                    .when_true(
                        Ellipse::new()
                            .fill(EnvironmentColor::DarkText)
                            .frame(6.0, 6.0)
                    ),
                button
            ]).frame(16.0, 16.0)
        });

        Box::new(RadioButton {
            id: Id::new_v4(),
            child,
            position: [0.0, 0.0],
            dimension: [235.0, 26.0],
        })
    }
}

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + PartialEq + 'static, GS: GlobalStateContract> CommonWidget<GS> for RadioButton<T, GS> {
    fn get_id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<GS> {
        WidgetIter::single(&self.child)
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<GS> {
        WidgetIterMut::single(&mut self.child)
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<GS> {
        WidgetIterMut::single(&mut self.child)
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut<GS> {
        WidgetIterMut::single(&mut self.child)
    }

    fn get_position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimensions {
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + PartialEq + 'static, GS: GlobalStateContract> ChildRender for RadioButton<T, GS> {}

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + PartialEq + 'static, GS: GlobalStateContract> Layout<GS> for RadioButton<T, GS> {
    fn flexibility(&self) -> u32 {
        5
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment<GS>) -> Dimensions {
        self.set_width(requested_size[0]);

        self.child.calculate_size(self.dimension, env);

        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.get_position();
        let dimension = self.get_dimension();


        positioning(position, dimension, &mut self.child);
        self.child.position_children();
    }
}


impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + PartialEq + 'static, GS: GlobalStateContract> WidgetExt<GS> for RadioButton<T, GS> {}
