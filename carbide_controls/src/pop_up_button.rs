use std::fmt::Debug;
use carbide_core::color::TRANSPARENT;

use carbide_core::draw::{Alignment, Color};
use carbide_core::environment::EnvironmentColor;
use carbide_core::focus::Focus;
use carbide_core::render::Style;
use carbide_core::state::{AnyReadState, AnyState, IntoReadState, IntoState, Map1, Map2, Map3, ReadState, ReadStateExtNew, State, StateContract, StateExt, TState};
use carbide_core::widget::*;
use carbide_core::widget::canvas::{Canvas, LineCap};

use crate::{PlainPopUpButton, PopupDelegate};

pub struct PopUpButton;

impl PopUpButton {
    pub fn new<T: StateContract + PartialEq, S: IntoState<T>, M: IntoReadState<Vec<T>>>(
        selected: S,
        model: M,
    ) -> PlainPopUpButton<T, TState<Focus>, S::Output, M::Output> {
        PlainPopUpButton::new(selected, model)
            .delegate(Self::delegate)
            .popup_item_delegate(Self::popup_item_delegate)
            .popup_delegate(Self::popup_delegate)
    }

    fn delegate<T: StateContract + PartialEq>(
        selected_item: Box<dyn AnyState<T=T>>,
        focused: Box<dyn AnyState<T=Focus>>,
        popup_open: Box<dyn AnyReadState<T=bool>>
    ) -> Box<dyn Widget> {
        let text = Map1::read_map(selected_item, |a| format!("{:?}", a));

        let arrows = Canvas::new(|_, mut context, _| {
            context.move_to(6.0, 9.0);
            context.line_to(10.0, 5.0);
            context.line_to(14.0, 9.0);
            context.move_to(6.0, 13.0);
            context.line_to(10.0, 17.0);
            context.line_to(14.0, 13.0);
            context.set_stroke_style(EnvironmentColor::DarkText);
            context.set_line_width(1.5);
            context.set_line_cap(LineCap::Round);
            context.stroke();

            context
        });

        let button_color = Map1::read_map(EnvironmentColor::Accent.color(), |color| {
            Style::Gradient(Gradient::linear(vec![color.lightened(0.05), *color], GradientPosition::Alignment(Alignment::Top), GradientPosition::Alignment(Alignment::Bottom)))
        });

        let outline_color = Map3::read_map(EnvironmentColor::Accent.color(), focused, popup_open, |color, focused, opened| {
            if *focused == Focus::Focused && !*opened {
                *color
            } else {
                TRANSPARENT
            }
        });

        ZStack::new(vec![
            RoundedRectangle::new(CornerRadii::all(3.0))
                .fill(EnvironmentColor::SecondarySystemBackground),
            HStack::new(vec![
                Text::new(text).padding(EdgeInsets::single(0.0, 0.0, 9.0, 0.0)).boxed(),
                Spacer::new(),
                ZStack::new(vec![
                    RoundedRectangle::new(CornerRadii::single(0.0, 0.0, 0.0, 2.0)) // TODO: Changing top_right, makes lyon mess up.
                        .fill(button_color),
                    arrows,
                ])
                    .padding(EdgeInsets::single(0.0, 0.0, 0.0, 1.0))
                    .frame_fixed_width(20.0),
            ]),
            RoundedRectangle::new(CornerRadii::all(3.0))
                .stroke_style(1.0)
                .stroke(EnvironmentColor::OpaqueSeparator),
        ])
            .background(
                RoundedRectangle::new(CornerRadii::all(3.0))
                    .stroke(outline_color)
                    .stroke_style(1.0)
                    .padding(-1.0)
            )
            .frame_fixed_height(22.0)
    }

    fn popup_item_delegate<T: StateContract + PartialEq, S: State<T=T>>(
        item: Box<dyn AnyState<T=T>>,
        _index: Box<dyn AnyReadState<T=usize>>,
        hover_state: Box<dyn AnyReadState<T=bool>>,
        _selected_state: S,
    ) -> Box<dyn Widget> {
        let text = Map1::read_map(item, |i| format!("{:?}", i));

        let background_color = Map1::read_map(hover_state, |hovered| {
            if *hovered {
                EnvironmentColor::Accent
            } else {
                EnvironmentColor::SecondarySystemBackground
            }
        });

        ZStack::new(vec![
            Rectangle::new()
                .fill(background_color)
                .stroke(EnvironmentColor::OpaqueSeparator)
                .stroke_style(0.5),
            HStack::new(vec![
                Text::new(text)
                    .color(EnvironmentColor::Label)
                    .padding(EdgeInsets::single(0.0, 0.0, 5.0, 0.0))
                    .boxed(),
                Spacer::new(),
            ]),
        ])
            .frame_fixed_height(24.0)
    }

    fn popup_delegate<T: StateContract + PartialEq, S: State<T=T>, M: ReadState<T=Vec<T>>, B: State<T=bool>>(
        model: M,
        delegate: PopupDelegate<T, S, B>,
    ) -> Box<dyn Widget> {
        VStack::new(vec![
            ForEach::new(model.ignore_writes(), delegate)
        ]).spacing(1.0)
            .padding(1.0)
            .background(*Rectangle::new().fill(EnvironmentColor::OpaqueSeparator))
    }
}
