use carbide_core::draw::Dimension;
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::state::{AnyState, Map1, ReadStateExtNew, State};
use carbide_core::widget::*;
use carbide_icons::all_icon_names;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(Window::new(
        "Image SVG example - Carbide",
        Dimension::new(800.0, 600.0),
        //Image::new("icons/ambulance.svg").color(EnvironmentColor::Label)
        //Image::system("chart.network")
        Scroll::new(VGrid::new(ForEach::new(all_icon_names(), |name: Box<dyn AnyState<T=String>>, _| ZStack::new((
            Rectangle::new(),
            VStack::new((
                Image::system(name.clone()),
                Text::new(name)
            )).cross_axis_alignment(CrossAxisAlignment::Center)
                .spacing(3.0)
                .padding(10.0)
        ))), vec![
            VGridColumn::Adaptive(130.0)
        ]).spacing(Dimension::new(5.0, 5.0))).padding(10.0)
        //Image::new("/Users/holgerchristensen/Repositories/resvg/crates/resvg/tests/tests/shapes/circle/simple-case.svg").color(EnvironmentColor::Label)
        //Image::new("/Users/holgerchristensen/Repositories/resvg/crates/resvg/tests/tests/shapes/ellipse/simple-case.svg").color(EnvironmentColor::Label)
        //Image::new("/Users/holgerchristensen/Repositories/resvg/crates/resvg/tests/tests/shapes/line/simple-case.svg").color(EnvironmentColor::Label)
        //Image::new("/Users/holgerchristensen/Repositories/resvg/crates/resvg/tests/tests/shapes/path/M-L.svg").color(EnvironmentColor::Label)
        //Image::new("/Users/holgerchristensen/Repositories/resvg/crates/resvg/tests/tests/shapes/polygon/simple-case.svg").color(EnvironmentColor::Label)
        //Image::new("/Users/holgerchristensen/Repositories/resvg/crates/resvg/tests/tests/shapes/polyline/simple-case.svg").color(EnvironmentColor::Label)
        //Image::new("/Users/holgerchristensen/Repositories/resvg/crates/resvg/tests/tests/shapes/rect/simple-case.svg").color(EnvironmentColor::Label)
            .border(),
    ));

    application.launch();
}
