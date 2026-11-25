//! Rendering of a scene with no nodes.

use carbide_3d::Scene3d;
use carbide_core::color::BLUE;
use carbide_core::draw::Dimension;
use carbide_core::widget::WidgetExt;
use carbide_testing::Harness;

/// A `Scene3d` with no nodes, framed by a blue border. Pins down that an empty scene renders
/// without a 3D backend or any nodes, and that its layer is placed and sized as the widget is: the
/// border marks the widget's bounds, and nothing is drawn inside it.
#[test]
fn empty_scene() {
    let scene = Scene3d::new(())
        .border()
        .color(BLUE)
        .padding(20.0);

    let mut harness = Harness::new(scene, Dimension::new(400.0, 300.0));
    harness.snapshot("empty");
}
