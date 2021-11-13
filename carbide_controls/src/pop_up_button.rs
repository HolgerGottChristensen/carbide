use std::fmt::Debug;

use carbide_core::DeserializeOwned;
use carbide_core::draw::Dimension;
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::Serialize;
use carbide_core::state::{BoolState, FocusState, StateContract, StateExt, StateKey, TState, UsizeState};
use carbide_core::widget::*;
use carbide_core::widget::canvas::Canvas;

use crate::{List, PlainPopUpButton, PopupDelegate};

pub struct PopUpButton();

impl PopUpButton {
    pub fn new<T: StateContract + PartialEq + 'static, M: Into<TState<Vec<T>>>, S: Into<TState<T>>>(
        model: M,
        selected_state: S,
    ) -> Box<PlainPopUpButton<T>> {
        let mut plain = PlainPopUpButton::new(model, selected_state)
            .delegate(Self::delegate)
            .popup_item_delegate(Self::popup_item_delegate);

        plain
    }

    fn delegate<T: StateContract + PartialEq + 'static>(selected_item: TState<T>, _focused: FocusState) -> Box<dyn Widget> {
        let text = selected_item.mapped(|a: &T| format!("{:?}", a));

        ZStack::new(vec![
            RoundedRectangle::new(CornerRadii::all(3.0))
                .fill(EnvironmentColor::SecondarySystemBackground),
            HStack::new(vec![
                Padding::init(EdgeInsets::single(0.0, 0.0, 7.0, 0.0), Text::new(text)),
                Spacer::new(),
                ZStack::new(vec![
                    RoundedRectangle::new(CornerRadii::single(0.0, 0.0, 0.0, 2.0))
                        .fill(EnvironmentColor::Accent),
                    Canvas::new(|_, mut context| {
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
                    }),
                ])
                    .padding(EdgeInsets::single(0.0, 0.0, 0.0, 1.0))
                    .frame(20.0, SCALE),
            ]),
            RoundedRectangle::new(CornerRadii::all(3.0))
                .stroke_style(1.0)
                .stroke(EnvironmentColor::OpaqueSeparator),
        ]).frame(SCALE, 22)
    }

    fn popup_item_delegate<T: StateContract + PartialEq + 'static>(
        item: TState<T>, _index: UsizeState, hover_state: BoolState, _selected_state: TState<T>,
    ) -> Box<dyn Widget> {
        let background_color = hover_state.mapped_env(|hovered: &bool, _: &_, env: &Environment| {
            if *hovered {
                env.env_color(EnvironmentColor::Accent).unwrap()
            } else {
                env.env_color(EnvironmentColor::SecondarySystemBackground).unwrap()
            }
        });

        let text = item.mapped(|item: &T| format!("{:?}", item));

        ZStack::new(vec![
            Rectangle::new()
                .fill(background_color)
                .stroke(EnvironmentColor::OpaqueSeparator)
                .stroke_style(0.5),
            HStack::new(vec![
                Padding::init(
                    EdgeInsets::single(0.0, 0.0, 5.0, 0.0),
                    Text::new(text).color(EnvironmentColor::Label),
                ),
                Spacer::new(),
            ]),
        ]).frame(SCALE, 24)
    }
}
