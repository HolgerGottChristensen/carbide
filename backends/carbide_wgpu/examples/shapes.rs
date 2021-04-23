use carbide_wgpu::window::Window;
use futures::executor::block_on;
use carbide_core::window::TWindow;
use carbide_core::state::state::CommonState;
use carbide_core::widget::primitive::v_stack::VStack;
use carbide_core::widget::{Text, Image, Rectangle, HStack, SCALE, Oval};
use carbide_core::widget::complex::SyncTest;
use carbide_core::widget::primitive::widget::WidgetExt;
use carbide_core::widget::primitive::spacer::{Spacer};
use carbide_core::widget::primitive::edge_insets::EdgeInsets;
use carbide_core::widget::primitive::overlaid_layer::OverlaidLayer;
use carbide_core::widget::primitive::scroll::Scroll;
use carbide_core::widget::types::scroll_direction::ScrollDirection;
use carbide_core::widget::types::scale_mode::ScaleMode;
use carbide_core::widget::types::spacer_direction::SpacerDirection;
use carbide_core::widget::primitive::shape::rounded_rectangle::RoundedRectangle;
use carbide_core::widget::primitive::canvas::canvas::Canvas;
use carbide_core::widget::primitive::canvas::context::Context;
use carbide_core::widget::primitive::canvas::context::ContextAction;
use carbide_core::state::environment_font_size::EnvironmentFontSize;
use carbide_core::state::environment_color::EnvironmentColor;
use carbide_core::widget::primitive::shape::capsule::Capsule;
use carbide_core::color::{WHITE, BLUE, RED};
use carbide_core::widget::primitive::canvas::LineCap;
use carbide_core::{Rect, Point};
use std::f64::consts::PI;

fn main() {
    env_logger::init();

    let icon_path = Window::<String>::path_to_assets("images/rust_press.png");

    let mut window = block_on(Window::new("Hello world 2".to_string(), 800, 1200,Some(icon_path), String::from("Hejsa")));

    window.add_font("fonts/NotoSans/NotoSans-Regular.ttf").unwrap();
    //let rust_image = window.add_image("images/rust_press.png").unwrap();

    //let sync_state = CommonState::new_local_with_key(&"Hello".to_string());

    fn draw_star(center: Point, number_of_spikes: u32, outer_radius: f64, inner_radius: f64, mut context: Context) -> Context {
        let mut rotation = PI / 2.0 * 3.0;

        let center_x = center[0];
        let center_y = center[1];

        let mut x = center[0];
        let mut y = center[1];

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

        context
    }

    window.set_widgets(
        VStack::initialize(vec![
            HStack::initialize(vec![
                Rectangle::initialize(vec![])
                    .fill(EnvironmentColor::Accent.into())
                    .frame(100.0.into(), 100.0.into()),
                Rectangle::initialize(vec![])
                    .stroke(EnvironmentColor::Accent.into())
                    .stroke_style(10.0)
                    .frame(100.0.into(), 100.0.into()),
                Rectangle::initialize(vec![])
                    .fill(EnvironmentColor::Accent.into())
                    .stroke(EnvironmentColor::Red.into())
                    .frame(100.0.into(), 100.0.into())
            ]),
            HStack::initialize(vec![
                RoundedRectangle::initialize(25.0)
                    .fill(EnvironmentColor::Accent.into())
                    .frame(100.0.into(), 100.0.into()),
                RoundedRectangle::initialize(25.0)
                    .stroke(EnvironmentColor::Accent.into())
                    .stroke_style(10.0)
                    .frame(100.0.into(), 100.0.into()),
                RoundedRectangle::initialize(25.0)
                    .fill(EnvironmentColor::Accent.into())
                    .stroke(EnvironmentColor::Red.into())
                    .frame(100.0.into(), 100.0.into())
            ]),
            HStack::initialize(vec![
                Oval::new()
                    .fill(EnvironmentColor::Accent.into())
                    .frame(100.0.into(), 100.0.into()),
                Oval::new()
                    .stroke(EnvironmentColor::Accent.into())
                    .stroke_style(10.0)
                    .frame(100.0.into(), 100.0.into()),
                Oval::new()
                    .fill(EnvironmentColor::Accent.into())
                    .stroke(EnvironmentColor::Red.into())
                    .frame(100.0.into(), 100.0.into())
            ]),
            HStack::initialize(vec![
                Capsule::initialize()
                    .fill(EnvironmentColor::Accent.into())
                    .frame(100.0.into(), 50.0.into())
                    .frame(100.0.into(), 100.0.into()),
                Capsule::initialize()
                    .stroke(EnvironmentColor::Accent.into())
                    .stroke_style(10.0)
                    .frame(100.0.into(), 50.0.into())
                    .frame(100.0.into(), 100.0.into()),
                Capsule::initialize()
                    .fill(EnvironmentColor::Accent.into())
                    .stroke(EnvironmentColor::Red.into())
                    .frame(100.0.into(), 50.0.into())
                    .frame(100.0.into(), 100.0.into())
            ]),
            HStack::initialize(vec![
                Canvas::initialize(|rect, mut context| {
                    context = draw_star([50.0, 50.0], 5, 45.0, 20.0, context);
                    context.set_fill_style(BLUE);
                    context.fill();
                    context
                }).frame(100.0.into(), 100.0.into()),
                Canvas::initialize(|rect, mut context| {
                    context = draw_star([50.0, 50.0], 5, 45.0, 20.0, context);
                    context.set_line_width(10.0);
                    context.set_stroke_style(BLUE);
                    context.stroke();
                    context
                }).frame(100.0.into(), 100.0.into()),
                Canvas::initialize(|rect, mut context| {
                    context = draw_star([50.0, 50.0], 5, 45.0, 20.0, context);
                    context.set_fill_style(BLUE);
                    context.set_stroke_style(RED);
                    context.fill();
                    context.stroke();
                    context
                }).frame(100.0.into(), 100.0.into()),
            ]),
        ])
    );

    /*
    OverlaidLayer::new ("overlay_test",
                            VStack::initialize(vec![
                                Text::initialize("Hello world!".into())
                                    .font_size(EnvironmentFontSize::Title.into())
                                    .color(EnvironmentColor::Green.into())
                                    .padding(EdgeInsets::all(10.0)),
                                Text::initialize("Hvad sker der i denne verden og vil den layoute rigtigt når der er en lang tekst og der ikke er nok plads til at det hele kan være på en linje".into())
                                    .padding(EdgeInsets::all(10.0)),
                                Image::new(rust_image),
                                Rectangle::initialize(vec![
                                    SyncTest::new(sync_state)
                                ]).fill(EnvironmentColor::SecondarySystemBackground.into()),
                                HStack::initialize(vec![
                                    RoundedRectangle::initialize(vec![]).frame(100.0.into(), 100.0.into()),
                                    Canvas::initialize(Context { actions: vec![
                                        ContextAction::MoveTo([75.0, 40.0]),
                                        ContextAction::CubicBezierTo{ctrl1: [75.0, 37.0], ctrl2: [70.0, 25.0], to: [50.0, 25.0]},
                                        ContextAction::CubicBezierTo{ctrl1: [20.0, 25.0], ctrl2: [20.0, 62.5], to: [20.0, 62.5]},
                                        ContextAction::CubicBezierTo{ctrl1: [20.0, 80.0], ctrl2: [40.0, 102.0], to: [75.0, 120.0]},
                                        ContextAction::CubicBezierTo{ctrl1: [110.0, 102.0], ctrl2: [130.0, 80.0], to: [130.0, 62.5]},
                                        ContextAction::CubicBezierTo{ctrl1: [130.0, 62.5], ctrl2: [130.0, 25.0], to: [100.0, 25.0]},
                                        ContextAction::CubicBezierTo{ctrl1: [85.0, 25.0], ctrl2: [75.0, 37.0], to: [75.0, 40.0]},
                                        ContextAction::Close
                                    ]}).frame(150.0.into(), 150.0.into()),
                                    Rectangle::initialize(vec![
                                        Scroll::new(
                                            Image::new(rust_image)
                                                .resizeable()
                                                .aspect_ratio(ScaleMode::Fill)
                                                .frame(800.0.into(), 500.0.into())
                                        ).set_scroll_direction(ScrollDirection::Both)
                                            .clip()
                                    ]).fill(EnvironmentColor::SecondarySystemBackground.into()).frame(SCALE.into(), 200.0.into()),
                                ]).padding(EdgeInsets::all(10.0)),
                                HStack::initialize(vec![
                                    Spacer::new(SpacerDirection::Horizontal),
                                    Oval::new()
                                        .padding(EdgeInsets::all(10.0))
                                        .frame(150.0.into(), 150.0.into()),
                                    Spacer::new(SpacerDirection::Horizontal),
                                    Spacer::new(SpacerDirection::Horizontal)
                                ]),
                            ])),
    */

    window.run_event_loop();

}