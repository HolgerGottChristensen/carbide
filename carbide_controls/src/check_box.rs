use carbide_core::Color;
use carbide_core::draw::Dimension;
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::state::{ColorState, FocusState, Map1, Map3, MapOwnedState, State, StateExt, StateKey, StringState};
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
        plain
    }

    fn delegate(_: FocusState, checked: CheckBoxState) -> Box<dyn Widget> {
        let accent = EnvironmentColor::Accent.state();
        let secondary = EnvironmentColor::SecondarySystemBackground.state();

        let checked_color = Map3::read_map(checked.clone(), accent, secondary,
        |checked: &CheckBoxValue, accent: &Color, secondary: &Color| {
            match *checked {
                CheckBoxValue::True | CheckBoxValue::Intermediate => *accent,
                CheckBoxValue::False => *secondary,
            }
        }).ignore_writes();

        let checked_true = checked.map(|check: &CheckBoxValue| {
            *check == CheckBoxValue::True
        }).ignore_writes();

        let checked_intermediate = checked.map(|check: &CheckBoxValue| {
            *check == CheckBoxValue::Intermediate
        }).ignore_writes();

        ZStack::new(vec![
            RoundedRectangle::new(CornerRadii::all(3.0))
                .fill(checked_color)
                .stroke(EnvironmentColor::OpaqueSeparator)
                .stroke_style(1.0),
            IfElse::new(checked_intermediate).when_true(Canvas::new(
                |_, mut context, _| {
                    context.move_to(4.0, 7.0);
                    context.line_to(10.0, 7.0);

                    context.set_stroke_style(EnvironmentColor::DarkText);
                    context.set_line_width(2.0);
                    context.stroke();

                    context
                },
            )),
            IfElse::new(checked_true).when_true(Canvas::new(|_, mut context, _| {
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