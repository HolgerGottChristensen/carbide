use crate::toggle::toggle_value::ToggleValue;
use crate::toggle::ToggleAction;
use crate::toggle::ToggleStyle;
use crate::UnfocusAction;
use carbide_core::accessibility::Role;
use carbide_core::color::{Color, ColorExt, TRANSPARENT};
use carbide_core::environment::{EnvironmentColor, IntoColorReadState};
use carbide_core::focus::Focus;
use carbide_core::state::{AnyReadState, AnyState, Map1, Map2, ReadState, ReadStateExtTransition, State};
use carbide_core::widget::{AnyWidget, Capsule, Ellipse, HStack, MouseArea, Text, Widget, WidgetExt, ZStack};
use carbide_core::time::*;

#[derive(Debug, Clone)]
pub struct SwitchStyle;

impl SwitchStyle {
    fn create(focus: impl State<T=Focus>, value: impl State<T=ToggleValue>, enabled: impl ReadState<T=bool>, label: Box<dyn AnyReadState<T=String>>) -> impl Widget {
        MouseArea::new(Self::widget(focus.clone(), value.clone(), enabled.clone(), label))
            .custom_on_click(ToggleAction {
                value,
                focus: focus.clone(),
                enabled,
            }).custom_on_click_outside(UnfocusAction(focus.clone()))
            .focused(focus.clone())
    }

    fn widget(focus: impl State<T=Focus>, value: impl State<T=ToggleValue>, enabled: impl ReadState<T=bool>, label: Box<dyn AnyReadState<T=String>>) -> impl Widget {
        let switch_width = 26.0;
        let knob_diameter = 13.0;
        let transition_duration = Duration::from_secs_f64(0.15);

        let background_color = Map2::read_map(
            value.clone(),
            enabled.clone(),
            |checked, enabled| {
                if *enabled {
                    if *checked == ToggleValue::True {
                        EnvironmentColor::Accent
                    } else {
                        EnvironmentColor::SecondarySystemBackground
                    }
                } else {
                    if *checked == ToggleValue::True {
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
                Color::new_rgba(222, 222, 224, 255)
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

        let offset = switch_width / 2.0 - knob_diameter / 2.0 - 1.0;

        let offset = Map1::read_map(value, move |a| { if *a == ToggleValue::True { offset } else { -offset } })
            .transition()
            .duration(transition_duration);

        let switch = ZStack::new((
            Capsule::new()
                .fill(background_color)
                .stroke(border_color)
                .stroke_style(1.0),
            Ellipse::new()
                .fill(knob_color)
                .frame(knob_diameter, knob_diameter)
                .offset(offset, 0.0),
        ))
            .background(
                Capsule::new()
                    .stroke(outline_color)
                    .stroke_style(1.0)
                    .padding(-2.0)
            ).frame(switch_width, knob_diameter + 2.0);

        HStack::new((Text::new(label).color(label_color), switch)).spacing(7.0)
    }
}

impl ToggleStyle for SwitchStyle {
    fn create(&self, focus: Box<dyn AnyState<T=Focus>>, value: Box<dyn AnyState<T=ToggleValue>>, enabled: Box<dyn AnyReadState<T=bool>>, label: Box<dyn AnyReadState<T=String>>) -> Box<dyn AnyWidget> {
        SwitchStyle::create(focus, value, enabled, label).boxed()
    }

    fn toggle_role(&self) -> Role {
        Role::Switch
    }
}