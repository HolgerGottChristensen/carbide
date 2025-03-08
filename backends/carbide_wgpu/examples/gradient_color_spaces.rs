use carbide_core::draw::{Alignment, Color, ColorSpace, Dimension};
use carbide_core::draw::gradient::Gradient;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Gradient colorspace example - Carbide",
        Dimension::new(1000.0, 600.0),
        HStack::new((
            VStack::new((
                VStack::new((
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(0, 0, 255),
                            Color::new_rgb(255, 255, 255),
                        ], Alignment::Leading, Alignment::Trailing))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(0, 0, 255),
                            Color::new_rgb(255, 255, 255),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::OkLAB))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(0, 0, 255),
                            Color::new_rgb(255, 255, 255),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::Srgb))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(0, 0, 255),
                            Color::new_rgb(255, 255, 255),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::Xyz))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(0, 0, 255),
                            Color::new_rgb(255, 255, 255),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::Cielab))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(0, 0, 255),
                            Color::new_rgb(255, 255, 255),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::HSL))
                        .frame(300.0, 30.0),
                )),
                VStack::new((
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(0, 0, 0),
                            Color::new_rgb(255, 255, 255),
                        ], Alignment::Leading, Alignment::Trailing))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(0, 0, 0),
                            Color::new_rgb(255, 255, 255),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::OkLAB))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(0, 0, 0),
                            Color::new_rgb(255, 255, 255),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::Srgb))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(0, 0, 0),
                            Color::new_rgb(255, 255, 255),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::Xyz))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(0, 0, 0),
                            Color::new_rgb(255, 255, 255),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::Cielab))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(0, 0, 0),
                            Color::new_rgb(255, 255, 255),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::HSL))
                        .frame(300.0, 30.0),
                )),
            )).spacing(20.0),

            VStack::new((
                VStack::new((
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(255, 0, 0),
                            Color::new_rgb(0, 0, 255),
                        ], Alignment::Leading, Alignment::Trailing))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(255, 0, 0),
                            Color::new_rgb(0, 0, 255),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::OkLAB))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(255, 0, 0),
                            Color::new_rgb(0, 0, 255),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::Srgb))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(255, 0, 0),
                            Color::new_rgb(0, 0, 255),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::Xyz))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(255, 0, 0),
                            Color::new_rgb(0, 0, 255),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::Cielab))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(255, 0, 0),
                            Color::new_rgb(0, 0, 255),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::HSL))
                        .frame(300.0, 30.0),
                )),
                VStack::new((
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(255, 0, 0),
                            Color::new_rgb(0, 255, 0),
                        ], Alignment::Leading, Alignment::Trailing))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(255, 0, 0),
                            Color::new_rgb(0, 255, 0),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::OkLAB))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(255, 0, 0),
                            Color::new_rgb(0, 255, 0),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::Srgb))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(255, 0, 0),
                            Color::new_rgb(0, 255, 0),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::Xyz))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(255, 0, 0),
                            Color::new_rgb(0, 255, 0),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::Cielab))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(255, 0, 0),
                            Color::new_rgb(0, 255, 0),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::HSL))
                        .frame(300.0, 30.0),
                )),
            )).spacing(20.0),

            VStack::new((
                VStack::new((
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(0, 0, 255),
                            Color::new_rgb(255, 255, 0),
                        ], Alignment::Leading, Alignment::Trailing))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(0, 0, 255),
                            Color::new_rgb(255, 255, 0),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::OkLAB))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(0, 0, 255),
                            Color::new_rgb(255, 255, 0),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::Srgb))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(0, 0, 255),
                            Color::new_rgb(255, 255, 0),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::Xyz))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(0, 0, 255),
                            Color::new_rgb(255, 255, 0),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::Cielab))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(0, 0, 255),
                            Color::new_rgb(255, 255, 0),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::HSL))
                        .frame(300.0, 30.0),
                )),
                VStack::new((
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(255, 0, 0),
                            Color::new_rgb(255, 255, 0),
                            Color::new_rgb(0, 255, 0),
                            Color::new_rgb(0, 255, 255),
                            Color::new_rgb(0, 0, 255),
                            Color::new_rgb(255, 0, 255),
                            Color::new_rgb(255, 0, 0),
                        ], Alignment::Leading, Alignment::Trailing))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(255, 0, 0),
                            Color::new_rgb(255, 255, 0),
                            Color::new_rgb(0, 255, 0),
                            Color::new_rgb(0, 255, 255),
                            Color::new_rgb(0, 0, 255),
                            Color::new_rgb(255, 0, 255),
                            Color::new_rgb(255, 0, 0),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::OkLAB))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(255, 0, 0),
                            Color::new_rgb(255, 255, 0),
                            Color::new_rgb(0, 255, 0),
                            Color::new_rgb(0, 255, 255),
                            Color::new_rgb(0, 0, 255),
                            Color::new_rgb(255, 0, 255),
                            Color::new_rgb(255, 0, 0),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::Srgb))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(255, 0, 0),
                            Color::new_rgb(255, 255, 0),
                            Color::new_rgb(0, 255, 0),
                            Color::new_rgb(0, 255, 255),
                            Color::new_rgb(0, 0, 255),
                            Color::new_rgb(255, 0, 255),
                            Color::new_rgb(255, 0, 0),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::Xyz))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(255, 0, 0),
                            Color::new_rgb(255, 255, 0),
                            Color::new_rgb(0, 255, 0),
                            Color::new_rgb(0, 255, 255),
                            Color::new_rgb(0, 0, 255),
                            Color::new_rgb(255, 0, 255),
                            Color::new_rgb(255, 0, 0),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::Cielab))
                        .frame(300.0, 30.0),
                    Rectangle::new()
                        .fill(Gradient::linear(vec![
                            Color::new_rgb(255, 0, 0),
                            Color::new_rgb(255, 255, 0),
                            Color::new_rgb(0, 255, 0),
                            Color::new_rgb(0, 255, 255),
                            Color::new_rgb(0, 0, 255),
                            Color::new_rgb(255, 0, 255),
                            Color::new_rgb(255, 0, 0),
                        ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::HSL))
                        .frame(300.0, 30.0),
                )),
            )).spacing(20.0)
        )).spacing(20.0)
    ));

    application.launch();
}
