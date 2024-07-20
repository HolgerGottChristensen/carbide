use std::time::Duration;
use carbide::color::ColorExt;
use carbide::draw::Color;
use carbide::environment::IntoColorReadState;
use carbide::state::{ReadStateExtNew, ReadStateExtTransition};
use carbide_core::color::{TRANSPARENT};
use carbide_core::environment::EnvironmentColor;
use carbide_core::focus::Focus;
use carbide_core::state::{IntoReadState, IntoState, LocalState, Map1, Map2, ReadState};
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
    fn call(&self, focus: impl ReadState<T=Focus>, checked: impl ReadState<T=bool>, enabled: impl ReadState<T=bool>) -> Box<dyn AnyWidget> {
        let switch_width = 38.0;
        let knob_width = 20.0;
        let transition_duration = Duration::from_secs_f64(0.15);

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

        let border_color = background_color
            .clone()
            .color()
            .lightened(0.08)
            .transition()
            .duration(transition_duration);

        let background_color = background_color
            .color()
            .transition()
            .duration(transition_duration);

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

        let knob_color = Map2::read_map(enabled.clone(), EnvironmentColor::TertiaryLabel.color(), |enabled, color| {
            if *enabled {
                Color::new_rgba(202, 202, 204, 255)
            } else {
                *color
            }
        }).transition()
            .duration(transition_duration);

        let label_color = Map1::read_map(enabled, |enabled| {
            if *enabled {
                EnvironmentColor::DarkText
            } else {
                EnvironmentColor::TertiaryLabel
            }
        }).color()
            .transition()
            .duration(transition_duration);

        let offset = switch_width / 2.0 - knob_width / 2.0 - 1.0;

        let offset = Map1::read_map(checked.clone(), move |a| { if *a { offset } else { -offset } })
            .transition()
            .duration(transition_duration);

        let switch = ZStack::new((
            Capsule::new()
                .fill(background_color)
                .stroke(border_color)
                .stroke_style(1.0),
            Ellipse::new()
                .fill(knob_color)
                .frame(knob_width, 20.0)
                .offset(offset, 0.0),
        ))
            .background(
                Capsule::new()
                    .stroke(outline_color)
                    .stroke_style(1.0)
                    .padding(-2.0)
            ).frame(switch_width, 22.0);

        HStack::new((switch, Text::new(self.label.clone()).color(label_color))).spacing(5.0).boxed()
    }
}
