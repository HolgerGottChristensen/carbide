use std::process::Command;
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::prelude::WidgetExt;
use carbide_core::text::{FontFamily, FontStyle, FontWeight};
use carbide_core::widget::{Circle, HStack, Image, Rectangle, Text, VStack, ZStack};
use carbide_core::window::TWindow;
use carbide_printpdf::Pdf;

fn main() {
    let mut pdf = Pdf::new("test_rectangle");


    let image_id = pdf.add_image_from_path("images/landscape.png");

    let mut family = FontFamily::new("NotoSans");
    family.add_font_with_hints(
        "fonts/NotoSans/NotoSans-Regular.ttf",
        FontWeight::Normal,
        FontStyle::Normal,
    );
    pdf.add_font_family(family);

    //pdf.set_widgets(Circle::new().fill(EnvironmentColor::Red).frame(100.0, 100.0));
    //pdf.set_widgets(Rectangle::new().fill(EnvironmentColor::Red).frame(100.0, 100.0));
    /*pdf.set_widgets(HStack::new(
        vec![
            Rectangle::new().frame(10.0, 170.0),
            VStack::new(vec![
                Rectangle::new().fill(EnvironmentColor::Red).frame(50.0, 50.0),
                Circle::new().fill(EnvironmentColor::Green).frame(50.0, 50.0),
                Image::new(image_id).resizeable().frame(50.0, 50.0)
            ]).background(Rectangle::new().stroke(EnvironmentColor::Yellow).stroke_style(1.0))
        ]
    ).padding(10.0));*/

    pdf.set_widgets(
        Text::new("Nu tester vi rigtigt hvor meget tekst der kan stå og hvordan den wrapper tekst der er så langt at det ikke kan stå på en linje.")
            .font_size(10)
            //.font_size(EnvironmentFontSize::LargeTitle)
            .color(EnvironmentColor::Accent)
            .padding(10.0)
    );

    let path = pdf.render();

    Command::new("open")
        .arg(path)
        .spawn()
        .unwrap();
}