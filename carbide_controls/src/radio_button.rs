use carbide_core::Color;
use std::fmt::Debug;

use carbide_core::draw::Dimension;
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::focus::Focus;
use carbide_core::state::{
    Map3, StateContract, StateExt, StateKey, TState,
};
use carbide_core::widget::*;

use crate::PlainRadioButton;

pub struct RadioButton;

impl RadioButton {
    pub fn new<T: StateContract + PartialEq + 'static>(
        label: impl Into<TState<String>>,
        reference: T,
        local_state: impl Into<TState<T>>,
    ) -> Box<PlainRadioButton<T>> {
        let mut plain =
            PlainRadioButton::new(label, reference, local_state).delegate(Self::delegate);
        plain
    }

    fn delegate(_: TState<Focus>, selected: TState<bool>) -> Box<dyn Widget> {
        let selected_color = Map3::read_map(
            selected.clone(),
            EnvironmentColor::Accent.state(),
            EnvironmentColor::SecondarySystemBackground.state(),
            |selected: &bool, selected_color: &Color, unselected_color: &Color| {
                if *selected {
                    *selected_color
                } else {
                    *unselected_color
                }
            },
        )
        .ignore_writes();

        ZStack::new(vec![
            Ellipse::new()
                .fill(selected_color)
                .stroke(EnvironmentColor::OpaqueSeparator)
                .stroke_style(1.0),
            IfElse::new(selected).when_true(
                Ellipse::new()
                    .fill(EnvironmentColor::DarkText)
                    .frame(4.0, 4.0),
            ),
        ])
        .frame(14.0, 14.0)
    }
}
