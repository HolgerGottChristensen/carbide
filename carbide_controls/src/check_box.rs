use carbide_core::draw::Dimension;
use carbide_core::widget::*;

use crate::PlainCheckBox;
use crate::types::*;

#[derive(Clone, Widget)]
pub struct CheckBox<GS> where GS: GlobalStateContract {
    id: Id,
    child: PlainCheckBox<GS>,
    position: Point,
    dimension: Dimensions,
}

impl<GS: GlobalStateContract> CheckBox<GS> {
    pub fn new<S: Into<StringState<GS>>, L: Into<CheckBoxState<GS>>>(label: S, checked: L) -> Box<Self> {
        let mut child = *PlainCheckBox::new(label, checked.into());

        child = *child.delegate(|focus_state, checked_state, button: Box<dyn Widget<GS>>| {
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

            let checked_color = TupleState3::new(
                checked_state.clone(),
                EnvironmentColor::SecondarySystemBackground,
                EnvironmentColor::Accent,
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
                RoundedRectangle::new(CornerRadii::all(3.0))
                    .fill(checked_color)
                    .stroke(focus_color)
                    .stroke_style(1.0),
                IfElse::new(checked_intermediate)
                    .when_true(
                        Canvas::initialize(|_, mut context| {
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
                        Canvas::initialize(|_, mut context| {
                            context.move_to(4.0, 9.0);
                            context.line_to(7.0, 12.0);
                            context.line_to(12.0, 4.0);

                            context.set_stroke_style(EnvironmentColor::DarkText);
                            context.set_line_width(2.0);
                            context.stroke();

                            context
                        })
                    ),
                button,
            ]).frame(16.0, 16.0)
        });

        Box::new(CheckBox {
            id: Id::new_v4(),
            child,
            position: [0.0, 0.0],
            dimension: [235.0, 26.0],
        })
    }
}

impl<GS: GlobalStateContract> CommonWidget<GS> for CheckBox<GS> {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn children(&self) -> WidgetIter {
        WidgetIter::single(&self.child)
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn proxied_children(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn proxied_children_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn dimension(&self) -> Dimensions {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl<GS: GlobalStateContract> ChildRender for CheckBox<GS> {}

impl<GS: GlobalStateContract> Layout<GS> for CheckBox<GS> {
    fn flexibility(&self) -> u32 {
        5
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment) -> Dimensions {
        self.set_width(requested_size[0]);

        self.child.calculate_size(self.dimension, env);

        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position();
        let dimension = self.dimension();


        positioning(position, dimension, &mut self.child);
        self.child.position_children();
    }
}


impl<GS: GlobalStateContract> WidgetExt<GS> for CheckBox<GS> {}
