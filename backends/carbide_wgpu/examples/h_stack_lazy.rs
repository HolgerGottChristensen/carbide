use carbide_core::draw::{Angle, Dimension};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "LazyHStack - Carbide",
            Dimension::new(600.0, 600.0),
            Scroll::new(
                LazyHStack::new(
                    ForEach::new(0..1_000_000_000, |_, idx| {
                        ZStack::new((
                            Rectangle::new(),
                            Text::new(idx)
                                .rotation_effect(-90.0)
                                .frame_fixed_width(20.0)
                        )).frame_fixed_width(20.0)
                    })
                ).spacing(3.0)
            )
                .clip()
                .border()
                .padding(50.0)
        )
    );

    application.launch()
}
