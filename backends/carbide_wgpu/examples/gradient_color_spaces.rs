use carbide_core::draw::{Alignment, Color, Dimension};
use carbide_core::draw::color_space::ColorSpace;
use carbide_core::draw::gradient::Gradient;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let color_spaces = vec![
        ColorSpace::Linear,
        ColorSpace::OkLAB,
        ColorSpace::Srgb,
        ColorSpace::Xyz,
        ColorSpace::Cielab,
        ColorSpace::HSL
    ];

    application.set_scene(Window::new(
        "Gradient colorspace example - Carbide",
        Dimension::new(1000.0, 600.0),
        HStack::new((
            VStack::new((
                VStack::new(
                    ForEach::new(color_spaces.clone(), |color_space: &ColorSpace, _| {
                        let gradient = Gradient::linear(vec![
                            Color::new_rgb(0, 0, 255),
                            Color::new_rgb(255, 255, 255),
                        ], Alignment::Leading, Alignment::Trailing)
                            .color_space(*color_space);

                        Rectangle::new()
                            .fill(gradient)
                            .frame(300.0, 30.0)
                    })
                ),
                VStack::new(
                    ForEach::new(color_spaces.clone(), |color_space: &ColorSpace, _| {
                        let gradient = Gradient::linear(vec![
                            Color::new_rgb(0, 0, 0),
                            Color::new_rgb(255, 255, 255),
                        ], Alignment::Leading, Alignment::Trailing)
                            .color_space(*color_space);

                        Rectangle::new()
                            .fill(gradient)
                            .frame(300.0, 30.0)
                    })
                )
            )).spacing(20.0),

            VStack::new((
                VStack::new(
                    ForEach::new(color_spaces.clone(), |color_space: &ColorSpace, _| {
                        let gradient = Gradient::linear(vec![
                            Color::new_rgb(255, 0, 0),
                            Color::new_rgb(0, 0, 255),
                        ], Alignment::Leading, Alignment::Trailing)
                            .color_space(*color_space);

                        Rectangle::new()
                            .fill(gradient)
                            .frame(300.0, 30.0)
                    })
                ),
                VStack::new(
                    ForEach::new(color_spaces.clone(), |color_space: &ColorSpace, _| {
                        let gradient = Gradient::linear(vec![
                            Color::new_rgb(255, 0, 0),
                            Color::new_rgb(0, 255, 0),
                        ], Alignment::Leading, Alignment::Trailing)
                            .color_space(*color_space);

                        Rectangle::new()
                            .fill(gradient)
                            .frame(300.0, 30.0)
                    })
                )
            )).spacing(20.0),

            VStack::new((
                VStack::new(
                    ForEach::new(color_spaces.clone(), |color_space: &ColorSpace, _| {
                        let gradient = Gradient::linear(vec![
                            Color::new_rgb(0, 0, 255),
                            Color::new_rgb(255, 255, 0),
                        ], Alignment::Leading, Alignment::Trailing)
                            .color_space(*color_space);

                        Rectangle::new()
                            .fill(gradient)
                            .frame(300.0, 30.0)
                    })
                ),
                VStack::new(
                    ForEach::new(color_spaces.clone(), |color_space: &ColorSpace, _| {
                        let gradient = Gradient::linear(vec![
                            Color::new_rgb(255, 0, 0),
                            Color::new_rgb(255, 255, 0),
                            Color::new_rgb(0, 255, 0),
                            Color::new_rgb(0, 255, 255),
                            Color::new_rgb(0, 0, 255),
                            Color::new_rgb(255, 0, 255),
                            Color::new_rgb(255, 0, 0),
                        ], Alignment::Leading, Alignment::Trailing)
                            .color_space(*color_space);

                        Rectangle::new()
                            .fill(gradient)
                            .frame(300.0, 30.0)
                    })
                )
            )).spacing(20.0),
        )).spacing(20.0)
    ));

    application.launch();
}
