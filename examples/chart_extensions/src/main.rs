//! A gallery of charts to play with, one at a time, the built-in ones and one built from this crate's
//! own chart elements: the bar along the top picks the chart, the side
//! bar holds its options, and the bar along the bottom reports what the pointer hovers and clicks.
//!
//! On the charts that zoom: drag or scroll to pan, pinch (or hold Ctrl or Cmd and scroll) to zoom
//! about the pointer, Shift-drag to zoom to a box, and double-click to show everything again. Over
//! an axis, the gestures move only that axis.
//!
//! Every option is a state the chart reads, so it changes the chart as it is set. Each chart also
//! offers a choice of data; as the data's types differ, each choice is a chart of its own.
//!
//! Run it with `cargo run -p chart_extensions`.

mod gallery;

use carbide::draw::Dimension;
use carbide::state::LocalState;
use carbide::{Application, Window};
use gallery::{gallery, Example};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Charts - Carbide",
        Dimension::new(1280.0, 860.0),
        gallery(LocalState::new(Example::Bars)),
    ));

    application.launch();
}

#[cfg(test)]
mod tests {
    use carbide::draw::{Dimension, Position};
    use carbide::state::{LocalState, State};
    use carbide::testing::Harness;
    use super::{gallery, Example};

    /// Every chart in the gallery renders, and can be hovered. A smoke test: it renders each chart
    /// and moves the pointer over it, without comparing pixels.
    #[test]
    fn every_chart_renders_and_hovers() {
        let mut selected = LocalState::new(Example::Bars);
        let mut harness = Harness::new(gallery(selected.clone()), Dimension::new(1280.0, 860.0));

        for (example, at) in [
            (Example::Bars, Position::new(450.0, 300.0)),
            (Example::Lines, Position::new(300.0, 350.0)),
            (Example::Bubbles, Position::new(520.0, 450.0)),
            (Example::Pie, Position::new(600.0, 300.0)),
            (Example::Polar, Position::new(560.0, 300.0)),
            (Example::Scales, Position::new(450.0, 300.0)),
            (Example::Marks, Position::new(450.0, 300.0)),
            (Example::Mixed, Position::new(450.0, 300.0)),
            (Example::Custom, Position::new(450.0, 300.0)),
        ] {
            *selected.value_mut() = example;
            harness.render();
            harness.mouse_move(at);
            harness.render();
        }
    }


}
