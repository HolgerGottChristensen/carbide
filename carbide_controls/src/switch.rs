use std::time::Duration;
use carbide::a;
use carbide::state::TransitionState;
use carbide_core::color::{TRANSPARENT};
use carbide_core::environment::EnvironmentColor;
use carbide_core::focus::Focus;
use carbide_core::state::{AnyReadState, AnyState, IntoReadState, IntoState, LocalState, Map1, Map2, ReadState, State};
use carbide_core::widget::*;

use crate::{EnabledState, PlainSwitch, PlainSwitchDelegate};

pub struct Switch;

impl Switch {
    // TODO: Consider creating a newtype wrapper macro for Switch, wrapping plainswitch, to simplify the signature of the function
    pub fn new<L: IntoReadState<String>, C: IntoState<bool>>(label: L, checked: C) -> PlainSwitch<LocalState<Focus>, C::Output, SwitchDelegate<L::Output>, EnabledState> {
        PlainSwitch::new(checked)
            .delegate(SwitchDelegate { label: label.into_read_state() })
    }
}

#[derive(Clone)]
pub struct SwitchDelegate<L: ReadState<T=String>> {
    label: L,
}

impl<L: ReadState<T=String>> PlainSwitchDelegate for SwitchDelegate<L> {
    fn call(&self, focus: Box<dyn AnyState<T=Focus>>, checked: Box<dyn AnyState<T=bool>>, enabled: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
        let switch_width = 39.0;
        let knob_width = 20.0;

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

        let offset = switch_width / 2.0 - knob_width / 2.0 - 1.0;

        let inner_offset = LocalState::new(0.0);
        let knob_offset = TransitionState::new(inner_offset.clone())
            .duration(Duration::from_secs_f64(0.15));

        let switch = ZStack::new((
            Capsule::new()
                .fill(background_color)
                .stroke(EnvironmentColor::OpaqueSeparator)
                .stroke_style(1.0),
            Ellipse::new()
                .fill(knob_color)
                .frame(knob_width, 20.0)
                .offset(knob_offset.clone(), 0.0),
        ))
            .background(
                Capsule::new()
                    .stroke(outline_color)
                    .stroke_style(1.0)
                    .padding(-1.0)
            )
            .on_change(checked.clone(),a!(|old, new| {
                if old.is_none() {
                    inner_offset.clone().set_value(if *new { offset } else { -offset });
                } else {
                    knob_offset.clone().set_value(if *new { offset } else { -offset });
                }
            }))
            .frame(switch_width, 22.0);

        HStack::new((switch, Text::new(self.label.clone()).color(label_color))).spacing(5.0).boxed()
    }
}
