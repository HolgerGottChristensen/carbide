use carbide_core::color::TRANSPARENT;
use carbide_core::environment::{EnvironmentColor, IntoColorReadState};
use carbide_core::focus::Focus;
use carbide_core::state::{IntoReadState, IntoState, LocalState, Map1, Map2, ReadState, StateContract};
use carbide_core::widget::*;

use crate::{PlainRadioButton, PlainRadioButtonDelegate};

pub struct RadioButton;

impl RadioButton {
    // TODO: Consider creating a newtype wrapper macro for Switch, wrapping plainswitch, to simplify the signature of the function
    pub fn new<L: IntoReadState<String>, T: StateContract + PartialEq, S: IntoState<T>>(label: L, reference: T, selected: S) -> PlainRadioButton<T, LocalState<Focus>, S::Output, RadioButtonDelegate<L::Output>, bool> {
        PlainRadioButton::new(reference, selected)
            .delegate(RadioButtonDelegate { label: label.into_read_state() })
    }
}

#[derive(Clone)]
pub struct RadioButtonDelegate<L: ReadState<T=String>> {
    label: L,
}

impl<L: ReadState<T=String>> PlainRadioButtonDelegate for RadioButtonDelegate<L> {
    fn call(&self, focus: impl ReadState<T=Focus>, selected: impl ReadState<T=bool>, enabled: impl ReadState<T=bool>) -> Box<dyn AnyWidget> {
        let background_color = Map2::read_map(
            selected.clone(),
            enabled.clone(),
            |checked, enabled| {
                if *enabled {
                    if *checked {
                        EnvironmentColor::Accent
                    } else {
                        EnvironmentColor::SecondarySystemBackground
                    }
                } else {
                    if *checked {
                        EnvironmentColor::SecondarySystemFill
                    } else {
                        EnvironmentColor::QuaternarySystemFill
                    }
                }

            }
        );

        let outline_color = Map2::read_map(
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

        let mark_color = Map1::read_map(enabled, |enabled| {
            if *enabled {
                EnvironmentColor::DarkText
            } else {
                EnvironmentColor::TertiaryLabel
            }
        });

        let label_color = mark_color.clone();

        let radio = ZStack::new((
            Circle::new()
                .fill(background_color)
                .stroke(EnvironmentColor::OpaqueSeparator)
                .stroke_style(1.0),
            IfElse::new(selected).when_true(
                Ellipse::new()
                    .fill(mark_color)
                    .frame(4.0, 4.0),
            ),
        )).background(
            Circle::new()
                .stroke(outline_color)
                .stroke_style(1.0)
                .padding(-1.0)
        ).frame(14.0, 14.0);

        HStack::new((
            radio,
            Text::new(self.label.clone())
                .color(label_color)
        )).spacing(5.0).boxed()
    }
}