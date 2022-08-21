use carbide_macro::{CarbideUI, gen_optionals};
use carbide::cursor::MouseCursor;
use carbide::state::{State, TState};
use carbide_controls::Button;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::environment::EnvironmentFontSize;
use carbide_core::state::{LocalState, ReadState, StateExt};
use carbide_core::text::{FontFamily, FontWeight, TextDecoration};
use carbide_core::widget::*;
use carbide_core::{task, Widget};
use carbide_wgpu::{Application, Window};
use carbide_core::draw::Position;
use tokio::time::{sleep, Duration};

fn main() {
    CarbideUI!{
        struct AlbumDetail {
            let articles: Vec<String>
            //let articles: Vec<String> = vec!["Hej".to_string(), "Verden".to_string()]
            let alignment: u32 = 53
            let option: Option<usize> = Some(42)
            let test: String = String::from("Hejsa")

            fn body() -> Widget {
                if $alignment < 100 || $alignment >= 42 {
                    Text("Is [42;100[")
                } else {
                    Text("Is none")
                }

                /*if ($alignment + 10) * 1 == 52 || $alignment == 32 {
                    Text("Is some")
                } else {
                    Text("Is none")
                }*/

                /*if $option.is_some() {
                    Text("Is some")
                } else {
                    Text("Is none")
                }*/

                /*HStack {
                    Text($test)
                    Text($articles[0usize])
                    Text(10)
                    Text("Text")
                    Text($alignment)
                }*/

                /*match $alignment {
                    20 => {
                        Text("It is 20")
                    }
                    x => {
                        Text(x)
                    }
                }*/

                /*match $alignment {
                    53 => {
                        Text("Its default")
                    }
                    42 => {
                        Text("Its nice")
                    }
                    x => {
                        Text("Not as nice")
                    }
                }*/

                /*VStack {
                    for i in $articles {
                        Text(i).font_size(EnvironmentFontSize::LargeTitle)
                    }
                }*/

                /*VStack {
                    for i in vec![1, 2, 3, 42] {
                        Text(i).font_size(EnvironmentFontSize::LargeTitle)
                    }
                }*/

                /*VStack {
                    for (i, j) in vec![(1, 2), (3, 42)] {
                        Text((i + j).ignore_writes()).font_size(EnvironmentFontSize::LargeTitle)
                    }
                }*/

                /*if alignment == 42 {
                    Text("Its nice")
                } else if alignment == 53 {
                    Text("Its default")
                } else {
                    Text("Not as nice")
                }*/

                /*VStack {
                    HStack(spacing: 20.0) {
                        Text("Hejsa").bold()
                        Text("Verden").underline()
                        Text($alignment)
                    }
                    ZStack {
                        Rectangle.fill(EnvironmentColor::Red)
                        Circle.fill(EnvironmentColor::Green)
                            .frame(50.0, 50.0)
                    }.frame(100.0, 100.0)
                }*/

                /*HStack (articles: arts, aaa, optionally: 2 + 23) {

                }
                VStack {
                    VStack { hejsa in

                    }
                    VStack {

                    }
                }
                List(articles) { article in
                    HStack {
                        Image(article.front_image)
                        VStack(alignment: Alignment::Leading) {
                            Text(song.title)
                            Text(song.artist.name)
                                .foregroundStyle(Alignment::Leading)
                                .foregroundStyle(Alignment::Leading)
                        }
                    }
                }*/
            }
        }
    }


    let count = LocalState::new(42);


    let mut application = Application::new();

    let env = application.environment_mut();

    task!(env, count := {
        sleep(Duration::new(2, 0)).await;
        420
    });

    let family =
        FontFamily::new_from_paths("NotoSans", vec![
            "fonts/NotoSans/NotoSans-Regular.ttf",
            "fonts/NotoSans/NotoSans-Bold.ttf",
        ]);
    application.add_font_family(family);

    let child = CarbideUI! {
        AlbumDetail(vec!["Album 1".to_string(), "Album 2".to_string()], alignment: $count)
    };

    application.set_scene(Window::new(
        "Carbide syntax example".to_string(),
        Dimension::new(400.0, 600.0),
        child
    ).close_application_on_window_close());

    application.launch();

}