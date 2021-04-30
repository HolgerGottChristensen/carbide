use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use carbide_core::DeserializeOwned;
use carbide_core::event_handler::{KeyboardEvent, MouseEvent};
use carbide_core::input::Key;
use carbide_core::input::MouseButton;
use carbide_core::prelude::Uuid;
use carbide_core::Serialize;
use carbide_core::widget::*;

use crate::{PlainCheckBox, CheckBoxState, CheckBoxValue};

#[derive(Clone, Widget)]
pub struct CheckBox<GS> where GS: GlobalState {
    id: Id,
    child: PlainCheckBox<GS>,
    position: Point,
    dimension: Dimensions,
}

impl<GS: GlobalState> CheckBox<GS> {

    pub fn new<S: Into<StringState<GS>>, L: Into<CheckBoxState<GS>>>(label: S, checked: L) -> Box<Self> {

        let mut child = *PlainCheckBox::new(label, checked.into());

        child = *child.delegate(|focus_state, checked_state, button: Box<dyn Widget<GS>>| {

            let focus_color = TupleState3::new(
                focus_state,
                EnvironmentColor::OpaqueSeparator.into(),
                EnvironmentColor::Accent.into()
            ).mapped(|(focus, primary_color, focus_color)| {
                if focus == &Focus::Focused {
                    *focus_color
                } else {
                    *primary_color
                }
            });

            let checked_color = TupleState3::new(
                checked_state.clone(),
                EnvironmentColor::SecondarySystemBackground.into(),
                EnvironmentColor::Accent.into()
            ).mapped(|(selected, primary_color, checked_color)| {
                if *selected == CheckBoxValue::False {
                    *primary_color
                } else {
                    *checked_color
                }
            });

            let checked_true = checked_state.clone().mapped(|checked| {
                *checked == CheckBoxValue::True
            });

            let checked_intermediate = checked_state.clone().mapped(|checked| {
                *checked == CheckBoxValue::Intermediate
            });

            ZStack::initialize(vec![
                RoundedRectangle::initialize(CornerRadii::all(3.0))
                    .fill(checked_color)
                    .stroke(focus_color)
                    .stroke_style(1.0),
                IfElse::new(checked_intermediate)
                    .when_true(
                        Canvas::initialize(|rect, mut context| {
                            context.move_to(4.0, 8.0);
                            context.line_to(12.0, 8.0);


                            context.set_stroke_style(EnvironmentColor::DarkText);
                            context.set_line_width(2.0);
                            context.stroke();

                            context
                        })
                    ),
                IfElse::new(checked_true)
                    .when_true(
                        Canvas::initialize(|rectangle, mut context| {
                            context.move_to(4.0, 9.0);
                            context.line_to(7.0, 12.0);
                            context.line_to(12.0, 4.0);

                            context.set_stroke_style(EnvironmentColor::DarkText);
                            context.set_line_width(2.0);
                            context.stroke();

                            context
                        })
                    ),
                button
            ]).frame(16.0, 16.0)
        });

        Box::new(CheckBox {
            id: Id::new_v4(),
            child,
            position: [0.0,0.0],
            dimension: [235.0, 26.0],
        })
    }

}

impl<GS: GlobalState> CommonWidget<GS> for CheckBox<GS> {
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

impl<GS: GlobalState> ChildRender for CheckBox<GS> {}

impl<GS: GlobalState> Layout<GS> for CheckBox<GS> {
    fn flexibility(&self) -> u32 {
        5
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment<GS>) -> Dimensions {
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


impl<GS: GlobalState> WidgetExt<GS> for CheckBox<GS> {}
