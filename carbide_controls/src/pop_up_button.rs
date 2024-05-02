use carbide_core::color::TRANSPARENT;
use carbide_core::draw::{Alignment, Rect};
use carbide_core::environment::{Environment, EnvironmentColor, IntoColorReadState};
use carbide_core::focus::Focus;
use carbide_core::render::Style;
use carbide_core::state::{AnyReadState, AnyState, IntoReadState, IntoState, LocalState, Map1, Map3, ReadState, ReadStateExtNew, State, StateContract, Map2};
use carbide_core::widget::*;
use carbide_core::widget::canvas::{Canvas, Context, LineCap};

use crate::{PlainPopUpButton, PopupItemDelegate};

pub struct PopUpButton;

impl PopUpButton {
    pub fn new<T: StateContract + PartialEq, S: IntoState<T>, M: IntoReadState<Vec<T>>>(
        selected: S,
        model: M,
    ) -> PlainPopUpButton<T, LocalState<Focus>, S::Output, M::Output, bool> {
        PlainPopUpButton::new(selected, model)
            .delegate(Self::delegate)
            .popup_item_delegate(Self::popup_item_delegate)
            .popup_delegate(Self::popup_delegate)
    }

    fn delegate<T: StateContract + PartialEq>(
        selected_item: Box<dyn AnyState<T=T>>,
        focused: Box<dyn AnyState<T=Focus>>,
        enabled: Box<dyn AnyReadState<T=bool>>,
        text_delegate: fn(Box<dyn AnyReadState<T=T>>) -> Box<dyn AnyReadState<T=String>>,
    ) -> Box<dyn AnyWidget> {
        let text = text_delegate(selected_item.as_dyn_read());

        let mark_color = Map1::read_map(enabled.clone(), |enabled| {
            if *enabled {
                EnvironmentColor::DarkText
            } else {
                EnvironmentColor::TertiaryLabel
            }
        });

        let label_color = mark_color.clone();

        let arrows = Canvas::new(move |_rect: Rect, context: &mut Context, _env: &mut Environment| {
            context.move_to(6.0, 9.0);
            context.line_to(10.0, 5.0);
            context.line_to(14.0, 9.0);
            context.move_to(6.0, 13.0);
            context.line_to(10.0, 17.0);
            context.line_to(14.0, 13.0);
            context.set_stroke_style(mark_color.clone());
            context.set_line_width(1.5);
            context.set_line_cap(LineCap::Round);
            context.stroke();
        });

        let button_color = Map3::read_map(enabled.clone(), EnvironmentColor::Accent.color(), EnvironmentColor::TertiarySystemFill.color(), |enabled, color, disabled_color| {
            if *enabled {
                Style::Gradient(Gradient::linear(
                    vec![color.lightened(0.05), *color],
                    GradientPosition::Alignment(Alignment::Top),
                    GradientPosition::Alignment(Alignment::Bottom)
                ))
            } else {
                Style::Gradient(Gradient::linear(
                    vec![disabled_color.lightened(0.05), *disabled_color],
                    GradientPosition::Alignment(Alignment::Top),
                    GradientPosition::Alignment(Alignment::Bottom)
                ))
            }

        });

        let outline_color = Map2::read_map(EnvironmentColor::Accent.color(), focused, |color, focused| {
            if *focused == Focus::Focused {
                *color
            } else {
                TRANSPARENT
            }
        });

        ZStack::new((
            RoundedRectangle::new(CornerRadii::all(3.0))
                .fill(EnvironmentColor::SecondarySystemBackground),
            HStack::new((
                Text::new(text).color(label_color).padding(EdgeInsets::single(0.0, 0.0, 9.0, 0.0)),
                Spacer::new(),
                ZStack::new((
                    RoundedRectangle::new(CornerRadii::single(0.0, 0.0, 0.0, 2.0)) // TODO: Changing top_right, makes lyon mess up.
                        .fill(button_color),
                    arrows,
                ))
                    .padding(EdgeInsets::single(0.0, 0.0, 0.0, 1.0))
                    .frame_fixed_width(20.0),
            )),
            RoundedRectangle::new(CornerRadii::all(3.0))
                .stroke_style(1.0)
                .stroke(EnvironmentColor::OpaqueSeparator),
        ))
            .background(
                RoundedRectangle::new(CornerRadii::all(3.0))
                    .stroke(outline_color)
                    .stroke_style(1.0)
                    .padding(-1.0)
            )
            .frame_fixed_height(22.0)
            .boxed()
    }

    fn popup_item_delegate<T: StateContract + PartialEq, S: State<T=T>>(
        item: Box<dyn AnyState<T=T>>,
        _index: Box<dyn AnyReadState<T=usize>>,
        hover_state: Box<dyn AnyReadState<T=bool>>,
        _selected_state: S,
        _enabled: Box<dyn AnyReadState<T=bool>>,
        text_delegate: fn(Box<dyn AnyReadState<T=T>>) -> Box<dyn AnyReadState<T=String>>,
    ) -> Box<dyn AnyWidget> {
        let text = text_delegate(item.as_dyn_read());

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
                .stroke_style(0.5)
                .boxed(),
            HStack::new(vec![
                Text::new(text)
                    .color(EnvironmentColor::Label)
                    .padding(EdgeInsets::single(0.0, 0.0, 5.0, 0.0))
                    .boxed(),
                Spacer::new().boxed(),
            ]).boxed(),
        ])
            .frame_fixed_height(24.0)
            .boxed()
    }

    fn popup_delegate<T: StateContract + PartialEq, S: State<T=T>, M: ReadState<T=Vec<T>>>(
        model: M,
        delegate: PopupItemDelegate<T, S>,
        _enabled: Box<dyn AnyReadState<T=bool>>,
    ) -> Box<dyn AnyWidget> {
        VStack::new(ForEach::new(model.ignore_writes(), delegate))
            .spacing(1.0)
            .padding(1.0)
            .background(Rectangle::new().fill(EnvironmentColor::OpaqueSeparator))
            .boxed()
    }
}
