use crate::identifiable::SelectableForEach;
use crate::picker::style::{PickerStyle, SelectableSequence};
use crate::UnfocusAction;
use carbide::color::{Color, TRANSPARENT};
use carbide::environment::{EnvironmentColor, IntoColorReadState};
use carbide::focus::{Focus, FocusManager, Refocus};
use carbide::state::{AnyReadState, AnyState, LocalState, Map1, Map2, State};
use carbide::widget::{AnyWidget, Circle, CrossAxisAlignment, Ellipse, HStack, IfElse, Text, VStack, Widget, WidgetExt, ZStack};
use carbide::closure;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct RadioStyle;

impl RadioStyle {
    fn delegate(label: Box<dyn AnyWidget>, selected: Box<dyn AnyState<T=bool>>, enabled: Box<dyn AnyReadState<T=bool>>) -> impl Widget {
        let focus = LocalState::new(Focus::Unfocused);

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
                Color::new_rgba(170, 170, 170, 255)
            }
        });

        let label_color = Map1::read_map(enabled, |enabled| {
            if *enabled {
                EnvironmentColor::DarkText
            } else {
                EnvironmentColor::TertiaryLabel
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

        let stack = HStack::new((
            radio,
            label.foreground_color(label_color)
        )).spacing(5.0);

        stack.on_click(closure!(|ctx| {
            if *$focus != Focus::Focused {
                *$focus = Focus::FocusRequested;
                FocusManager::get(ctx.env_stack, |manager| {
                    manager.request_focus(Refocus::FocusRequest)
                });
            }

            *$selected = true;
        })).custom_on_click_outside(UnfocusAction(focus.clone()))
            .focused(focus)
    }
}

impl PickerStyle for RadioStyle {
    fn create(&self, focus: Box<dyn AnyState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, label: Box<dyn AnyReadState<T=String>>, model: Box<dyn SelectableSequence>) -> Box<dyn AnyWidget> {
        let radio_group = VStack::new(
            SelectableForEach::new(model, move |widget: Box<dyn AnyWidget>, selected: Box<dyn AnyState<T=bool>>| {
                RadioStyle::delegate(widget, selected, enabled.clone())
            })
        ).spacing(8.0).cross_axis_alignment(CrossAxisAlignment::Start);

        let labelled = HStack::new((
            Text::new(label),
            radio_group
        )).cross_axis_alignment(CrossAxisAlignment::Start)
            .spacing(8.0);

        labelled.boxed()
    }
}