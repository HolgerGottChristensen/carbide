use chrono::{Days, Local};

use carbide_controls::DatePicker;
use carbide_core::draw::Dimension;
use carbide_core::state::LocalState;
use carbide_core::widget::WidgetExt;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    // let selection = LocalState::new(Some(Local::now().naive_local().date()));
    // let selection = LocalState::new(HashSet::new());
    let selection = LocalState::new(Some(Local::now().naive_local().date()..=Local::now().checked_add_days(Days::new(10)).unwrap().naive_local().date()));

    application.set_scene(Window::new(
        "DatePicker Example - Carbide",
        Dimension::new(400.0, 600.0),
        DatePicker::new(selection)
            .frame(180.0, 40.0),
    ));

    application.launch();
}
