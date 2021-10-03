use carbide_core::draw::Dimension;
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::state::{ColorState, FocusState, MapOwnedState, State, StateKey, StringState};
use carbide_core::widget::*;
use carbide_core::widget::canvas::Canvas;

use crate::PlainCheckBox;
use crate::types::*;

pub struct CheckBox();

impl CheckBox {
    pub fn new<S: Into<StringState>, L: Into<CheckBoxState>>(
        label: S,
        checked: L,
    ) -> Box<PlainCheckBox> {
        let mut plain = PlainCheckBox::new(label, checked.into())
            .delegate(Self::delegate);
        /*
                plain = *plain.delegate(|focus_state, checked_state, button: Box<dyn Widget<GS>>| {

                });
        */
        plain
    }

    fn delegate(_: FocusState, checked: CheckBoxState) -> Box<dyn Widget> {
        /*let focus_color = TupleState3::new(
            focus_state,
            EnvironmentColor::OpaqueSeparator,
            EnvironmentColor::Accent,
        )
            .mapped(|(focus, primary_color, focus_color)| {
                if focus == &Focus::Focused {
                    *focus_color
                } else {
                    *primary_color
                }
            });*/

        let checked_color = checked.mapped_env(|check: &CheckBoxValue, env: &Environment| {
            match *check {
                CheckBoxValue::True | CheckBoxValue::Intermediate => {
                    env.get_color(&StateKey::Color(EnvironmentColor::Accent)).unwrap()
                }
                CheckBoxValue::False => {
                    env.get_color(&StateKey::Color(EnvironmentColor::SecondarySystemBackground)).unwrap()
                }
            }
        });

        let checked_true = checked.mapped(|check: &CheckBoxValue| { *check == CheckBoxValue::True });
        let checked_intermediate = checked.mapped(|check: &CheckBoxValue| { *check == CheckBoxValue::Intermediate });

        ZStack::new(vec![
            RoundedRectangle::new(CornerRadii::all(3.0))
                .fill(checked_color)
                .stroke(EnvironmentColor::OpaqueSeparator)
                .stroke_style(1.0),
            IfElse::new(checked_intermediate).when_true(Canvas::new(
                |_, mut context| {
                    context.move_to(4.0, 7.0);
                    context.line_to(10.0, 7.0);

                    context.set_stroke_style(EnvironmentColor::DarkText);
                    context.set_line_width(2.0);
                    context.stroke();

                    context
                },
            )),
            IfElse::new(checked_true).when_true(Canvas::new(|_, mut context| {
                context.move_to(4.0, 8.0);
                context.line_to(6.0, 10.0);
                context.line_to(10.0, 4.0);

                context.set_stroke_style(EnvironmentColor::DarkText);
                context.set_line_width(2.0);
                context.stroke();

                context
            })),
        ])
            .frame(14.0, 14.0)
    }
}