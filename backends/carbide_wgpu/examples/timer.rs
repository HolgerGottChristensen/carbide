use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::Local;

use carbide_core::asynchronous::Timer;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentFontSize;
use carbide_core::state::Map1;
use carbide_core::widget::{Text, VStack, WidgetExt};
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();


    let mut counter = Arc::new(AtomicUsize::new(0));
    let counter2 = counter.clone();

    let timer = Timer::new(move || {
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
            "Timer example - Carbide",
            Dimension::new(400.0, 600.0),
            VStack::new((
                Text::new("Click this to restart the timer")
                    .on_click(move |_| {
                        let timer = timer.clone();
                        timer.restart();
                    }),
                Text::new(counter_state)
                    .font_size(EnvironmentFontSize::LargeTitle),
                Text::new("Click this to toggle the timer")
                    .on_click(move |_| {
                        let timer = timer2.clone();
                        if timer.is_running() {
                            timer.stop();
                        } else {
                            timer.start();
                        }
                    }),
                Text::new(time_state)
                    .font_size(EnvironmentFontSize::LargeTitle),
                Text::new("Only the counter is tied directly to the timer. The clock is updated each re-render.")
                    .justify_center()
                    .frame_fixed_width(250.0)
                    .fit_height(),
            ))
        )
    );

    application.launch()
}