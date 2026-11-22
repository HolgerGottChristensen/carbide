use carbide::chart::{Axis, Chart, DataColor, DatasetExt, LegendPosition, LineController, ScaleId};
use carbide::color::BLUE;
use carbide::draw::Dimension;
use carbide::widget::VStack;
use carbide::{Application, Window};
use chart_extensions::{Kelvin, LogScale, LollipopController};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    // A week of readings from a sensor that broke down on Thursday, in kelvin: the line is drawn
    // in degrees Celsius, with a gap where the reading is missing, and the forecast as lollipops.
    let readings = vec![(0.0, Kelvin(285.2)), (1.0, Kelvin(287.9)), (2.0, Kelvin(290.1)), (3.0, Kelvin(0.0)), (4.0, Kelvin(288.4)), (5.0, Kelvin(284.6)), (6.0, Kelvin(283.0))];
    let forecast = vec![(0.0, 11.0), (1.0, 14.5), (2.0, 16.0), (3.0, 17.5), (4.0, 15.0), (5.0, 12.0), (6.0, 10.5)];

    let temperatures = Chart::new((
        LineController::new(readings.label("Measured (°C)")),
        LollipopController::new(forecast.label("Forecast (°C)")),
    ))
        .title("Temperature")
        .legend(LegendPosition::Bottom);

    // Populations spanning five orders of magnitude, on a logarithmic y scale.
    let populations = vec![(0.0, 120.0), (1.0, 2_400.0), (2.0, 58_000.0), (3.0, 730_000.0), (4.0, 9_100_000.0)];

    let cities = Chart::new(LollipopController::new(populations.color(DataColor::Color(BLUE)).label("Population")))
        .scale(ScaleId::Y, LogScale::new(Axis::Vertical).title("Inhabitants"))
        .title("Settlements")
        .legend(LegendPosition::Bottom);

    application.set_scene(Window::new(
        "Chart extensions - Carbide",
        Dimension::new(700.0, 800.0),
        VStack::new((temperatures, cities)),
    ));

    application.launch();
}
