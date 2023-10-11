use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant, SystemTime};
use chrono::Local;
use carbide_core::asynchronous::Timer;
use carbide_core::draw::{Color, Dimension};
use carbide_core::environment::{Environment, EnvironmentFontSize};
use carbide_core::event::ModifierKey;
use carbide_core::state::Map1;
use carbide_core::widget::{HStack, Rectangle, Text, VStack, WidgetExt};
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();


    let mut counter = Arc::new(AtomicUsize::new(0));
    let mut counter2 = counter.clone();

    let mut timer = Timer::new(move || {
        counter.fetch_add(1, Ordering::Relaxed);
    })
        .interval(Duration::from_secs_f64(0.2))
        .repeat()
        .start();

    let timer2 = timer.clone();

    let counter_state = Map1::read_map(0, move |_| {
        counter2.load(Ordering::Relaxed)
    });

    let time_state = Map1::read_map(0, move |_| {
        format!("{}", Local::now().format("%H:%M:%S"))
    });

    application.set_scene(
        Window::new(
            "Timer example",
            Dimension::new(400.0, 600.0),
            *VStack::new(vec![
                Text::new("Click the counter to restart the timer"),
                Text::new(counter_state)
                    .font_size(EnvironmentFontSize::LargeTitle)
                    .frame(50.0, 50.0)
                    .on_click(move |env: &mut Environment, modifier: ModifierKey| {
                        let timer = timer.clone();
                        timer.restart();
                        println!("Clicked");
                    }).boxed(),
                Text::new("Click the clock to toggle the timer"),
                Text::new(time_state)
                    .font_size(EnvironmentFontSize::LargeTitle)
                    .on_click(move |env: &mut Environment, modifier: ModifierKey| {
                        let timer = timer2.clone();
                        if timer.is_running() {
                            timer.stop();
                        } else {
                            timer.start();
                        }
                        println!("Clicked");
                    }).boxed(),
                Text::new("Only the counter is tied directly to the timer. The clock is updated each re-render")
                    .justify_center()
                    .frame_fixed_width(250.0)
                    .fit_height(),
            ])
        ).close_application_on_window_close()
    );

    application.launch()
}