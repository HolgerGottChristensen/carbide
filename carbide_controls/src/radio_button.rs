use carbide_core::color::{TRANSPARENT};
use carbide_core::environment::EnvironmentColor;
use carbide_core::focus::Focus;
use carbide_core::state::{AnyReadState, IntoReadState, IntoState, Map1, Map2, ReadState, StateContract, TState};
use carbide_core::widget::*;

use crate::{PlainRadioButton, PlainRadioButtonDelegate};

pub struct RadioButton;

impl RadioButton {
    // TODO: Consider creating a newtype wrapper macro for Switch, wrapping plainswitch, to simplify the signature of the function
    pub fn new<L: IntoReadState<String>, T: StateContract + PartialEq, S: IntoState<T>>(label: L, reference: T, selected: S) -> PlainRadioButton<T, TState<Focus>, S::Output, RadioButtonDelegate<L::Output>> {
        PlainRadioButton::new(reference, selected)
            .delegate(RadioButtonDelegate { label: label.into_read_state() })
    }
}

#[derive(Clone)]
pub struct RadioButtonDelegate<L: ReadState<T=String>> {
    label: L,
}

impl<L: ReadState<T=String>> PlainRadioButtonDelegate for RadioButtonDelegate<L> {
    fn call(&self, focus: Box<dyn AnyReadState<T=Focus>>, selected: Box<dyn AnyReadState<T=bool>>) -> Box<dyn Widget> {
        let background_color = Map1::read_map(
            selected.clone(),
            |checked| {
                if *checked {
                    EnvironmentColor::Accent
                } else {
                    EnvironmentColor::SecondarySystemBackground
                }
            }
        );

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

        let radio = ZStack::new(vec![
            Circle::new()
                .fill(background_color)
                .stroke(EnvironmentColor::OpaqueSeparator)
                .stroke_style(1.0),
            IfElse::new(selected).when_true(
                *Ellipse::new()
                    .fill(EnvironmentColor::DarkText)
                    .frame(4.0, 4.0),
            ),
        ]).background(
            Circle::new()
                .stroke(border_color)
                .stroke_style(1.0)
                .padding(-1.0)
        ).frame(14.0, 14.0);

        HStack::new(vec![radio, Text::new(self.label.clone())]).spacing(5.0)

    }
}