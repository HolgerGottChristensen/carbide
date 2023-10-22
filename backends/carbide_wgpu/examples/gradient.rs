use carbide_core::draw::{Dimension, Position, Color, Alignment};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let colors = vec![
        Color::Rgba(1.0, 0.0, 0.0, 1.0),
        Color::Rgba(1.0, 1.0, 1.0, 1.0),
    ];

    let colors1 = vec![
        Color::Rgba(1.0, 0.0, 0.0, 1.0),
        Color::Rgba(0.0, 1.0, 0.0, 1.0),
        Color::Rgba(0.0, 0.0, 1.0, 1.0),
    ];

    let colors2 = vec![
        Color::Rgba(1.0, 0.0, 0.0, 1.0),
        Color::Rgba(1.0, 0.0, 0.0, 0.0),
    ];

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Gradients example",
        Dimension::new(400.0, 600.0),
        VStack::new(vec![
            HStack::new(vec![
                // We have 4 different kinds of gradients. These are linear, radial, diamond and conic
                // They are all applied as a fill to rectangles. The gradients are all based on alignments.
                // This is the simplest way to have gradients start and end at the corners of the figure.
                Rectangle::new()
                    .fill(Gradient::linear(
                        colors.clone(),
                        Alignment::Leading,
                        Alignment::Trailing,
                    ))
                    .frame(80.0, 80.0)
                    .boxed(),
                Rectangle::new()
                    .fill(Gradient::radial(
                        colors.clone(),
                        Alignment::Center,
                        Alignment::Trailing,
                    ))
                    .frame(80.0, 80.0)
                    .boxed(),
                Rectangle::new()
                    .fill(Gradient::diamond(
                        colors.clone(),
                        Alignment::Center,
                        Alignment::Trailing,
                    ))
                    .frame(80.0, 80.0)
                    .boxed(),
                Rectangle::new()
                    .fill(Gradient::conic(
                        colors.clone(),
                        Alignment::Center,
                        Alignment::Trailing,
                    ))
                    .frame(80.0, 80.0)
                    .boxed(),
            ]),
            HStack::new(vec![
                // We have three different gradient modes, which corresponds to different ways of
                // handling areas outside of the specified gradient, but still in the state.
                // The first is clamp which is also the default. This keeps the latest color at the
                // point of the clamp and uses this outside the area,
                // The second is repeat and repeats start to finish and then starts again.
                // The third is mirror that behaves like repeat other than every other is mirrored.
                // Gradients here use relative positioning, with tuples specifying the start and
                // end of the gradient.
                Rectangle::new()
                    .fill(Gradient::linear(colors.clone(), (0.3, 0.0), (0.7, 0.0)).clamp())
                    .frame(80.0, 80.0)
                    .boxed(),
                Rectangle::new()
                    .fill(Gradient::linear(colors.clone(), (0.3, 0.0), (0.7, 0.0)).repeat())
                    .frame(80.0, 80.0)
                    .boxed(),
                Rectangle::new()
                    .fill(Gradient::linear(colors.clone(), (0.3, 0.0), (0.7, 0.0)).mirror())
                    .frame(80.0, 80.0)
                    .boxed(),
            ]),
            HStack::new(vec![
                // The gradients below show that they can be applied to all other shapes, and not
                // just rectangles. They also show gradients with absolute positioning, useful for
                // creating continuous gradients across multiple shapes. Try to resize the window
                // to see a funky effect of the gradient being absolute even when the shape moves.
                RoundedRectangle::new(10.0)
                    .fill(
                        Gradient::linear(
                            colors.clone(),
                            Position::new(0.0, 0.0),
                            Position::new(10.0, 10.0),
                        )
                            .repeat(),
                    )
                    .frame(80.0, 80.0)
                    .boxed(),
                RoundedRectangle::new(10.0)
                    .fill(
                        Gradient::linear(
                            colors.clone(),
                            Position::new(0.0, 0.0),
                            Position::new(10.0, 10.0),
                        )
                            .repeat(),
                    )
                    .frame(80.0, 80.0)
                    .boxed(),
                Circle::new()
                    .fill(
                        Gradient::radial(
                            colors.clone(),
                            Position::new(0.0, 0.0),
                            Position::new(10.0, 20.0),
                        )
                            .repeat(),
                    )
                    .frame(80.0, 80.0)
                    .boxed(),
                Circle::new()
                    .fill(
                        Gradient::radial(
                            colors.clone(),
                            Position::new(0.0, 0.0),
                            Position::new(10.0, 20.0),
                        )
                            .repeat(),
                    )
                    .frame(80.0, 80.0)
                    .boxed(),
            ]),
            HStack::new(vec![
                // Gradients can have more than two colors, actually up to 16. This limit is imposed for now, but
                // may increase or be customizable in the future.
                Rectangle::new()
                    .fill(Gradient::linear(
                        colors1.clone(),
                        Alignment::Leading,
                        Alignment::Trailing,
                    ))
                    .frame(80.0, 80.0)
                    .boxed(),
                Rectangle::new()
                    .fill(Gradient::radial(
                        colors1.clone(),
                        Alignment::Center,
                        Alignment::Trailing,
                    ))
                    .frame(80.0, 80.0)
                    .boxed(),
                Rectangle::new()
                    .fill(Gradient::diamond(
                        colors1.clone(),
                        Alignment::Center,
                        Alignment::Trailing,
                    ))
                    .frame(80.0, 80.0)
                    .boxed(),
                Rectangle::new()
                    .fill(Gradient::conic(
                        colors1.clone(),
                        Alignment::Center,
                        Alignment::Trailing,
                    ))
                    .frame(80.0, 80.0)
                    .boxed(),
            ]),
            HStack::new(vec![
                // Gradients can not only be used to fill shapes, but also to stroke shapes.
                Rectangle::new()
                    .stroke(Gradient::linear(
                        colors1.clone(),
                        Alignment::Leading,
                        Alignment::Trailing,
                    ))
                    .stroke_style(10.0)
                    .frame(80.0, 80.0)
                    .boxed(),
                Rectangle::new()
                    .stroke(Gradient::radial(
                        colors1.clone(),
                        Alignment::Center,
                        Alignment::BottomTrailing,
                    ))
                    .stroke_style(10.0)
                    .frame(80.0, 80.0)
                    .boxed(),
                Rectangle::new()
                    .stroke(Gradient::diamond(
                        colors1.clone(),
                        Alignment::Center,
                        Alignment::BottomTrailing,
                    ))
                    .stroke_style(10.0)
                    .frame(80.0, 80.0)
                    .boxed(),
                Rectangle::new()
                    .stroke(Gradient::conic(
                        colors1.clone(),
                        Alignment::Center,
                        Alignment::Trailing,
                    ))
                    .stroke_style(10.0)
                    .frame(80.0, 80.0)
                    .boxed(),
            ]),
            HStack::new(vec![
                // Gradients work with transparency as well.
                Rectangle::new()
                    .fill(Gradient::linear(
                        colors2.clone(),
                        Alignment::Leading,
                        Alignment::Trailing,
                    ))
                    .frame(80.0, 80.0)
                    .boxed(),
                Rectangle::new()
                    .fill(Gradient::radial(
                        colors2.clone(),
                        Alignment::Center,
                        Alignment::Trailing,
                    ))
                    .frame(80.0, 80.0)
                    .boxed(),
                Rectangle::new()
                    .fill(Gradient::diamond(
                        colors2.clone(),
                        Alignment::Center,
                        Alignment::Trailing,
                    ))
                    .frame(80.0, 80.0)
                    .boxed(),
                Rectangle::new()
                    .fill(Gradient::conic(
                        colors2.clone(),
                        Alignment::Center,
                        Alignment::Trailing,
                    ))
                    .frame(80.0, 80.0)
                    .boxed(),
            ]),
        ])
    ).close_application_on_window_close());

    application.launch();
}
