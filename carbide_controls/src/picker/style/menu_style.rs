use crate::identifiable::SelectableForEach;
use crate::picker::picker_action::PickerAction;
use crate::picker::picker_selection::PickerSelectionType;
use crate::picker::style::{PickerStyle, SelectableSequence};
use crate::toggle::{CheckboxStyle, ToggleValue};
use crate::UnfocusAction;
use carbide::color::{Color, ColorExt, TRANSPARENT};
use carbide::environment::{EnvironmentColor, IntoColorReadState};
use carbide::focus::Focus;
use carbide::state::{AnyReadState, AnyState, IntoState, LocalState, Map1, Map2, Map3, RMap1};
use carbide::widget::{AnyWidget, Circle, CommonWidget, CornerRadii, CrossAxisAlignment, EdgeInsets, Ellipse, Gradient, GradientPosition, HStack, IfElse, MouseArea, Rectangle, RoundedRectangle, Spacer, Text, VStack, Widget, WidgetExt, Wrap, ZStack};
use std::fmt::Debug;
use carbide::draw::{Alignment, Dimension};
use carbide::render::Style;
use carbide::widget::canvas::{Canvas, CanvasContext, LineCap};

#[derive(Debug, Clone)]
pub struct MenuStyle;

impl MenuStyle {
    fn generate(&self, focus: Box<dyn AnyState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, label: Box<dyn AnyReadState<T=String>>, model: Box<dyn SelectableSequence>, picker_selection_type: PickerSelectionType) -> impl Widget {
        let arrows = Self::mark(&enabled);

        let label_color = Map1::read_map(enabled.clone(), |enabled| {
            if *enabled {
                EnvironmentColor::DarkText
            } else {
                EnvironmentColor::TertiaryLabel
            }
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

        let outline_color = Map2::read_map(EnvironmentColor::Accent.color(), focus, |color, focused| {
            if *focused == Focus::Focused {
                *color
            } else {
                TRANSPARENT
            }
        });

        let mark = ZStack::new((
            RoundedRectangle::new(5.0).fill(button_color),
            arrows
        ))
            .padding(3.0)
            .aspect_ratio(Dimension::new(1.0, 1.0));

        ZStack::new((
            RoundedRectangle::new(CornerRadii::all(5.0))
                .fill(EnvironmentColor::SecondarySystemBackground),
            HStack::new((
                Text::new("Teawdoiajw oaiwj doawjdiajwdoiawjdioawjda awd awdawdawdawdawd")
                    .wrap_mode(Wrap::None)
                    .color(label_color)
                    .clip()
                    .alignment(Alignment::Leading)
                    .padding(EdgeInsets::single(0.0, 0.0, 6.0, 2.0)),
                mark.custom_flexibility(15)
            )).spacing(0.0).cross_axis_alignment(CrossAxisAlignment::Start),
            RoundedRectangle::new(CornerRadii::all(5.0))
                .stroke_style(1.0)
                .stroke(EnvironmentColor::OpaqueSeparator),
        ))
            .background(
                RoundedRectangle::new(CornerRadii::all(5.0))
                    .stroke(outline_color)
                    .stroke_style(1.0)
                    .padding(-1.0)
            )
            .frame_fixed_height(22.0)
    }

    fn mark(enabled: &Box<dyn AnyReadState<T=bool>>) -> impl Widget {
        let mark_color = Map1::read_map(enabled.clone(), |enabled| {
            if *enabled {
                EnvironmentColor::DarkText
            } else {
                EnvironmentColor::TertiaryLabel
            }
        });

        let arrows = Canvas::new(move |ctx: &mut CanvasContext| {
            let padding = ctx.dimension() * 0.22;
            let arrow_width = ctx.width() * 0.4;
            let arrow_height = ctx.height() * 0.2;


            // Points for upward arrow
            let x = ctx.width() / 2.0;
            let y = ctx.height() / 2.0 - padding.height / 2.0;
            ctx.move_to(x - arrow_width / 2.0, y);
            ctx.line_to(x, y - arrow_height);
            ctx.line_to(x + arrow_width / 2.0, y);


            // Points for downward arrow
            let x = ctx.width() / 2.0;
            let y = ctx.height() / 2.0 + padding.height / 2.0;
            ctx.move_to(x - arrow_width / 2.0, y);
            ctx.line_to(x, y + arrow_height);
            ctx.line_to(x + arrow_width / 2.0, y);

            ctx.set_stroke_style(mark_color.clone());
            ctx.set_line_width(1.5);
            ctx.set_line_cap(LineCap::Round);
            ctx.stroke()
        });

        arrows
    }
}

impl PickerStyle for MenuStyle {
    fn create(&self, focus: Box<dyn AnyState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, label: Box<dyn AnyReadState<T=String>>, model: Box<dyn SelectableSequence>, picker_selection_type: PickerSelectionType) -> Box<dyn AnyWidget> {
        MenuStyle.generate(focus, enabled, label, model, picker_selection_type).boxed()
    }
}