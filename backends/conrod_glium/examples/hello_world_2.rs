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
use conrod_core::widget::{Rectangle, Oval, Line, Text, Image};
use conrod_core::widget::oval::Full;
use conrod_core::widget::primitive::v_stack::VStack;

const WIDTH: u32 = 750/2;
const HEIGHT: u32 = 1334/2;

fn main() {

    let mut window = Window::new("Hello world 2".to_string(), WIDTH, HEIGHT);

    window.add_font("fonts/NotoSans/NotoSans-Regular.ttf").unwrap();
    let rust_image = window.add_image("images/rust_hover.png").unwrap();

    window.widgets = Some(Box::new(|ui| {
        widget_ids!(struct Ids { text });
        let ids = Ids::new(ui.widget_id_generator());

        /*widget::Text::new("Hello World!")
            .middle_of(ui.window)
            .color(conrod_core::color::WHITE)
            .font_size(32)
            .set(ids.text, ui);*/
    }));

    // Rectangle::new(params!(alignment: Alignment::Leading))

    /*window.set_widgets(Rectangle::new(
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
            Image::new(
                rust_image,
                [100.0,0.0],
                [100.0,100.0],
                vec![]
            ),
            Text::new(
               "Hello world! Dette er tekst".to_string(),
               [200.0, 200.0],
               [100.0, 100.0],
               vec![
                    Rectangle::new(
                   [0.0,0.0],
                   [5.0,5.0],
                   vec![]
                    )
               ]
            )
        ]
    ));*/

    //window.set_widgets(Rectangle::initialize([100.0,100.0], vec![]));
    //window.set_widgets(Text::initialize("Hello world! \nHvad sker der i denne verden og vil den laypute rigtigt når der er en lang tekst".to_string(), vec![]));
    window.set_widgets(VStack::initialize([150.0,150.0], vec![Rectangle::initialize([100.0,100.0], vec![])]));

    window.draw()
}
