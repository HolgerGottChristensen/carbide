use copypasta::{ClipboardContext, ClipboardProvider};
use carbide_core::asynchronous::Timer;
use carbide_core::draw::{Alignment, Dimension};
use carbide_core::environment::EnvironmentColor::Green;
use carbide_core::state::{GlobalState, State};
use carbide_core::widget::{CrossAxisAlignment, ForEach, GridItem, Image, LazyVGrid, Rectangle, Scroll, Text, VStack, WidgetExt, ZStack};
use carbide_icons::all_icon_names;
use carbide_wgpu::{Application, Window};

pub fn build_application(name: &str) -> Application {
    let mut application = Application::new();

    let toast_offset = GlobalState::new(100.0);
    let toast_offset_for_content = toast_offset.clone();
    let mut toast_offset_for_timer = toast_offset.clone();

    let timer = Timer::new(move || {
        *toast_offset_for_timer.value_mut() = 60.0;
    });

    let content = ForEach::new(all_icon_names(), move |name: &String, _| ZStack::new(
        (
            Rectangle::new(),
            VStack::new((
                Image::system(name.clone()),
                Text::new(name.clone())
            )).cross_axis_alignment(CrossAxisAlignment::Center)
                .spacing(3.0)
                .padding(10.0)
        ))
        .on_click({
            let name2 = name.clone();
            let timer2 = timer.clone();
            let toast_offset_for_content2 = toast_offset_for_content.clone();

            move |_ctx| {
                let mut toast_offset_for_content3 = toast_offset_for_content2.clone();

                let mut clipboard = ClipboardContext::new().unwrap();

                clipboard.set_contents(name2.clone()).expect("Could not set contents");

                *toast_offset_for_content3.value_mut() = 0.0;
                timer2.clone().restart();
            }
        })
        .frame_fixed_height(70.0));

    let scrollview = Scroll::new(
        LazyVGrid::new(vec![
            GridItem::Adaptive(250.0),
        ], content).spacing(Dimension::new(5.0, 5.0))
    )
        .clip()
        .padding(10.0);



    let stack = ZStack::new((
        scrollview,
        Rectangle::new().accent_color(Green).frame(140.0, 40.0).padding(5.0).offset(0.0, toast_offset)
    )).with_alignment(Alignment::Bottom);

    application.set_scene(Window::new(
        format!("{} example - Carbide", name),
        Dimension::new(800.0, 600.0),
        stack,
    ));

    application
}