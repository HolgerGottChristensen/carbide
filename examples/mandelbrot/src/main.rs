use carbide::draw::{Dimension, Texture, TextureFormat};
use carbide::draw::image::ImageId;
use carbide::state::LocalState;
use carbide::task;
use carbide::widget::{Image, WidgetExt};
use carbide::{Application, Window};
use carbide::state::State;

use crate::mandelbrot::{generate_image, Mandelbrot};

mod mandelbrot;

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    //let image_id = LocalState::new(None);
    //let image_id_for_async = image_id.clone();

    /*task!({
            //let image = generate_image(150, 100);
            let now = Instant::now();
            let image = generate_image(600, 400);
            println!("Generated in: {}s", now.elapsed().as_secs_f64());
            image
        },
        move |image, env: &mut Environment| {
            let id = ImageId::new(PathBuf::new().join("ThisIsNotValid"));

            let texture = Texture {
                width: image.width(),
                height: image.height(),
                bytes_per_row: image.width() * 4,
                format: TextureFormat::RGBA8,
                data: &image.to_rgba8().into_raw(),
            };

            env.image_context.update_texture(id.clone(), texture);
            image_id_for_async.clone().set_value(Some(id));
        }
    );*/

    application.set_scene(
        Window::new(
            "Mandelbrot",
            Dimension::new(800.0, 800.0),
            /*Image::new(image_id)
                .resizeable()
                .frame(600.0, 400.0)*/
            Mandelbrot::new()
                .border()
                .padding(200.0)
        ).close_application_on_window_close()
    );

    application.launch()
}