use carbide_core::widget::WidgetExt;

fn main() {
    /*let mut pdf = Pdf::new("test_rectangle");

    //pdf.set_widgets(Rectangle::new().fill(EnvironmentColor::Red).frame(100.0, 100.0));

    let complex = HStack::new((
            Rectangle::new().frame(10.0, 170.0),
            VStack::new((
                Rectangle::new().fill(EnvironmentColor::Red).frame(100.0, 100.0),
                Image::new("images/landscape.png")
                    .resizeable()
                    .scaled_to_fill()
                    .clip()
                    .frame(100.0, 100.0),
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
                Circle::new().fill(EnvironmentColor::Red).frame(110.0, 110.0).clip().frame(100.0, 100.0)
            )).background(Rectangle::new().stroke(EnvironmentColor::Yellow).stroke_style(1.0))
        )
    );


    let text = Text::new("Nu tester vi rigtigt hvor meget tekst der kan stå og hvordan den wrapper tekst der er så langt at det ikke kan stå på en linje.")
        .font_size(EnvironmentFontSize::Body)
        .color(EnvironmentColor::Accent);


    pdf.set_widgets(
        VStack::new((
            text,
            complex,
        )).padding(10.0)
            .border()
            .boxed()
    );

    let path = pdf.render();
    println!("Output pdf at: {}", path);

    Command::new("open")
        .arg(path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();*/
}