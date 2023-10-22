use carbide_core::color::{TRANSPARENT};
use carbide_core::draw::Color;
use carbide_core::environment::EnvironmentColor;
use carbide_core::focus::Focus;
use carbide_core::state::{AnyReadState, AnyState, IntoReadState, IntoState, LocalState, Map1, Map2, ReadState, TState};
use carbide_core::widget::*;

use crate::{PlainSwitch, PlainSwitchDelegate};

pub struct Switch;

impl Switch {
    // TODO: Consider creating a newtype wrapper macro for Switch, wrapping plainswitch, to simplify the signature of the function
    pub fn new<L: IntoReadState<String>, C: IntoState<bool>>(label: L, checked: C) -> PlainSwitch<LocalState<Focus>, C::Output, SwitchDelegate<L::Output>, bool> {
        PlainSwitch::new(checked)
            .delegate(SwitchDelegate { label: label.into_read_state() })
    }
}

#[derive(Clone)]
pub struct SwitchDelegate<L: ReadState<T=String>> {
    label: L,
}

impl<L: ReadState<T=String>> PlainSwitchDelegate for SwitchDelegate<L> {
    fn call(&self, focus: Box<dyn AnyState<T=Focus>>, checked: Box<dyn AnyState<T=bool>>, enabled: Box<dyn AnyReadState<T=bool>>) -> Box<dyn Widget> {
        let background_color = Map2::read_map(
            checked.clone(),
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

        let knob_color = Map1::read_map(enabled, |enabled| {
            if *enabled {
                EnvironmentColor::DarkText
            } else {
                EnvironmentColor::TertiaryLabel
            }
        });

        let label_color = knob_color.clone();

        let switch = ZStack::new(vec![
            Capsule::new()
                .fill(background_color)
                .stroke(EnvironmentColor::OpaqueSeparator)
                .stroke_style(1.0)
                .boxed(),
            IfElse::new(checked)
                .when_true(HStack::new(vec![
                    Spacer::new(),
                    Ellipse::new()
                        .fill(knob_color.clone())
                        .frame(20.0, 20.0).boxed(),
                ]))
                .when_false(HStack::new(vec![
                    Ellipse::new()
                        .fill(knob_color)
                        .frame(20.0, 20.0).boxed(),
                    Spacer::new(),
                ]))
                .padding(2.0)
                .boxed(),
        ])
            .background(
                Capsule::new()
                    .stroke(outline_color)
                    .stroke_style(1.0)
                    .padding(-1.0)
            )
            .frame(39.0, 22.0)
            .boxed();

        HStack::new(vec![switch, Text::new(self.label.clone()).color(label_color).boxed()]).spacing(5.0).boxed()
    }
}
