use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::<String>::path_to_assets("images/rust_press.png");

    let mut window = Window::new("Hello world 2".to_string(), 800, 1200, Some(icon_path), String::from("Hejsa"));

    let mut noto_family = FontFamily::new("NotoSans");
    noto_family.add_font("fonts/NotoSans/NotoSans-Regular.ttf", FontWeight::Normal, FontStyle::Normal);
    noto_family.add_font("fonts/NotoSans/NotoSans-Italic.ttf", FontWeight::Normal, FontStyle::Italic);
    noto_family.add_font("fonts/NotoSans/NotoSans-Bold.ttf", FontWeight::Bold, FontStyle::Normal);
    window.add_font_family(noto_family);

    let mut family = FontFamily::new("Apple Color Emoji");
    family.add_bitmap_font("/System/Library/Fonts/Apple Color Emoji.ttc", FontWeight::Normal, FontStyle::Normal);
    window.add_font_family(family);

    //window.add_font("fonts/NotoSans/NotoSans-Regular.ttf");
    //window.add_font("fonts/NotoSans/NotoSans-Italic.ttf");

    window.set_widgets(
        //Text::new("😀 😀")
        Text::new("# Rich text\nHello *world*, this is /italic/, _underlined_ and -striked-. We can even show 😀, and we support a list of fallback fonts!")
            .foreground_color(EnvironmentColor::Orange)
            /*Text::new("# This is bacon ipsum.
    ## This is bacon ipsum.
    Bacon /ipsum/ dolor *amet* boudin… chicken frankfurter tongue sausage jowl tenderloin biltong ribeye beef filet mignon porchetta. Tenderloin strip steak spare ribs short loin tri-tip. Ball tip sausage buffalo, ham pork loin prosciutto boudin short loin brisket porchetta doner fatback tenderloin pork burgdoggen. Short ribs filet mignon swine, drumstick bacon turkey capicola prosciutto venison short loin doner pork belly ham hock beef pork. Doner shoulder pig andouille ham hock capicola. Sirloin ribeye porchetta tenderloin short ribs.

    Jerky chicken pork loin, landjaeger rump frankfurter kielbasa leberkas chislic beef sausage burgdoggen. Biltong cupim picanha rump hamburger tri-tip. Brisket short loin andouille, alcatra cow pancetta prosciutto rump sausage salami kevin pork belly landjaeger filet mignon ham. Pork chop corned beef bacon, pork ribeye biltong tail cupim leberkas meatloaf prosciutto kevin. Ball tip picanha leberkas, fatback shankle swine tail sirloin. Turducken flank picanha buffalo venison. Jowl pork chop corned beef turducken, tail ground round andouille shankle biltong cow prosciutto kevin picanha short loin chislic.")*/
            .border()
            .border_width(1)
            .color(EnvironmentColor::Green)
            .padding(EdgeInsets::all(40.0))
    );

    window.run_event_loop();
}

