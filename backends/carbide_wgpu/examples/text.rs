use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::<String>::path_to_assets("images/rust_press.png");

    let mut window = Window::new("Hello world 2".to_string(), 800, 1200, Some(icon_path), String::from("Hejsa"));

    window.add_font("fonts/NotoSans/NotoSans-Regular.ttf");

    window.set_widgets(
        Text::new("Bacon ipsum dolor amet boudin chicken frankfurter tongue sausage jowl tenderloin biltong ribeye beef filet mignon porchetta. Tenderloin strip steak spare ribs short loin tri-tip. Ball tip sausage buffalo, ham pork loin prosciutto boudin short loin brisket porchetta doner fatback tenderloin pork burgdoggen. Short ribs filet mignon swine, drumstick bacon turkey capicola prosciutto venison short loin doner pork belly ham hock beef pork. Doner shoulder pig andouille ham hock capicola. Sirloin ribeye porchetta tenderloin short ribs.

Jerky chicken pork loin, landjaeger rump frankfurter kielbasa leberkas chislic beef sausage burgdoggen. Biltong cupim picanha rump hamburger tri-tip. Brisket short loin andouille, alcatra cow pancetta prosciutto rump sausage salami kevin pork belly landjaeger filet mignon ham. Pork chop corned beef bacon, pork ribeye biltong tail cupim leberkas meatloaf prosciutto kevin. Ball tip picanha leberkas, fatback shankle swine tail sirloin. Turducken flank picanha buffalo venison. Jowl pork chop corned beef turducken, tail ground round andouille shankle biltong cow prosciutto kevin picanha short loin chislic.

Hamburger salami pig pancetta pork belly sirloin chicken. Beef ribs rump tenderloin buffalo tri-tip. Beef ribs meatball spare ribs chicken andouille, shank shoulder short loin buffalo turkey salami pancetta pig. Landjaeger sausage chuck, ham drumstick alcatra bresaola bacon. Sirloin turducken kevin, chicken short loin pancetta pig. Fatback doner capicola corned beef salami pork loin t-bone shoulder filet mignon hamburger kielbasa pig boudin. Jerky swine ham hock biltong pork salami meatball pork belly pancetta doner ribeye leberkas.

Landjaeger kielbasa meatball pork belly frankfurter bresaola. Doner pork belly venison, drumstick filet mignon pastrami buffalo cow short loin prosciutto turkey landjaeger turducken. Cupim chislic pancetta, leberkas andouille sausage tail doner beef. Cow rump porchetta shank. Jerky spare ribs drumstick leberkas rump, kevin sirloin shoulder pastrami tri-tip.

Biltong chicken bacon, prosciutto tongue jerky pig pork belly alcatra beef tri-tip flank short ribs landjaeger. Sirloin t-bone ham, meatloaf buffalo ground round chicken pancetta short loin. Sirloin beef ribs shankle ribeye short ribs tail. Turducken pig ground round porchetta chuck. Picanha kevin jowl prosciutto corned beef fatback shoulder. Salami andouille kevin kielbasa.

Short ribs ball tip beef, tri-tip drumstick turducken bresaola filet mignon rump tenderloin bacon biltong meatball t-bone. Short ribs sausage tenderloin bresaola pork belly burgdoggen corned beef. Andouille filet mignon tongue ham hock. Brisket shankle doner pork pork chop meatball pancetta venison ball tip chicken. Jowl kevin buffalo turducken flank, swine chislic andouille pig landjaeger cow. Short loin strip steak cow ribeye prosciutto capicola andouille drumstick jerky. Ground round meatloaf sausage alcatra porchetta flank pork belly tail chuck jowl cow hamburger capicola.

Ribeye pastrami ham hock pork belly ground round venison kevin jowl flank biltong ham kielbasa chicken drumstick. Cow burgdoggen ground round t-bone meatloaf kielbasa turducken jowl chislic drumstick sausage. Ham pastrami shank, cupim ham hock tongue strip steak. Drumstick short ribs tail, ground round doner pancetta ball tip tongue shankle. Filet mignon alcatra fatback tenderloin, jowl rump buffalo bacon burgdoggen tri-tip t-bone jerky landjaeger pig. Pig ham beef ribs salami, ribeye kevin pork.

Does your lorem ipsum text long for something a little meatier? Give our generator a try… it’s tasty!")
    );

    window.run_event_loop();
}

