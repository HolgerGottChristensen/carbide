use std::collections::HashSet;
use chrono::{Days, Local, Month, NaiveDate};
use carbide_controls::{Calendar, CheckBoxValue, PlainButton, PlainCalendar, PlainCheckBox};
use carbide_core::a;
use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, StateExtNew};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

use carbide_core as carbide;
use carbide_core::environment::EnvironmentColor;

fn main() {

    let mut application = Application::new()
        .with_asset_fonts();

    //let selection = LocalState::new(Some(Local::now().naive_local().date()));
    //let selection = LocalState::new(HashSet::new());
    let selection = LocalState::new(Some(Local::now().naive_local().date()..=Local::now().checked_add_days(Days::new(10)).unwrap().naive_local().date()));

    application.set_scene(Window::new(
        "Calendar Example - Carbide",
        Dimension::new(400.0, 600.0),
        Calendar::new(selection),
    ).close_application_on_window_close());

    application.launch();
}
