use carbide_core::color::{TRANSPARENT};
use carbide_core::environment::EnvironmentColor;
use carbide_core::focus::Focus;
use carbide_core::state::{AnyReadState, IntoReadState, IntoState, Map1, Map2, ReadState, TState};
use carbide_core::widget::*;
use carbide_core::widget::canvas::Canvas;

use crate::{CheckBoxValue, PlainCheckBox, PlainCheckBoxDelegate};

pub struct CheckBox;

impl CheckBox {
    // TODO: Consider creating a newtype wrapper macro for CheckBox, wrapping plaincheckbox, to simplify the signature of the function
    pub fn new<L: IntoReadState<String>, C: IntoState<CheckBoxValue>>(label: L, checked: C) -> PlainCheckBox<TState<Focus>, C::Output, CheckBoxDelegate<L::Output>> {
        PlainCheckBox::new(checked)
            .delegate(CheckBoxDelegate { label: label.into_read_state() })
    }
}

#[derive(Clone)]
pub struct CheckBoxDelegate<L: ReadState<T=String>> {
    label: L,
}

impl<L: ReadState<T=String>> PlainCheckBoxDelegate for CheckBoxDelegate<L> {
    fn call(&self, focus: Box<dyn AnyReadState<T=Focus>>, checked: Box<dyn AnyReadState<T=CheckBoxValue>>) -> Box<dyn Widget> {
        let background_color = Map1::read_map(checked.clone(), |value| {
            match *value {
                CheckBoxValue::True | CheckBoxValue::Intermediate => EnvironmentColor::Accent,
                CheckBoxValue::False => EnvironmentColor::SecondarySystemBackground,
            }
        });

        let checked_true =
            Map1::read_map(checked.clone(), |value| *value == CheckBoxValue::True);

        let checked_intermediate =
            Map1::read_map(checked.clone(), |value| *value == CheckBoxValue::Intermediate);

        let border_color = Map2::read_map(
            focus.clone(),
            EnvironmentColor::Accent.color(),
            |focus, color| {
                if *focus == Focus::Focused {
                    *color
                } else {
                    TRANSPARENT
                }
            }
        );

        let check_box = ZStack::new(vec![
            RoundedRectangle::new(CornerRadii::all(3.0))
                .fill(background_color)
                .stroke(EnvironmentColor::OpaqueSeparator)
                .stroke_style(1.0),
            IfElse::new(checked_intermediate).when_true(*Canvas::new(|_, mut context, _| {
                context.move_to(4.0, 7.0);
                context.line_to(10.0, 7.0);
                context.set_stroke_style(EnvironmentColor::DarkText);
                context.set_line_width(2.0);
                context.stroke();
                context
            })),
            IfElse::new(checked_true).when_true(*Canvas::new(|_, mut context, _| {
                context.move_to(4.0, 8.0);
                context.line_to(6.0, 10.0);
                context.line_to(10.0, 4.0);
                context.set_stroke_style(EnvironmentColor::DarkText);
                context.set_line_width(2.0);
                context.stroke();
                context
            })),
        ])
            .background(
                RoundedRectangle::new(CornerRadii::all(4.0))
                    .stroke(border_color)
                    .stroke_style(1.0)
                    .padding(-1.0)
            )
            .frame(14.0, 14.0);

        HStack::new(vec![check_box, Text::new(self.label.clone())]).spacing(5.0)
    }
}