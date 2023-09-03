use carbide_core::color::{TRANSPARENT};
use carbide_core::draw::Color;
use carbide_core::environment::EnvironmentColor;
use carbide_core::focus::Focus;
use carbide_core::state::{AnyState, IntoReadState, IntoState, Map1, Map2, ReadState, TState};
use carbide_core::widget::*;

use crate::{PlainSwitch, PlainSwitchDelegate};

pub struct Switch;

impl Switch {
    // TODO: Consider creating a newtype wrapper macro for Switch, wrapping plainswitch, to simplify the signature of the function
    pub fn new<L: IntoReadState<String>, C: IntoState<bool>>(label: L, checked: C) -> PlainSwitch<TState<Focus>, C::Output, SwitchDelegate<L::Output>> {
        PlainSwitch::new(checked)
            .delegate(SwitchDelegate { label: label.into_read_state() })
    }
}

#[derive(Clone)]
pub struct SwitchDelegate<L: ReadState<T=String>> {
    label: L,
}

impl<L: ReadState<T=String>> PlainSwitchDelegate for SwitchDelegate<L> {
    fn call(&self, focus: Box<dyn AnyState<T=Focus>>, checked: Box<dyn AnyState<T=bool>>) -> Box<dyn Widget> {
        let background_color = Map1::read_map(
            checked.clone(),
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

        let switch = ZStack::new(vec![
            Capsule::new()
                .fill(background_color)
                .stroke(EnvironmentColor::OpaqueSeparator)
                .stroke_style(1.0),
            IfElse::new(checked)
                .when_true(*HStack::new(vec![
                    Spacer::new(),
                    Ellipse::new()
                        .fill(EnvironmentColor::DarkText)
                        .frame(20.0, 20.0).boxed(),
                ]))
                .when_false(*HStack::new(vec![
                    Ellipse::new()
                        .fill(EnvironmentColor::DarkText)
                        .frame(20.0, 20.0).boxed(),
                    Spacer::new(),
                ]))
                .padding(2.0)
                .boxed(),
        ])
            .background(
                Capsule::new()
                    .stroke(border_color)
                    .stroke_style(1.0)
                    .padding(-1.0)
            )
            .frame(39.0, 22.0);

        HStack::new(vec![switch, Text::new(self.label.clone())]).spacing(5.0)
    }
}
