use carbide_controls::picker::{MenuStyle, Picker};
use carbide_controls::toggle::{CheckboxStyle, Toggle};
use carbide_controls::{ControlsExt, Slider, TextInput};
use carbide_core::closure;
use carbide_core::color::{TRANSPARENT, WHITE};
use carbide_core::draw::{Dimension, Position, StrokeDashCap, StrokeDashMode};
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::mouse_position::MousePositionEnvironmentExt;
use carbide_core::state::{LocalState, Map1, State};
use carbide_core::widget::canvas::{Canvas, CanvasContext, LineCap, LineJoin};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};
use std::f64::consts::PI;
use std::num::ParseFloatError;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum DrawType {
    Line,
    Circle,
    Star,
    BezierMouse,
    LineMouse,
    U,
}

fn main() {
    let mut application = Application::new();

    let draw_figure = LocalState::new(DrawType::LineMouse);
    let dash_mode = LocalState::new(StrokeDashMode::Pretty);
    let line_cap = LocalState::new(LineCap::Butt);
    let line_join = LocalState::new(LineJoin::Miter);
    let dash_start_cap = LocalState::new(StrokeDashCap::None);
    let dash_end_cap = LocalState::new(StrokeDashCap::None);

    let line_width = LocalState::new(30.0);
    let moving = LocalState::new(true);
    let moving_speed = LocalState::new(0.5);
    let dash_offset = LocalState::new(0.0);
    let dash = LocalState::new(true);
    let dash_pattern = LocalState::<Result<Vec<f64>, String>>::new(Ok(vec![2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]));


    let dash_pattern_input = Map1::map_owned(dash_pattern.clone(), |val, existing| {
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
                    Canvas::new(closure!(|ctx: &mut CanvasContext| {

                        if *$moving {
                            *$dash_offset += *$moving_speed / *$line_width;
                            ctx.request_animation_frame();
                        }

                        let dimension = ctx.dimension();
                        match $draw_figure {
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
                            DrawType::LineMouse => {
                                let mouse_position = ctx.mouse_position();

                                ctx.move_to(0.0, 0.0);
                                ctx.line_to(dimension.width / 2.0, dimension.height / 2.0);
                                ctx.line_to(mouse_position.x, mouse_position.y);
                            }
                            DrawType::U => {
                                ctx.move_to(50.0, 50.0);
                                ctx.line_to(dimension.width - 50.0, 50.0);
                                ctx.line_to(dimension.width - 50.0, dimension.height - 50.0);
                                ctx.line_to(50.0, dimension.height - 50.0);
                            }
                        }

                        ctx.set_line_width(*$line_width);

                        ctx.set_line_cap(*$line_cap);
                        ctx.set_line_join(*$line_join);
                        ctx.set_dash_mode(*$dash_mode);

                        if *$dash {
                            if let Ok(pattern) = $dash_pattern {
                                ctx.set_dash_offset(*$dash_offset);
                                ctx.set_dash_pattern(Some(pattern.clone()));
                                ctx.set_dash_start_cap(*$dash_start_cap);
                                ctx.set_dash_end_cap(*$dash_end_cap);
                            }
                        }

                        ctx.set_stroke_style(WHITE);

                        ctx.stroke();
                    })).padding(50.0)
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
                        Group::new((
                            Picker::new("Figure", draw_figure, (
                                Text::new("Line").tag(DrawType::Line),
                                Text::new("Circle").tag(DrawType::Circle),
                                Text::new("Star").tag(DrawType::Star),
                                Text::new("BezierMouse").tag(DrawType::BezierMouse),
                                Text::new("LineMouse").tag(DrawType::LineMouse),
                                Text::new("U").tag(DrawType::U),
                            )),
                            Picker::new("Dash mode", dash_mode, (
                                Text::new("Fast").tag(StrokeDashMode::Fast),
                                Text::new("Pretty").tag(StrokeDashMode::Pretty),
                            )),
                            Picker::new("Line join", line_join, (
                                Text::new("Miter").tag(LineJoin::Miter),
                                Text::new("Round").tag(LineJoin::Round),
                                Text::new("Bevel").tag(LineJoin::Bevel),
                            )),
                            Picker::new("Line cap", line_cap, (
                                Text::new("Butt").tag(LineCap::Butt),
                                Text::new("Square").tag(LineCap::Square),
                                Text::new("Round").tag(LineCap::Round),
                            )),
                            Picker::new("Start cap", dash_start_cap, (
                                Text::new("None").tag(StrokeDashCap::None),
                                Text::new("Square").tag(StrokeDashCap::Square),
                                Text::new("Round").tag(StrokeDashCap::Round),
                                Text::new("TriangleIn").tag(StrokeDashCap::TriangleIn),
                                Text::new("TriangleOut").tag(StrokeDashCap::TriangleOut),
                            )),
                            Picker::new("End cap", dash_end_cap, (
                                Text::new("None").tag(StrokeDashCap::None),
                                Text::new("Square").tag(StrokeDashCap::Square),
                                Text::new("Round").tag(StrokeDashCap::Round),
                                Text::new("TriangleIn").tag(StrokeDashCap::TriangleIn),
                                Text::new("TriangleOut").tag(StrokeDashCap::TriangleOut),
                            )),
                        )),
                        Slider::new(line_width, 1.0, 50.0).step(1.0),
                        Slider::new(moving_speed, -2.0, 2.0),
                        Toggle::new("Moving", moving),
                        Toggle::new("Dash", dash),
                        TextInput::new(dash_pattern_input),
                        Spacer::new()
                    )).padding(10.0)
                )).frame_fixed_width(300.0)
            )).spacing(2.0)
                .toggle_style(CheckboxStyle)
                .picker_style(MenuStyle)
        )
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
