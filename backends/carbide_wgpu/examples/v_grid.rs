use carbide_core::draw::Dimension;
use carbide_core::environment::*;
use carbide_core::state::Map1;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "VGrid - Carbide",
            Dimension::new(600.0, 600.0),
            VGrid::new(ForEach::new(0x1f600..=0x1f64f, |val, idx| unsafe {
                ZStack::new((
                    RoundedRectangle::new(5.0),
                    Text::new(Map1::read_map(val, |c| char::from_u32_unchecked(*c))).font_size(EnvironmentFontSize::LargeTitle),
                )).frame_fixed_height(60.0)
            }), vec![
                VGridColumn::Adaptive(80.0)
            ]).spacing(Dimension::new(1.0, 1.0)).padding(10.0),
        )
    );

    application.launch()
}
