use std::f64::consts::PI;
use chrono::{Local, Timelike};

use carbide::color::{Color, ColorExt, WHITE};
use carbide::draw::{Dimension, Position, Rect};
use carbide::environment::*;
use carbide::widget::canvas::*;
use carbide::widget::*;
use carbide::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "Fractal clock example",
            Dimension::new(600.0, 600.0),
            Canvas::new(FractalClock {
                color: WHITE,
                depth: 10,
                scale: 0.9,
                opacity: 0.6,
            }).frame(100.0, 100.0),
        ).close_application_on_window_close()
    );

    application.launch()
}

#[derive(Copy, Clone)]
struct Angles {
    seconds: f64,
    minutes: f64,
    hours: f64,
}

impl Angles {
    fn current() -> Self {
        let time = Local::now();

        fn ratio_to_radians(ratio: f64) -> f64 {
            ratio * 2.0 * PI - PI / 2.0
        }

        let seconds = (time.second() as f64 * 1000.0 + time.timestamp_subsec_millis() as f64) / 1000.0;
        let minutes = time.minute() as f64 + seconds / 60.0;
        let hours = (time.hour() % 12) as f64 + minutes / 60.0;

        Angles {
            seconds: ratio_to_radians(seconds / 60.0),
            minutes: ratio_to_radians(minutes / 60.0),
            hours: ratio_to_radians(hours / 12.0),
        }
    }
}

#[derive(Clone)]
struct FractalClock {
    color: Color,
    depth: u32,
    scale: f64,
    opacity: f64,
}

// https://codepen.io/slavanossar/pen/RezLEa
impl FractalClock {
    fn draw(&self, center: Position, angle: Angles, context: &mut CanvasContext) {
        context.begin_path();
        context.move_to(center.x, center.y);
        context.line_to(center.x + angle.hours.cos() * 50.0, center.y + angle.hours.sin() * 50.0);
        context.stroke();

        self.draw_minutes_seconds(center, 0.0, angle, self.depth, 100.0, 1.0, context);
    }

    fn draw_minutes_seconds(&self, center: Position, angle_offset: f64, angle: Angles, depth: u32, length: f64, alpha: f64, context: &mut CanvasContext) {

        context.set_stroke_style(self.color.alpha(alpha as f32));

        let minute_position = Position::new(
            center.x + (angle_offset + angle.minutes).cos() * length,
            center.y + (angle_offset + angle.minutes).sin() * length
        );

        let second_position = Position::new(
            center.x + (angle_offset + angle.seconds).cos() * length,
            center.y + (angle_offset + angle.seconds).sin() * length
        );

        context.begin_path();
        context.move_to(center.x, center.y);
        context.line_to(minute_position.x, minute_position.y);
        context.stroke();

        context.begin_path();
        context.move_to(center.x, center.y);
        context.line_to(second_position.x, second_position.y);
        context.stroke();

        if depth != 0 {
            self.draw_minutes_seconds(second_position, angle.seconds - angle.hours - PI + angle_offset, angle, depth - 1, length * self.scale, alpha * self.opacity, context);
            self.draw_minutes_seconds(minute_position, angle.minutes - angle.hours - PI + angle_offset, angle, depth - 1, length * self.scale, alpha * self.opacity, context);
        }
    }
}


impl Context for FractalClock {
    fn call(&mut self, context: &mut CanvasContext) {
        context.env().request_animation_frame();

        let angles = Angles::current();

        let center = Position::new(
            context.dimension().width / 2.0,
            context.dimension().height / 2.0,
        );

        context.set_stroke_style(self.color);
        context.set_line_cap(LineCap::Butt);
        self.draw(center, angles, context);
    }
}