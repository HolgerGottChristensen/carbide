use carbide_core::environment::EnvironmentColor;
use carbide_core::text::{FontFamily, FontStyle, FontWeight};
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new("Pretty text example".to_string(), 400, 600, Some(icon_path));

    let mut family = FontFamily::new("NotoSans");
    family.add_font_with_hints(
        "fonts/NotoSans/NotoSans-Regular.ttf",
        FontWeight::Normal,
        FontStyle::Normal,
    );
    family.add_font_with_hints(
        "fonts/NotoSans/NotoSans-Italic.ttf",
        FontWeight::Normal,
        FontStyle::Italic,
    );
    family.add_font_with_hints(
        "fonts/NotoSans/NotoSans-Bold.ttf",
        FontWeight::Bold,
        FontStyle::Normal,
    );
    window.add_font_family(family);

    //window.add_font("fonts/NotoSans/NotoSans-Regular.ttf");
    //window.add_font("fonts/NotoSans/NotoSans-Italic.ttf");

    window.set_widgets(Text::new("Bacon ipsum dolor amet boudin… chicken frankfurter tongue sausage jowl tenderloin biltong ribeye beef filet mignon porchetta. Tenderloin strip steak spare ribs short loin tritip. Ball tip sausage buffalo, ham pork loin prosciutto boudin short loin brisket porchetta doner fatback tenderloin pork burgdoggen. Short ribs filet mignon swine, drumstick bacon turkey capicola prosciutto venison short loin doner pork belly ham hock beef pork. Doner shoulder pig andouille ham hock capicola. Sirloin ribeye porchetta tenderloin short ribs.

        Jerky chicken pork loin, landjaeger rump frankfurter kielbasa leberkas chislic beef sausage burgdoggen. Biltong cupim picanha rump hamburger tritip. Brisket short loin andouille, alcatra cow pancetta prosciutto rump sausage salami kevin pork belly landjaeger filet mignon ham. Pork chop corned beef bacon, pork ribeye biltong tail cupim leberkas meatloaf prosciutto kevin. Ball tip picanha leberkas, fatback shankle swine tail sirloin. Turducken flank picanha buffalo venison. Jowl pork chop corned beef turducken, tail ground round andouille shankle biltong cow prosciutto kevin picanha short loin chislic.")
        .border()
        .border_width(1)
        .color(EnvironmentColor::Green)
        .padding(EdgeInsets::all(40.0))
    );

    window.launch();
}
