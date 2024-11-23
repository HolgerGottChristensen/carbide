use crate::picker::picker_action::PickerAction;
use crate::picker::picker_selection::PickerSelectionType;
use crate::picker::style::{PickerStyle};
use crate::toggle::{CheckboxStyle, ToggleValue};
use crate::UnfocusAction;
use carbide::color::{Color, TRANSPARENT};
use carbide::environment::{EnvironmentColor, IntoColorReadState};
use carbide::focus::Focus;
use carbide::state::{AnyReadState, AnyState, IntoState, LocalState, Map1, Map2};
use carbide::widget::{AnyWidget, Circle, CrossAxisAlignment, Ellipse, HStack, IfElse, MouseArea, Sequence, Text, VStack, Widget, WidgetExt, ZStack};
use std::fmt::Debug;
use crate::identifiable::AnySelectableWidget;

#[derive(Debug, Clone)]
pub struct InlineStyle;

impl InlineStyle {
    fn delegate(label: Box<dyn AnyWidget>, selected: Box<dyn AnyState<T=bool>>, enabled: Box<dyn AnyReadState<T=bool>>, picker_selection_type: PickerSelectionType) -> impl Widget {
        let focus = LocalState::new(Focus::Unfocused);

        let component = match picker_selection_type {
            PickerSelectionType::Optional |
            PickerSelectionType::Single => Self::radio_button(&selected, &enabled, &focus).boxed(),
            PickerSelectionType::Multi => CheckboxStyle::check_box(focus.clone(), IntoState::<ToggleValue>::into_state(selected.clone()), enabled.clone()).boxed(),
        };

        let label_color = Map1::read_map(enabled.clone(), |enabled| {
            if *enabled {
                EnvironmentColor::DarkText
            } else {
                EnvironmentColor::TertiaryLabel
            }
        });

        let stack = HStack::new((
            component,
            label.foreground_color(label_color)
        )).spacing(5.0);

        MouseArea::new(stack)
            .custom_on_click(PickerAction {
                value: selected,
                focus: focus.clone(),
                enabled,
            })
            .custom_on_click_outside(UnfocusAction(focus.clone()))
            .focused(focus)
    }

    fn radio_button(selected: &Box<dyn AnyState<T=bool>>, enabled: &Box<dyn AnyReadState<T=bool>>, focus: &LocalState<Focus>) -> impl Widget {
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

        let mark_color = Map1::read_map(enabled.clone(), |enabled| {
            if *enabled {
                Color::new_rgba(222, 222, 224, 255)
            } else {
                Color::new_rgba(150, 150, 150, 255)
            }
        });


        let radio = ZStack::new((
            Circle::new()
                .fill(background_color)
                .stroke(EnvironmentColor::OpaqueSeparator)
                .stroke_style(1.0),
            IfElse::new(selected.clone()).when_true(
                Ellipse::new()
                    .fill(mark_color)
                    .frame(6.0, 6.0),
            ),
        )).background(
            Circle::new()
                .stroke(outline_color)
                .stroke_style(1.0)
                .padding(-1.0)
        ).frame(14.0, 14.0);
        radio
    }
}

/*impl PickerStyle for InlineStyle {
    fn create(&self, focus: Box<dyn AnyState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, label: Box<dyn AnyReadState<T=String>>, model: Box<dyn Sequence<dyn AnySelectableWidget>>, picker_selection_type: PickerSelectionType) -> Box<dyn AnyWidget> {
        let radio_group = VStack::new(
            SelectableForEach::new(model, move |widget: Box<dyn AnyWidget>, selected: Box<dyn AnyState<T=bool>>| {
                InlineStyle::delegate(widget, selected, enabled.clone(), picker_selection_type)
            })
        ).spacing(8.0).cross_axis_alignment(CrossAxisAlignment::Start);

        let labelled = HStack::new((
            Text::new(label),
            radio_group
        )).cross_axis_alignment(CrossAxisAlignment::Start)
            .spacing(8.0);

        labelled.boxed()
    }
}*/