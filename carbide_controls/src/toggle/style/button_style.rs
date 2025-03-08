use carbide::draw::gradient::{Gradient, GradientPosition};
use crate::toggle::toggle_value::ToggleValue;
use crate::toggle::ToggleAction;
use crate::toggle::ToggleStyle;
use crate::UnfocusAction;
use carbide_core::accessibility::Role;
use carbide_core::color::{ColorExt, TRANSPARENT};
use carbide_core::draw::Alignment;
use carbide_core::environment::{EnvironmentColor, IntoColorReadState};
use carbide_core::focus::Focus;
use carbide_core::render::Style;
use carbide_core::state::{AnyReadState, AnyState, LocalState, Map1, Map2, Map3, Map5, ReadState, State};
use carbide_core::widget::{AnyWidget, CornerRadii, EdgeInsets, MouseArea, RoundedRectangle, Text, Widget, WidgetExt};

#[derive(Debug, Clone)]
pub struct ButtonStyle;

impl ButtonStyle {
    fn create(focus: impl State<T=Focus>, value: impl State<T=ToggleValue>, enabled: impl ReadState<T=bool>, label: Box<dyn AnyReadState<T=String>>) -> impl Widget {
        let pressed = LocalState::new(false);
        let hovered = LocalState::new(false);

        MouseArea::new(Self::widget(focus.clone(), value.clone(), enabled.clone(), label, pressed.clone(), hovered.clone()))
            .custom_on_click(ToggleAction {
                value,
                focus: focus.clone(),
                enabled,
            }).custom_on_click_outside(UnfocusAction(focus.clone()))
            .pressed(pressed)
            .hovered(hovered)
            .focused(focus.clone())
    }

    fn widget(focus: impl State<T=Focus>, value: impl State<T=ToggleValue>, enabled: impl ReadState<T=bool>, label: Box<dyn AnyReadState<T=String>>, pressed: impl ReadState<T=bool>, hovered: impl ReadState<T=bool>) -> impl Widget {
        let base_color = Map3::read_map(value.clone(), EnvironmentColor::Accent.color(), EnvironmentColor::SecondarySystemBackground.color(), |value, toggled, untoggled| {
            if *value == ToggleValue::True {
                *toggled
            } else {
                *untoggled
            }
        });

        let disabled_color = EnvironmentColor::TertiarySystemFill.color();

        let background_color = Map5::read_map(base_color, disabled_color, pressed, hovered, enabled.clone(), |col, disabled_col, pressed, hovered, enabled| {
            if !*enabled {
                return Style::Gradient(Gradient::linear(vec![disabled_col.lightened(0.05), *disabled_col], GradientPosition::Alignment(Alignment::Top), GradientPosition::Alignment(Alignment::Bottom)))
            }

            if *pressed {
                return Style::Gradient(Gradient::linear(vec![col.darkened(0.05), *col], GradientPosition::Alignment(Alignment::Top), GradientPosition::Alignment(Alignment::Bottom)))
            }

            if *hovered {
                return Style::Gradient(Gradient::linear(vec![col.lightened(0.1), *col], GradientPosition::Alignment(Alignment::Top), GradientPosition::Alignment(Alignment::Bottom)))
            }

            return Style::Gradient(Gradient::linear(vec![col.lightened(0.05), *col], GradientPosition::Alignment(Alignment::Top), GradientPosition::Alignment(Alignment::Bottom)))
        });

        let label_color = Map1::read_map(enabled, |enabled| {
            if *enabled {
                EnvironmentColor::Label
            } else {
                EnvironmentColor::TertiaryLabel
            }
        });

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

        Text::new(label)
            .foreground_color(label_color)
            .padding(EdgeInsets::vertical_horizontal(3.0, 9.0))
            .background(RoundedRectangle::new(CornerRadii::all(5.0))
                .fill(background_color)
                .stroke(EnvironmentColor::OpaqueSeparator)
                .stroke_style(1.0))
            .background(RoundedRectangle::new(CornerRadii::all(5.0))
                .stroke(outline_color)
                .stroke_style(1.0)
                .padding(-1.0))
    }
}

impl ToggleStyle for ButtonStyle {
    fn create(&self, focus: Box<dyn AnyState<T=Focus>>, value: Box<dyn AnyState<T=ToggleValue>>, enabled: Box<dyn AnyReadState<T=bool>>, label: Box<dyn AnyReadState<T=String>>) -> Box<dyn AnyWidget> {
        ButtonStyle::create(focus, value, enabled, label).boxed()
    }

    fn toggle_role(&self) -> Role {
        Role::Button
    }
}