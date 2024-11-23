use carbide_core::draw::{Alignment, Color, Dimension};
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let text = r#"Bacon ipsum dolor amet boudin… chicken frankfurter tongue sausage jowl tenderloin biltong ribeye beef filet mignon porchetta. Tenderloin strip steak spare ribs short loin tritip. Ball tip sausage buffalo, ham pork loin prosciutto boudin short loin brisket porchetta doner fatback tenderloin pork burgdoggen. Short ribs filet mignon swine, drumstick bacon turkey capicola prosciutto venison short loin doner pork belly ham hock beef pork. Doner shoulder pig andouille ham hock capicola. Sirloin ribeye porchetta tenderloin short ribs.

Jerky chicken pork loin, landjaeger rump frankfurter kielbasa leberkas chislic beef sausage burgdoggen. Biltong cupim picanha rump hamburger tritip. Brisket short loin andouille, alcatra cow pancetta prosciutto rump sausage salami kevin pork belly landjaeger filet mignon ham. Pork chop corned beef bacon, pork ribeye biltong tail cupim leberkas meatloaf prosciutto kevin. Ball tip picanha leberkas, fatback shankle swine tail sirloin. Turducken flank picanha buffalo venison. Jowl pork chop corned beef turducken, tail ground round andouille shankle biltong cow prosciutto kevin picanha short loin chislic."#;

    //let text = "Hejsa verden!\nBla bla bla👨‍👨‍👧‍👧";
    //let text = "O😀";

    let colors1 = vec![
        Color::Rgba(1.0, 0.0, 0.0, 1.0),
        Color::Rgba(0.0, 1.0, 0.0, 1.0),
        Color::Rgba(0.0, 0.0, 1.0, 1.0),
    ];

    application.set_scene(Window::new(
        "Text gradient example - Carbide",
        Dimension::new(400.0, 600.0),
        Text::new(text)
            .color(Gradient::linear(colors1, Alignment::Leading, Alignment::Trailing))
            .font_size(EnvironmentFontSize::Body)
            .wrap(Wrap::Whitespace)
            .border()
            .border_width(1)
            .color(EnvironmentColor::Green)
            .padding(40.0)
    ));

    application.launch();
}
