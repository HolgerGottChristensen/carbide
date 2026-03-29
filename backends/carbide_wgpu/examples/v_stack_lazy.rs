use carbide_core::draw::Dimension;
use carbide_core::environment::*;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "LazyVStack - Carbide",
            Dimension::new(600.0, 600.0),
            Scroll::new(
                LazyVStack::new(
                    ForEach::new(0..100, |_, idx| {
                        ZStack::new((
                            Rectangle::new()
                                .fill(EnvironmentColor::Accent),
                            Text::new(idx)
                                .font_size(EnvironmentFontSize::Title)
                        )).frame_fixed_height(50.0)
                    })
                ).spacing(10.0)
            ).border().padding(50.0)
        )
    );

    application.launch()
}
