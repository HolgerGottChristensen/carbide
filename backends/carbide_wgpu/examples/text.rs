use carbide_core::draw::Dimension;
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let text =
        "Bacon ipsum dolor amet boudin… chicken frankfurter tongue sausage jowl tenderloin \
         biltong ribeye beef filet mignon porchetta 😀. Tenderloin strip steak spare ribs \
         short loin tritip. Ball tip sausage buffalo, ham pork loin prosciutto boudin short \
         loin brisket porchetta doner fatback tenderloin pork burgdoggen 👨‍👨‍👧‍👧. Short ribs filet \
         mignon swine, drumstick bacon turkey capicola prosciutto venison short loin doner pork \
         belly ham hock beef pork. Doner shoulder pig andouille ham hock capicola. Sirloin ribeye \
         porchetta tenderloin short ribs.\n\nJerky chicken pork loin, landjaeger rump frankfurter \
         kielbasa leberkas chislic beef sausage burgdoggen. Biltong cupim picanha rump hamburger \
         tritip. Brisket short loin andouille, alcatra cow pancetta prosciutto rump sausage salami \
         kevin pork belly landjaeger filet mignon ham. Pork chop corned beef bacon, pork ribeye \
         biltong tail cupim leberkas meatloaf prosciutto kevin. Ball tip picanha leberkas, fatback \
         shankle swine tail sirloin. Turducken flank picanha buffalo venison. Jowl pork chop corned \
         beef turducken, tail ground round andouille shankle biltong cow prosciutto kevin picanha \
         short loin chislic.";

    //let text = "Hejsa verden!\nBla bla bla👨‍👨‍👧‍👧";
    //let text = "O😀";

    application.set_scene(Window::new(
        "Text example - Carbide",
        Dimension::new(400.0, 600.0),
        Text::new(text)
            .font_size(EnvironmentFontSize::Body)
            .wrap_mode(Wrap::Whitespace)
            .border()
            .border_width(1)
            .color(EnvironmentColor::Green)
            .padding(40.0)
    ));

    application.launch();
}
