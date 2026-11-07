use carbide_core::draw::Dimension;
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
                    ForEach::new(0..1_000_000_000, |_, idx| {
                        ZStack::new((
                            Rectangle::new(),
                            Text::new(idx)
                        )).frame_fixed_height(20.0)
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
