//! Plays with a chart's pointer input: hovering, clicking, zooming and panning.
//!
//! - Hover a chart to outline the data under the pointer; the line below it reports what was found.
//! - Click the bars to report the click.
//! - On the zoomable charts: drag or scroll to pan, pinch (or hold Ctrl or Cmd and scroll) to zoom
//!   about the pointer, Shift-drag to zoom to a box, and double-click to show everything again.
//!   A trackpad's smart zoom (a two-finger double tap) zooms in, or out again when zoomed.
//!   Over an axis, the gestures move only that axis.
//!
//! - Switch "Interactive" off to make every chart ignore the pointer.
//!
//! Run it with `cargo run -p chart_extensions --bin interaction`.

use carbide::chart::{Axis, BarController, CategoryScale, Chart, DatasetExt, LegendPosition, LinearScale, LineController, Measure, Pick, PickMode, PieController, ScaleId, ScatterController, SizedDataSetExt, ZoomWindow};
use carbide::draw::Dimension;
use carbide::state::{LocalState, ReadStateExt, State};
use carbide::widget::{HStack, Text, VStack, WidgetExt};
use carbide::controls::toggle::Toggle;
use carbide::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    // One switch for every chart's interactivity.
    let interactive = LocalState::new(true);

    application.set_scene(Window::new(
        "Chart interaction - Carbide",
        Dimension::new(1200.0, 940.0),
        VStack::new((
            Toggle::new("Interactive", interactive.clone()),
            HStack::new((
                VStack::new((bars(interactive.clone()), signal(interactive.clone()))),
                VStack::new((bubbles(interactive.clone()), pie(interactive))),
            )),
        )).padding(10.0),
    ));

    application.launch();
}

/// Grouped bars: hovering finds both quarters' bars at the category under the pointer, and a click
/// reports what was clicked.
fn bars(interactive: LocalState<bool>) -> impl carbide::widget::Widget {
    let hovered = LocalState::new(None::<Pick>);
    let clicked = LocalState::new("Click a bar".to_string());
    let on_click = clicked.clone();

    let revenue = vec![12.0, 19.0, 3.0, 5.0, 2.0, 8.0].label("Revenue");
    let costs = vec![8.0, 11.0, 6.0, 4.0, 3.0, 7.0].label("Costs");

    let chart = Chart::new(BarController::new((revenue, costs)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).labels(["Jan", "Feb", "Mar", "Apr", "May", "Jun"]))
        .title("Hover the months, click the bars")
        .legend(LegendPosition::Bottom)
        .interactive(interactive.clone())
        .hover(hovered.clone())
        .on_click(move |click| {
            let mut clicked = on_click.clone();
            *clicked.value_mut() = format!("Clicked: {}", describe(click.pick));
        })
        .frame(580.0, 380.0);

    VStack::new((
        chart,
        Text::new(hovered.map(|pick: &Option<Pick>| pick.as_ref().map_or("Hover the chart".to_string(), |pick| format!("Hovering: {}", describe(pick))))),
        Text::new(clicked),
    ))
}

/// A long line, zoomable along x, its y scale fitting to the part shown.
fn signal(interactive: LocalState<bool>) -> impl carbide::widget::Widget {
    let hovered = LocalState::new(None::<Pick>);
    let zoom = LocalState::new(ZoomWindow::WHOLE);

    let points: Vec<(f64, f64)> = (0..400).map(|i| {
        let x = i as f64;
        (x, 50.0 + 30.0 * (x / 9.0).sin() + 15.0 * (x / 37.0).cos() + 8.0 * (x / 3.0).sin())
    }).collect();

    let chart = Chart::new(LineController::new(points.label("Signal")))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(zoom.clone()))
        .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).fit_visible(true))
        .title("Pinch to zoom, drag or scroll to pan")
        .interactive(interactive.clone())
        .hover(hovered.clone())
        .hover_mode(PickMode::Nearest(Measure::X))
        .frame(580.0, 380.0);

    VStack::new((
        chart,
        Text::new(zoom.map(|window: &ZoomWindow| format!("x shows {:.0}% to {:.0}% of its range", window.start * 100.0, window.end * 100.0))),
        Text::new(hovered.map(|pick: &Option<Pick>| pick.as_ref().map_or("Hover the line".to_string(), describe))),
    ))
}

/// Bubbles, zoomable along both scales, hovered by the nearest bubble.
fn bubbles(interactive: LocalState<bool>) -> impl carbide::widget::Widget {
    let hovered = LocalState::new(None::<Pick>);
    let (x, y) = (LocalState::new(ZoomWindow::WHOLE), LocalState::new(ZoomWindow::WHOLE));

    let first: Vec<(f64, f64, f64)> = (0..25).map(|i| {
        let i = i as f64;
        ((i * 7.3) % 20.0, (i * 3.7) % 15.0, 5.0 + (i * 11.0) % 40.0)
    }).collect();
    let second: Vec<(f64, f64, f64)> = (0..15).map(|i| {
        let i = i as f64;
        ((i * 5.1 + 2.0) % 20.0, (i * 6.3 + 4.0) % 15.0, 10.0 + (i * 13.0) % 30.0)
    }).collect();

    let chart = Chart::new(ScatterController::new((first.bubbles().label("North"), second.bubbles().label("South"))))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(x.clone()))
        .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).zoom(y.clone()))
        .title("Zoom and pan both axes, or one over its axis")
        .legend(LegendPosition::Bottom)
        .interactive(interactive.clone())
        .hover(hovered.clone())
        .hover_mode(PickMode::Nearest(Measure::Both))
        .frame(580.0, 380.0);

    VStack::new((
        chart,
        Text::new(x.map(|window: &ZoomWindow| format!("x shows {:.0}% to {:.0}%", window.start * 100.0, window.end * 100.0))),
        Text::new(hovered.map(|pick: &Option<Pick>| pick.as_ref().map_or("Hover the bubbles".to_string(), describe))),
    ))
}

/// Two rings of a doughnut, hovered by the slice under the pointer.
fn pie(interactive: LocalState<bool>) -> impl carbide::widget::Widget {
    let hovered = LocalState::new(None::<Pick>);

    let rings = vec![
        vec![("Rent", 45.0), ("Food", 25.0), ("Travel", 15.0), ("Other", 15.0)],
        vec![("Rent", 40.0), ("Food", 30.0), ("Travel", 10.0), ("Other", 20.0)],
    ];

    let chart = Chart::new(PieController::new(rings))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).inner_radius(0.4))
        .title("Hover the slices")
        .interactive(interactive.clone())
        .hover(hovered.clone())
        .hover_mode(PickMode::Point)
        .frame(580.0, 380.0);

    VStack::new((
        chart,
        Text::new(hovered.map(|pick: &Option<Pick>| pick.as_ref().map_or("Hover a slice".to_string(), describe))),
    ))
}

/// What a pick found, for the lines under the charts: each datum by its dataset and index, and the
/// value under the pointer on each scale, or its category.
fn describe(pick: &Pick) -> String {
    let data = if pick.data.is_empty() {
        "nothing".to_string()
    } else {
        pick.data.iter().map(|datum| format!("dataset {} #{}", datum.dataset, datum.index)).collect::<Vec<_>>().join(", ")
    };

    let values = pick.values.iter()
        .map(|(id, value)| match &value.category {
            Some(category) => format!("{} {}", id.name(), category.name.clone().unwrap_or_else(|| format!("{}", category.index))),
            None => format!("{} {:.1}", id.name(), value.value),
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!("{data} at {values}")
}
