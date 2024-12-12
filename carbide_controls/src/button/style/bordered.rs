use carbide::color::{ColorExt, TRANSPARENT};
use carbide::draw::Alignment;
use carbide::draw::theme::Theme;
use carbide::environment::{EnvironmentColor, IntoColorReadState};
use carbide::focus::Focus;
use carbide::render::Style;
use carbide::state::{AnyReadState, Map1, Map2, Map3, Map4, Map5};
use carbide::widget::{AnyWidget, CornerRadii, EdgeInsets, Gradient, GradientPosition, RoundedRectangle, WidgetExt, ZStack};
use crate::button::style::ButtonStyle;

#[derive(Copy, Clone, Debug)]
pub struct BorderedStyle;

impl ButtonStyle for BorderedStyle {
    fn create(&self, label: Box<dyn AnyWidget>, focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, hovered: Box<dyn AnyReadState<T=bool>>, pressed: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
        let base_color = EnvironmentColor::SecondarySystemBackground.color();

        let disabled_color = EnvironmentColor::TertiarySystemFill.color();

        let background_color = Map5::read_map(base_color, disabled_color, pressed, hovered, enabled.clone(), |col, disabled_col, pressed, hovered, enabled| {
            if !*enabled {
                return Style::Gradient(Gradient::linear(vec![disabled_col.lightened(0.05), *disabled_col], GradientPosition::Alignment(Alignment::Top), GradientPosition::Alignment(Alignment::Bottom)))
            }

            if *pressed {
                return Style::Gradient(Gradient::linear(vec![col.darkened(0.001), *col], GradientPosition::Alignment(Alignment::Top), GradientPosition::Alignment(Alignment::Bottom)))
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

        ZStack::new((
            RoundedRectangle::new(CornerRadii::all(5.0))
                .fill(background_color)
                .stroke(EnvironmentColor::OpaqueSeparator)
                .stroke_style(1.0),
            label
                .foreground_color(label_color)
                .clip_shape(RoundedRectangle::new(3.0))
                .padding(1.0),
        )).background(
            RoundedRectangle::new(CornerRadii::all(5.0))
                .stroke(outline_color)
                .stroke_style(1.0)
                .padding(-1.0)
        ).boxed()
    }
}