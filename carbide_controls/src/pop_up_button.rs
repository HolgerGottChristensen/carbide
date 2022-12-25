use std::fmt::Debug;

use carbide_core::Color;
use carbide_core::environment::EnvironmentColor;
use carbide_core::focus::Focus;
use carbide_core::state::{
    StateContract, StateExt, TState
};
use carbide_core::widget::*;
use carbide_core::widget::canvas::Canvas;

use crate::PlainPopUpButton;

pub struct PopUpButton;

impl PopUpButton {
    pub fn new<T: StateContract + PartialEq + 'static>(
        model: impl Into<TState<Vec<T>>>,
        selected_state: impl Into<TState<T>>,
    ) -> Box<PlainPopUpButton<T>> {
        let mut plain = PlainPopUpButton::new(model, selected_state)
            .delegate(Self::delegate)
            .popup_item_delegate(Self::popup_item_delegate);

        plain
    }

    fn delegate<T: StateContract + PartialEq + 'static>(
        selected_item: TState<T>,
        _focused: TState<Focus>,
    ) -> Box<dyn Widget> {
        let text = selected_item
            .map(|a: &T| format!("{:?}", a))
            .ignore_writes();

        let arrows = Canvas::new(|_, mut context, _| {
            context.move_to(6.0, 9.0);
            context.line_to(10.0, 5.0);
            context.line_to(14.0, 9.0);
            context.move_to(6.0, 13.0);
            context.line_to(10.0, 17.0);
            context.line_to(14.0, 13.0);
            context.set_stroke_style(EnvironmentColor::DarkText);
            context.set_line_width(1.5);
            context.stroke();

            context
        });

        ZStack::new(vec![
            RoundedRectangle::new(CornerRadii::all(3.0))
                .fill(EnvironmentColor::SecondarySystemBackground),
            HStack::new(vec![
                Padding::new(EdgeInsets::single(0.0, 0.0, 7.0, 0.0), Text::new(text)),
                Spacer::new(),
                ZStack::new(vec![
                    RoundedRectangle::new(CornerRadii::single(0.0, 0.0, 0.0, 2.0))
                        .fill(EnvironmentColor::Accent),
                    arrows,
                ])
                .padding(EdgeInsets::single(0.0, 0.0, 0.0, 1.0))
                .frame_fixed_width(20.0),
            ]),
            RoundedRectangle::new(CornerRadii::all(3.0))
                .stroke_style(1.0)
                .stroke(EnvironmentColor::OpaqueSeparator),
        ])
        .frame_fixed_height(22)
    }

    fn popup_item_delegate<T: StateContract + PartialEq + 'static>(
        item: TState<T>,
        _index: TState<usize>,
        hover_state: TState<bool>,
        _selected_state: TState<T>,
    ) -> Box<dyn Widget> {
        let background_color: TState<Color> = hover_state
            .choice(
                EnvironmentColor::Accent.state(),
                EnvironmentColor::SecondarySystemBackground.state(),
            )
            .ignore_writes();

        let text = item.map(|item: &T| format!("{:?}", item)).ignore_writes();

        ZStack::new(vec![
            Rectangle::new()
                .fill(background_color)
                .stroke(EnvironmentColor::OpaqueSeparator)
                .stroke_style(0.5),
            HStack::new(vec![
                Padding::new(
                    EdgeInsets::single(0.0, 0.0, 5.0, 0.0),
                    Text::new(text).color(EnvironmentColor::Label),
                ),
                Spacer::new(),
            ]),
        ])
        .frame_fixed_height(24)
    }
}
