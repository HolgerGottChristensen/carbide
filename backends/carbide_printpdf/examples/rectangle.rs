use std::process::{Command, Stdio};
use carbide_core::environment::{EnvironmentColor};
use carbide_core::widget::{Circle, CornerRadii, HStack, Image, Rectangle, RoundedRectangle, Text, VStack, WidgetExt, ZStack};
use carbide_core::window::TWindow;
use carbide_printpdf::Pdf;

fn main() {
    let mut pdf = Pdf::new("test_rectangle")
        .with_asset_fonts();

    //pdf.set_widgets(Circle::new().fill(EnvironmentColor::Red).frame(110.0, 110.0).clip().frame(100.0, 100.0).padding(10.0));
    //pdf.set_widgets(Rectangle::new().fill(EnvironmentColor::Red).frame(100.0, 100.0));

    let complex = HStack::new(
        vec![
            Rectangle::new().frame(10.0, 170.0),
            VStack::new(vec![
                Rectangle::new().fill(EnvironmentColor::Red).frame(100.0, 100.0),
                Image::new("images/landscape.png").resizeable().scaled_to_fill().clip().frame(100.0, 100.0),
                Circle::new().fill(EnvironmentColor::Green).frame(100.0, 100.0),
                Image::new("images/landscape.png")
                    .resizeable()
                    .scaled_to_fill()
                    .frame(100.0, 100.0)
                    .clip_shape(
                        RoundedRectangle::new(CornerRadii::all(25.0))
                            .stroke(EnvironmentColor::Accent)
                            .stroke_style(10.0),
                    )
                    .frame(100.0, 100.0),
            ]).background(Rectangle::new().stroke(EnvironmentColor::Yellow).stroke_style(1.0))
        ]
    );


    let text = Text::new("Nu tester vi rigtigt hvor meget tekst der kan stå og hvordan den wrapper tekst der er så langt at det ikke kan stå på en linje.")
        .font_size(10)
        //.font_size(EnvironmentFontSize::LargeTitle)
        .color(EnvironmentColor::Accent);


    pdf.set_widgets(
        VStack::new(vec![
            text,
            complex,
        ]).padding(30.0)
    );

    /*pdf.set_widgets(

            .padding(10.0)
    );*/

    let path = pdf.render();
    println!("{}", path);

    Command::new("open")
        .arg(path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
}