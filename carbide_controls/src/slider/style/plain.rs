use carbide::closure;
use carbide::color::TRANSPARENT;
use carbide::environment::{EnvironmentColor, IntoColorReadState};
use carbide::focus::Focus;
use carbide::state::{AnyReadState, AnyState, Map1, Map2, ReadStateExtNew};
use carbide::widget::{AnyWidget, Capsule, Circle, IfElse, RoundedRectangle, WidgetExt};
use carbide::widget::canvas::{Canvas, CanvasContext};
use crate::slider::style::SliderStyle;

#[derive(Copy, Clone, Debug)]
pub struct PlainStyle;

impl SliderStyle for PlainStyle {
    fn create_thumb(&self, focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, percent: Box<dyn AnyReadState<T=f64>>, stepped: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {

        let thumb_color = Map1::read_map(enabled.clone(), |enabled| {
            if *enabled {
                EnvironmentColor::DarkText
            } else {
                EnvironmentColor::TertiaryLabel
            }
        });

        let thumb_overlay_color = Map1::read_map(enabled, |enabled| {
            if *enabled {
                EnvironmentColor::Gray
            } else {
                EnvironmentColor::Gray4
            }
        });

        let canvas = Canvas::new(closure!(|ctx: &mut CanvasContext| {
            ctx.circle(ctx.width() / 2.0, ctx.height() / 2.0, ctx.width());

            ctx.set_fill_style(*#thumb_color);
            ctx.fill();

            if *#percent < 0.0 {
                ctx.begin_path();
                ctx.move_to(ctx.width() / 10.0 * 6.0, ctx.width() / 10.0 * 3.0);
                ctx.line_to(ctx.width() / 10.0 * 4.0, ctx.width() / 10.0 * 5.0);
                ctx.line_to(ctx.width() / 10.0 * 6.0, ctx.width() / 10.0 * 7.0);
                ctx.set_stroke_style(*#thumb_overlay_color);
                ctx.stroke();
            } else if *#percent > 1.0 {
                ctx.begin_path();
                ctx.move_to(ctx.width() / 10.0 * 4.0, ctx.width() / 10.0 * 3.0);
                ctx.line_to(ctx.width() / 10.0 * 6.0, ctx.width() / 10.0 * 5.0);
                ctx.line_to(ctx.width() / 10.0 * 4.0, ctx.width() / 10.0 * 7.0);
                ctx.set_stroke_style(*#thumb_overlay_color);
                ctx.stroke();
            }
        }));

        IfElse::new(stepped)
            .when_true(RoundedRectangle::new(2.0).fill(thumb_color.clone()).frame(8.0, 15.0))
            .when_false(canvas.frame(15.0, 15.0))
            .boxed()
    }

    fn create_track(&self, focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, percent: Box<dyn AnyReadState<T=f64>>, stepped: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
        let track_color = Map1::read_map(enabled, |enabled| {
            if *enabled {
                EnvironmentColor::Accent
            } else {
                EnvironmentColor::SystemFill
            }
        });

        Capsule::new()
            .fill(track_color)
            .frame_fixed_height(5.0)
            .boxed()
    }

    fn create_background(&self, focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, percent: Box<dyn AnyReadState<T=f64>>, stepped: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
        let outline_color = Map2::read_map(focus, EnvironmentColor::Accent.color(), |focus, color| {
            if *focus == Focus::Focused {
                *color
            } else {
                TRANSPARENT
            }
        });

        let background_color = Map1::read_map(enabled, |enabled| {
            if *enabled {
                EnvironmentColor::SystemFill
            } else {
                EnvironmentColor::TertiarySystemFill
            }
        });

        Capsule::new()
            .fill(background_color)
            .background(
                Capsule::new()
                    .stroke(outline_color)
                    .stroke_style(1.0)
                    .padding(-2.0)
            )
            .frame_fixed_height(5.0)
            .boxed()
    }
}