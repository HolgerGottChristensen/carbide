use crate::identifiable::SelectableForEach;
use crate::picker::picker_action::PickerAction;
use crate::picker::picker_selection::PickerSelectionType;
use crate::picker::style::{PickerStyle, SelectableSequence};
use crate::UnfocusAction;
use carbide::color::TRANSPARENT;
use carbide::environment::EnvironmentColor::{OpaqueSeparator, SecondarySystemBackground};
use carbide::environment::{EnvironmentColor, IntoColorReadState};
use carbide::focus::Focus;
use carbide::state::{AnyReadState, AnyState, LocalState, Map1, Map2, Map4};
use carbide::widget::{AnyWidget, EdgeInsets, HStack, MouseArea, RoundedRectangle, Text, Widget, WidgetExt, ZStack};
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct SegmentedStyle;

impl SegmentedStyle {
    fn delegate(label: Box<dyn AnyWidget>, selected: Box<dyn AnyState<T=bool>>, enabled: Box<dyn AnyReadState<T=bool>>) -> impl Widget {
        let focus = LocalState::new(Focus::Unfocused);

        let background_color = Map4::read_map(
            selected.clone(),
            enabled.clone(),
            EnvironmentColor::TertiaryLabel.color(),
            EnvironmentColor::SystemFill.color(),
            |checked, enabled, col1, col2| {
                if *enabled {
                    if *checked {
                        *col1
                    } else {
                        TRANSPARENT
                    }
                } else {
                    if *checked {
                        *col2
                    } else {
                        TRANSPARENT
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

        let label_color = Map1::read_map(enabled.clone(), |enabled| {
            if *enabled {
                EnvironmentColor::Label
            } else {
                EnvironmentColor::TertiaryLabel
            }
        });

        let button = ZStack::new((
            RoundedRectangle::new(5.0)
                .fill(background_color)
                .stroke(outline_color)
                .stroke_style(1.0),
            label
                .padding(EdgeInsets::vertical_horizontal(0.0, 7.0))
                .foreground_color(label_color)
        ));

        MouseArea::new(button)
            .custom_on_click(PickerAction {
                value: selected,
                focus: focus.clone(),
                enabled,
            })
            .custom_on_click_outside(UnfocusAction(focus.clone()))
            .focused(focus)
    }
}

impl PickerStyle for SegmentedStyle {
    fn create(&self, focus: Box<dyn AnyState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, label: Box<dyn AnyReadState<T=String>>, model: Box<dyn SelectableSequence>, picker_selection_type: PickerSelectionType) -> Box<dyn AnyWidget> {
        let radio_group = HStack::new(
            SelectableForEach::new(model, move |widget: Box<dyn AnyWidget>, selected: Box<dyn AnyState<T=bool>>| {
                SegmentedStyle::delegate(widget, selected, enabled.clone())
            })
        ).spacing(0.0)
            .frame_fixed_height(22.0)
            .background(RoundedRectangle::new(5.0)
                .fill(SecondarySystemBackground)
                .stroke(OpaqueSeparator)
                .stroke_style(1.0)
            );

        let labelled = HStack::new((
            Text::new(label).custom_flexibility(15),
            radio_group
        )).spacing(8.0);

        labelled.boxed()
    }
}