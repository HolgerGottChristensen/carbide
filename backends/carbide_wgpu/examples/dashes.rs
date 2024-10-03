use std::f64::consts::PI;
use std::num::{ParseFloatError, ParseIntError};
use std::str::FromStr;
use std::time::Instant;
use carbide_controls::{CheckBox, PopUpButton, Slider, TextInput};
use carbide_core::color::{BLUE, GREEN, RED, TRANSPARENT, WHITE};
use carbide_core::draw::{Alignment, Dimension, Position, StrokeDashCap};
use carbide_core::environment::{Environment, EnvironmentColor, EnvironmentFontSize};
use carbide_core::state::{LocalState, Map1, ReadState, State, StateSync};
use carbide_core::widget::*;
use carbide_core::widget::canvas::{Canvas, CanvasContext, LineCap, LineJoin};
use carbide_wgpu::{Application, Window};

#[derive(Clone, Debug, PartialEq)]
enum DrawType {
    Line,
    Circle,
    Star,
    BezierMouse,
    LineMouse,
}

fn main() {
    let mut application = Application::new();

    let draw_figure = LocalState::new(DrawType::Line);
    let draw_figure_canvas = draw_figure.clone();
    let draw_figures = vec![DrawType::Line, DrawType::Circle, DrawType::Star, DrawType::BezierMouse, DrawType::LineMouse];

    let line_cap = LocalState::new(LineCap::Butt);
    let line_cap_canvas = line_cap.clone();
    let line_caps = vec![LineCap::Butt, LineCap::Square, LineCap::Round];

    let line_join = LocalState::new(LineJoin::Miter);
    let line_join_canvas = line_join.clone();
    let line_joins = vec![LineJoin::Miter, LineJoin::Round, LineJoin::Bevel];

    let dash_start_cap = LocalState::new(StrokeDashCap::None);
    let dash_start_cap_canvas = dash_start_cap.clone();
    let dash_start_caps = vec![StrokeDashCap::None, StrokeDashCap::Square, StrokeDashCap::Round, StrokeDashCap::TriangleIn, StrokeDashCap::TriangleOut];

    let dash_end_cap = LocalState::new(StrokeDashCap::None);
    let dash_end_cap_canvas = dash_end_cap.clone();
    let dash_end_caps = vec![StrokeDashCap::None, StrokeDashCap::Square, StrokeDashCap::Round, StrokeDashCap::TriangleIn, StrokeDashCap::TriangleOut];

    let line_width = LocalState::new(30.0);
    let line_width_canvas = line_width.clone();

    let moving = LocalState::new(true);
    let moving_canvas = moving.clone();

    let moving_speed = LocalState::new(0.01);
    let moving_speed_canvas = moving_speed.clone();

    let dash_offset = LocalState::new(0.0);
    let dash_offset_canvas = dash_offset.clone();

    let dash = LocalState::new(true);
    let dash_canvas = dash.clone();

    let dash_pattern = LocalState::<Result<Vec<f64>, String>>::new(Ok(vec![2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]));
    let dash_pattern_canvas = dash_pattern.clone();


    let dash_pattern_input = Map1::map_owned(dash_pattern, |val, existing| {
        match (val, existing) {
            (Ok(val), Ok(existing)) => {
                if let Ok(v) = parse_dash_pattern(existing) {
                    if *val == v {
                        return;
                    }
                }

                *existing = dash_pattern_to_string(val);
            }
            (Ok(val), existing) => *existing = Ok(dash_pattern_to_string(val)),
            (Err(val), existing) => *existing = Err(val.to_string()),
        }
    }, |new, mut val| {
        match new {
            Ok(s) | Err(s) => {
                *val = parse_dash_pattern(s).map_err(|_| s.to_string());
            }
        }
    }, Ok("".to_string()));

    application.set_scene(
        Window::new(
            "Dashes example - Carbide",
            Dimension::new(900.0, 600.0),
            HStack::new((
                ZStack::new((
                    Rectangle::new().fill(EnvironmentColor::SecondarySystemBackground),
                    Canvas::new(move |ctx: &mut CanvasContext| {
                        let mut draw_figure = draw_figure_canvas.clone();
                        draw_figure.sync(ctx.env());

                        let mut line_join = line_join_canvas.clone();
                        line_join.sync(ctx.env());

                        let mut line_cap = line_cap_canvas.clone();
                        line_cap.sync(ctx.env());

                        let mut dash_start_cap = dash_start_cap_canvas.clone();
                        dash_start_cap.sync(ctx.env());

                        let mut dash_end_cap = dash_end_cap_canvas.clone();
                        dash_end_cap.sync(ctx.env());

                        let mut line_width = line_width_canvas.clone();
                        line_width.sync(ctx.env());

                        let mut moving = moving_canvas.clone();
                        moving.sync(ctx.env());

                        let mut moving_speed = moving_speed_canvas.clone();
                        moving_speed.sync(ctx.env());

                        let mut dash_offset = dash_offset_canvas.clone();
                        dash_offset.sync(ctx.env());

                        let mut dash = dash_canvas.clone();
                        dash.sync(ctx.env());

                        let mut dash_pattern = dash_pattern_canvas.clone();
                        dash_pattern.sync(ctx.env());

                        if *moving.value() {
                            *dash_offset.value_mut() += *moving_speed.value();
                            ctx.env().request_animation_frame();
                        }

                        let dimension = ctx.dimension();
                        /*let middle = dimension / 2.0;
                        ctx.circle(middle.width, middle.height, dimension.width);*/
                        match &*draw_figure.value() {
                            DrawType::Line => {
                                ctx.move_to(0.0, 0.0);
                                ctx.line_to(dimension.width, dimension.height);
                            }
                            DrawType::Circle => {
                                ctx.circle(dimension.width / 2.0, dimension.height / 2.0, dimension.width);
                            }
                            DrawType::Star => {
                                draw_star(Position::new(dimension.width / 2.0, dimension.height / 2.0), 5, dimension.width / 2.0, dimension.width / 6.0, ctx);
                            }
                            DrawType::BezierMouse => {
                                let mouse_position = ctx.env().mouse_position();

                                ctx.move_to(0.0, 0.0);
                                ctx.bezier_curve_to(
                                    Position::new(dimension.width / 100.0 * 70.0, dimension.height / 100.0 * 5.0),
                                    Position::new(mouse_position.x, mouse_position.y),
                                    Position::new(dimension.width, dimension.width),
                                );
                            }
                            DrawType::LineMouse => {}
                        }

                        ctx.set_line_width(*line_width.value());

                        ctx.set_line_cap(*line_cap.value());
                        ctx.set_line_join(*line_join.value());

                        if *dash.value() {
                            if let Ok(pattern) = &*dash_pattern.value() {
                                ctx.set_dash_offset(*dash_offset.value());
                                ctx.set_dash_pattern(Some(pattern.clone()));
                                ctx.set_dash_start_cap(*dash_start_cap.value());
                                ctx.set_dash_end_cap(*dash_end_cap.value());
                            }
                        }

                        ctx.set_stroke_style(WHITE);

                        ctx.stroke();
                    }).padding(50.0)
                        .clip()
                        .aspect_ratio(Dimension::new(1.0, 1.0))
                )),
                ZStack::new((
                    Rectangle::new()
                        .fill(TRANSPARENT),
                    VStack::new((
                        HStack::new((
                            Text::new("Dash options: ")
                                .bold()
                                .font_size(EnvironmentFontSize::Title),
                            Spacer::new()
                        )).spacing(0.0),
                        PopUpButton::new(draw_figure, draw_figures),
                        PopUpButton::new(line_join, line_joins),
                        PopUpButton::new(line_cap, line_caps),
                        PopUpButton::new(dash_start_cap, dash_start_caps),
                        PopUpButton::new(dash_end_cap, dash_end_caps),
                        Slider::new(line_width, 1.0, 50.0).step(1.0),
                        Slider::new(moving_speed, -0.5, 0.5),
                        CheckBox::new("Moving", moving),
                        CheckBox::new("Dash", dash),
                        TextInput::new(dash_pattern_input),
                        Spacer::new()
                    )).padding(10.0)
                )).frame_fixed_width(300.0)
            )).spacing(2.0)
        ).close_application_on_window_close()
    );

    application.launch()
}

fn parse_dash_pattern(string: &str) -> Result<Vec<f64>, ParseFloatError> {
    string.split(",").map(|val| f64::from_str(val.trim())).collect::<Result<Vec<f64>, ParseFloatError>>()
}

fn dash_pattern_to_string(pattern: &Vec<f64>) -> String {
    pattern.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ")
}

fn draw_star(
    center: Position,
    number_of_spikes: u32,
    outer_radius: f64,
    inner_radius: f64,
    context: &mut CanvasContext,
) {
    let mut rotation = PI / 2.0 * 3.0;

    let center_x = center.x;
    let center_y = center.y;

    let mut x;
    let mut y;

    let step = PI / number_of_spikes as f64;

    context.begin_path();

    context.move_to(center_x, center_y - outer_radius);

    for _ in 0..number_of_spikes {
        x = center_x + rotation.cos() * outer_radius;
        y = center_y + rotation.sin() * outer_radius;

        context.line_to(x, y);
        rotation += step;

        x = center_x + rotation.cos() * inner_radius;
        y = center_y + rotation.sin() * inner_radius;
        context.line_to(x, y);
        rotation += step;
    }

    context.line_to(center_x, center_y - outer_radius);
    context.close_path();
}
