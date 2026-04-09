use carbide_core::draw::Dimension;
use carbide_core::environment::*;
use carbide_core::state::Map1;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "HGrid - Carbide",
            Dimension::new(600.0, 600.0),
            Scroll::new(
                LazyHGrid::new(vec![
                    GridItem::Adaptive(80.0)
                ], ForEach::new(0x1f600..=0x1f64f, |val, idx| unsafe {
                    ZStack::new((
                        RoundedRectangle::new(5.0),
                        Text::new(Map1::read_map(val, |c| char::from_u32_unchecked(*c))).font_size(EnvironmentFontSize::LargeTitle),
                    )).frame_fixed_width(60.0)
                })).spacing(Dimension::new(1.0, 1.0))
            )
                .clip()
                .border()
                .padding(50.0),
        )
    );

    application.launch()
}
