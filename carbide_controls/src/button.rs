use carbide::color::ColorExt;
use carbide_core::color::TRANSPARENT;
use carbide_core::draw::Alignment;
use carbide_core::environment::EnvironmentColor;
use carbide_core::environment::IntoColorReadState;
use carbide_core::focus::Focus;
use carbide_core::render::Style;
use carbide_core::state::{AnyReadState, LocalState, Map1, Map2, Map3, Map5, ReadState};
use carbide_core::widget::*;

use crate::{EnabledState, PlainButton, PlainButtonDelegate};

pub struct Button;

impl Button {
    // TODO: Consider creating a newtype wrapper macro for Button, wrapping plainbutton, to simplify the signature of the function
    pub fn new<L: IntoWidget, A: Action + Clone + 'static>(label: L, action: A) -> PlainButton<LocalState<Focus>, A, ButtonDelegate<L::Output, bool>, EnabledState, LocalState<bool>, LocalState<bool>> {
        PlainButton::new(action)
            .delegate(ButtonDelegate { label: label.into_widget(), is_primary: false })
    }

    pub fn new_primary<L: IntoWidget, A: Action + Clone + 'static>(label: L, action: A) -> PlainButton<LocalState<Focus>, A, ButtonDelegate<L::Output, bool>, EnabledState, LocalState<bool>, LocalState<bool>> {
        PlainButton::new(action)
            .delegate(ButtonDelegate { label: label.into_widget(), is_primary: true })
    }
}

#[derive(Clone)]
pub struct ButtonDelegate<L: Widget + WidgetExt, P: ReadState<T=bool>> {
    label: L,
    is_primary: P,
}

impl<L: Widget + WidgetExt, P: ReadState<T=bool>> PlainButtonDelegate for ButtonDelegate<L, P> {
    type Output = Box<dyn AnyWidget>;
    fn call(&self, focus: Box<dyn AnyReadState<T=Focus>>, hovered: Box<dyn AnyReadState<T=bool>>, pressed: Box<dyn AnyReadState<T=bool>>, enabled: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
        let base_color = Map3::read_map(self.is_primary.clone(), EnvironmentColor::Accent.color(), EnvironmentColor::SecondarySystemBackground.color(), |is_primary, primary, secondary| {
            if *is_primary {
                *primary
            } else {
                *secondary
            }
        });

        let disabled_color = EnvironmentColor::TertiarySystemFill.color();

        let background_color = Map5::read_map(base_color, disabled_color, pressed, hovered, enabled.clone(), |col, disabled_col, pressed, hovered, enabled| {
            if !*enabled {
                return Style::Gradient(Gradient::linear(vec![disabled_col.lightened(0.05), *disabled_col], GradientPosition::Alignment(Alignment::Top), GradientPosition::Alignment(Alignment::Bottom)))
            }

            if *pressed {
                return Style::Gradient(Gradient::linear(vec![col.darkened(0.05), *col], GradientPosition::Alignment(Alignment::Top), GradientPosition::Alignment(Alignment::Bottom)))
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
            self.label
                .clone()
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
