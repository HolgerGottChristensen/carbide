//! A simple example that demonstrates using conrod within a basic `winit` window loop, using
//! `glium` to render the `conrod_core::render::Primitives` to screen.

#[macro_use] extern crate conrod_core;
extern crate conrod_glium;
#[macro_use] extern crate conrod_winit;

extern crate glium;

mod support;

use conrod_core::{widget, Colorable, Positionable, Widget};
use glium::Surface;
use conrod_glium::Window;
use conrod_core::widget::primitive::CWidget;
use conrod_core::widget::{Rectangle, Oval, Line, Text};
use conrod_core::widget::oval::Full;

const WIDTH: u32 = 400;
const HEIGHT: u32 = 400;

fn main() {

    let mut window = Window::new("Hello world 2".to_string(), WIDTH, HEIGHT);

    window.add_font("fonts/NotoSans/NotoSans-Regular.ttf").unwrap();

    window.widgets = Some(Box::new(|ui| {
        widget_ids!(struct Ids { text });
        let ids = Ids::new(ui.widget_id_generator());

        /*widget::Text::new("Hello World!")
            .middle_of(ui.window)
            .color(conrod_core::color::WHITE)
            .font_size(32)
            .set(ids.text, ui);*/
    }));

    window.set_widgets(Rectangle::new(
        [0.0, 0.0],
        [100.0, 100.0],
        vec![
            Rectangle::new(
                [20.0,20.0],
                [60.0,60.0],
                vec![]
            ),
            Oval::new(
                [20.0,20.0],
                [60.0,60.0],
                vec![]
            ),
            Line::new(
                [0.0,0.0],
                [100.0,100.0],
                vec![]
            ),
            Text::new(
               "Hello world! Dette er tekst".to_string(),
               [100.0,200.0],
               [100.0,100.0],
               vec![
                    Rectangle::new(
                   [0.0,0.0],
                   [5.0,5.0],
                   vec![]
                    )
               ]
            )
        ]
    ));

    window.draw()
}
