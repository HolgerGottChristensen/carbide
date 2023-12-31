use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc::{channel, Receiver, TryRecvError};

use num::{Complex, Zero};
use num::complex::ComplexFloat;
use uuid::Uuid;

use carbide::asynchronous::get_event_sink;
use carbide::CommonWidgetImpl;
use carbide::draw::{Color, Dimension, Position, Rect, Scalar, Texture, TextureFormat};
use carbide::draw::image::ImageId;
use carbide::environment::Environment;
use carbide::event::{CustomEvent, MouseEvent, MouseEventContext, MouseEventHandler, OtherEventContext, OtherEventHandler};
use carbide::image::{DynamicImage, GenericImage, Rgba};
use carbide::mesh::MODE_IMAGE;
use carbide::render::{Render, RenderContext};
use carbide::render::matrix::{Deg, Matrix4, Vector3};
use carbide::widget::*;

const MAX_ITER: u32 = 1000;
//const MAX_ITER: u32 = 20;
const EXPONENT: f64 = 2.0;
const ESCRADIUS: f64 = 2.0;

//const ZOOM: f64 = 7500000000.0;
//const ZOOM: f64 = 1.0;
const ZOOM: f64 = 113388.0;

const CENTER: Position = Position::new(
    -0.7491649396736062,
    0.071803172645556
);

#[derive(Copy, Clone)]
pub struct ImageRenderJobInfo {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    zoom: f64,
    center: Position,
}


#[derive(Clone, Widget)]
#[carbide_exclude(MouseEvent, Render)]
pub struct Mandelbrot {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    jobs: Vec<(ImageRenderJobInfo, Rc<Receiver<(DynamicImage, ImageId)>>)>,

    images: HashMap<(i32, i32, u32), (ImageId, ImageRenderJobInfo)>,
    spawned: bool,

    offset: Position,
    rotation: f64,
}

impl Mandelbrot {
    pub fn new() -> Mandelbrot {
        Mandelbrot {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            jobs: vec![],
            images: HashMap::new(),
            spawned: false,
            offset: Position::origin(),
            rotation: 0.0,
        }
    }
}

impl Render for Mandelbrot {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {

        self.jobs.retain(|(job, receiver)| {
            match receiver.try_recv() {
                Ok((image, id)) => {
                    context.image.update_texture(id.clone(), Texture {
                        width: image.width(),
                        height: image.height(),
                        bytes_per_row: image.width() * 4,
                        format: TextureFormat::RGBA8,
                        data: image.as_bytes(),
                    });
                    true
                }
                Err(TryRecvError::Empty) => {
                    true
                }
                Err(TryRecvError::Disconnected) => {
                    false
                }
            }
        });

        let start_tile_x = ((-self.offset.x() + self.position.x()) / 200.0).floor() as i32;
        let end_tile_x = ((-self.offset.x() + self.position.x() + self.dimension.width) / 200.0).ceil() as i32;

        let start_tile_y = ((-self.offset.y() + self.position.y()) / 200.0).floor() as i32;
        let end_tile_y = ((-self.offset.y() + self.position.y() + self.dimension.height) / 200.0).ceil() as i32;

        for x in start_tile_x..end_tile_x {
            for y in start_tile_y..end_tile_y {
                if self.images.contains_key(&(x, y, 0)) {
                    continue;
                }

                let (sender, receiver) = channel::<(DynamicImage, ImageId)>();

                let width = 200.0;
                let height = 200.0;
                let zoom = ZOOM;
                let xn = x as f64 * 200.0 - 200.0;
                let yn = y as f64 * 200.0 - 200.0;

                let info = ImageRenderJobInfo {
                    x: x * 200,
                    y: y * 200,
                    width: 200,
                    height: 200,
                    zoom,
                    center: Position::new(
                        CENTER.x() + (xn - width / 2.0) / (zoom * width / 2.0),
                        CENTER.y() + (-1.0 * (yn - height / 2.0) / (zoom * height / 2.0))
                    ),
                };

                let id = ImageId::new(PathBuf::from(Uuid::new_v4().to_string()));
                let id2 = id.clone();

                let sink = get_event_sink();

                rayon::spawn(move || {
                    let info_for_job = info.clone();

                    let image = generate_image(info_for_job.width, info_for_job.height, info_for_job.zoom, info_for_job.center);

                    sender.send((image, id2)).unwrap();
                    sink.send(CustomEvent::Async);
                });

                self.jobs.push((info, Rc::new(receiver)));
                self.images.insert((x, y, 0), (id, info));
            }
        }

        let center = self.center_point();

        let transform =
            Matrix4::from_translation(Vector3::new(center.x() as f32, center.y() as f32, 0.0)) *
            Matrix4::from_angle_z(Deg(self.rotation as f32)) *
            Matrix4::from_translation(Vector3::new(self.offset.x() as f32, self.offset.y() as f32, 0.0)) *
            Matrix4::from_translation(Vector3::new(-center.x() as f32, -center.y() as f32, 0.0));

        context.transform(transform, |this| {
            for x in start_tile_x..end_tile_x {
                for y in start_tile_y..end_tile_y {
                    self.images.get(&(x, y, 0)).map(|(id, info)| {
                        if this.image.texture_exist(id) {
                            this.image(
                                id.clone(),
                                Rect::new(
                                    Position::new(info.x as Scalar, info.y as Scalar),
                                    Dimension::new(info.width as Scalar, info.height as Scalar),
                                ),
                                Rect::from_corners(Position::new(0.0, 1.0), Position::new(1.0, 0.0)),
                                MODE_IMAGE,
                            )
                        }
                    });
                }
            }

            /*this.style(DrawStyle::Color(WHITE.alpha(0.2)), |this| {
                this.rect(Rect::new(
                    self.position,
                    self.dimension
                ))
            })*/

        });


    }
}

impl MouseEventHandler for Mandelbrot {
    fn handle_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        match event {
            MouseEvent::Scroll { x, y, .. } => {
                self.offset += Position::new(*x, -*y);
            }
            MouseEvent::Rotation(delta, _, _) => {
                //self.rotation -= delta;
            }
            _ => ()
        }
    }
}

impl CommonWidget for Mandelbrot {
    CommonWidgetImpl!(self, id: self.id, child: (), position: self.position, dimension: self.dimension);
}

impl WidgetExt for Mandelbrot {}

impl Debug for Mandelbrot {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

pub fn generate_image(width: u32, height: u32, zoom: f64, center: Position) -> DynamicImage {
    let mut image = DynamicImage::new_rgba8(width, height);

    for x in 0..image.width() {
        for y in 0..image.height() {
            let c = point_to_complex(image.width() as f64, image.height() as f64, x as f64, y as f64, center.x(), center.y(), zoom);

            let (i, za) = mandelbrot(c);

            //let hue = normalize(i, za) / MAX_ITER as f64;
            //let hue = (i as f64 / MAX_ITER as f64) % 1.0;
            //let color = Color::Hsla(hue as f32, 0.75, 0.5, 0.0);



            //let color = 255 - ((m * 255) as f64 / MAX_ITER as f64) as u8;
            if i == MAX_ITER {
                image.put_pixel(x, y, Rgba([0, 0, 0, 0]));
            } else {
                let color = get_colormap(i, za);
                image.put_pixel(x, y, Rgba(color.to_byte_fsa()));
            }

        }
    }

    image
}

fn normalize(i: u32, za: f64) -> f64 {
    let lzn = za.powf(2.0).ln();
    let nu = (lzn / ESCRADIUS.ln()).ln() / 2.0.ln();
    i as f64 + 1.0 - nu
}

fn get_colormap(i: u32, za: f64) -> Color {
    let ni = normalize(i, za);
    //println!("{}", ni);
    let col1 = C1[ni as usize % C1.len()];
    let col2 = C1[(ni as usize + 1) % C1.len()];

    let col1 = Color::new_rgb(col1[0], col1[1], col1[2]);
    let col2 = Color::new_rgb(col2[0], col2[1], col2[2]);

    Color::hsla_blend(&col1, &col2, ni % 1.0)
}


fn point_to_complex(width: f64, height: f64, x: f64, y: f64, zxoff: f64, zyoff: f64, zoom: f64) -> Complex<f64> {
    let zx_coord = zxoff + ((width / height) * (x - width / 2.0) / (zoom * width / 2.0));
    let zy_coord = zyoff + (-1.0 * (y - height / 2.0) / (zoom * height / 2.0));

    Complex::new(zx_coord, zy_coord)
}

fn mandelbrot(c: Complex<f64>) -> (u32, f64) {
    let mut z = Complex::<f64>::zero();
    let mut last_z = Complex::<f64>::zero();
    let mut n = 0;
    let mut per = 0;

    while z.abs() <= ESCRADIUS.powf(2.0) && n < MAX_ITER {
        z = z.powf(EXPONENT) + c;
        n += 1;

        if z == last_z {
            return (MAX_ITER, z.abs());
        }

        per += 1;
        if per > 20 {
            per = 0;
            last_z = z;
        }
    }

    (n, z.abs())
}

const C1: &[[u8; 3]; 256] = &[
    [248, 132, 247],
    [249, 131, 245],
    [250, 130, 243],
    [251, 129, 240],
    [252, 128, 238],
    [252, 127, 235],
    [252, 125, 232],
    [253, 123, 229],
    [253, 122, 225],
    [252, 120, 222],
    [252, 118, 218],
    [252, 115, 215],
    [251, 113, 211],
    [251, 111, 207],
    [250, 109, 204],
    [250, 107, 200],
    [249, 104, 196],
    [248, 102, 192],
    [248, 100, 188],
    [247, 97, 185],
    [246, 95, 181],
    [245, 93, 177],
    [245, 90, 173],
    [244, 88, 170],
    [243, 85, 166],
    [242, 83, 162],
    [241, 81, 158],
    [240, 78, 155],
    [239, 76, 151],
    [238, 74, 147],
    [236, 71, 143],
    [235, 69, 139],
    [234, 67, 136],
    [233, 65, 132],
    [231, 63, 128],
    [230, 60, 124],
    [228, 58, 120],
    [227, 56, 117],
    [225, 54, 113],
    [223, 52, 109],
    [222, 50, 105],
    [220, 49, 101],
    [218, 47, 97],
    [216, 45, 94],
    [214, 43, 90],
    [212, 41, 86],
    [211, 39, 82],
    [209, 38, 78],
    [207, 36, 75],
    [205, 34, 71],
    [203, 32, 67],
    [201, 31, 63],
    [199, 29, 60],
    [197, 27, 56],
    [195, 26, 53],
    [193, 25, 49],
    [192, 24, 46],
    [190, 23, 42],
    [188, 22, 39],
    [187, 22, 36],
    [185, 22, 33],
    [184, 23, 30],
    [183, 24, 27],
    [182, 25, 24],
    [181, 26, 21],
    [181, 28, 19],
    [181, 30, 17],
    [180, 33, 15],
    [180, 35, 13],
    [180, 38, 11],
    [181, 40, 9],
    [181, 43, 8],
    [182, 45, 7],
    [182, 48, 6],
    [183, 51, 5],
    [184, 53, 5],
    [185, 56, 4],
    [186, 59, 4],
    [187, 61, 4],
    [188, 64, 4],
    [189, 67, 4],
    [190, 69, 4],
    [191, 72, 4],
    [192, 74, 4],
    [193, 77, 4],
    [194, 79, 4],
    [195, 82, 4],
    [196, 84, 4],
    [197, 86, 4],
    [198, 89, 4],
    [199, 91, 4],
    [199, 93, 4],
    [200, 96, 4],
    [201, 98, 4],
    [202, 101, 4],
    [203, 103, 4],
    [204, 105, 4],
    [204, 108, 4],
    [205, 110, 4],
    [206, 112, 4],
    [206, 115, 4],
    [207, 117, 4],
    [208, 120, 4],
    [208, 122, 4],
    [209, 124, 4],
    [209, 127, 4],
    [210, 129, 4],
    [210, 131, 4],
    [211, 134, 4],
    [211, 136, 4],
    [212, 138, 4],
    [212, 141, 4],
    [213, 143, 4],
    [213, 145, 4],
    [213, 147, 4],
    [214, 150, 5],
    [214, 152, 5],
    [214, 154, 6],
    [214, 156, 8],
    [214, 158, 10],
    [214, 160, 12],
    [214, 162, 15],
    [213, 163, 17],
    [213, 165, 21],
    [212, 166, 24],
    [211, 167, 27],
    [210, 168, 31],
    [209, 169, 35],
    [207, 170, 39],
    [205, 170, 42],
    [203, 170, 46],
    [201, 170, 50],
    [199, 170, 54],
    [196, 169, 58],
    [194, 169, 62],
    [191, 168, 66],
    [188, 167, 70],
    [185, 166, 74],
    [182, 165, 77],
    [178, 164, 81],
    [175, 163, 85],
    [172, 162, 88],
    [168, 161, 92],
    [164, 160, 95],
    [161, 159, 98],
    [157, 157, 102],
    [153, 156, 105],
    [149, 155, 108],
    [145, 154, 111],
    [141, 152, 114],
    [137, 151, 117],
    [133, 150, 121],
    [128, 149, 124],
    [124, 147, 127],
    [119, 146, 130],
    [114, 145, 133],
    [109, 144, 135],
    [104, 142, 138],
    [99, 141, 141],
    [94, 140, 144],
    [88, 138, 147],
    [83, 137, 150],
    [77, 135, 153],
    [72, 134, 156],
    [67, 132, 159],
    [61, 130, 162],
    [56, 128, 165],
    [51, 126, 168],
    [47, 124, 171],
    [43, 122, 174],
    [40, 120, 177],
    [37, 118, 180],
    [35, 115, 184],
    [34, 113, 187],
    [33, 110, 190],
    [33, 108, 193],
    [34, 105, 196],
    [35, 102, 199],
    [36, 99, 202],
    [38, 96, 206],
    [39, 93, 209],
    [41, 90, 212],
    [43, 87, 215],
    [44, 84, 218],
    [46, 81, 221],
    [47, 78, 224],
    [49, 75, 226],
    [51, 72, 229],
    [53, 70, 231],
    [55, 67, 234],
    [57, 65, 236],
    [59, 63, 238],
    [62, 62, 240],
    [64, 61, 241],
    [67, 60, 243],
    [70, 60, 244],
    [73, 60, 245],
    [76, 61, 246],
    [79, 61, 246],
    [82, 62, 247],
    [85, 64, 247],
    [88, 65, 248],
    [91, 67, 248],
    [93, 68, 248],
    [96, 70, 249],
    [99, 72, 249],
    [102, 74, 249],
    [104, 76, 249],
    [107, 78, 249],
    [109, 80, 249],
    [112, 82, 249],
    [114, 84, 249],
    [117, 86, 249],
    [119, 88, 249],
    [121, 90, 249],
    [124, 92, 249],
    [126, 94, 249],
    [129, 96, 249],
    [132, 97, 249],
    [134, 99, 249],
    [137, 101, 249],
    [140, 102, 250],
    [143, 104, 250],
    [146, 106, 250],
    [149, 107, 250],
    [153, 108, 250],
    [156, 110, 250],
    [159, 111, 251],
    [163, 112, 251],
    [166, 113, 251],
    [170, 115, 251],
    [173, 116, 252],
    [177, 117, 252],
    [180, 118, 252],
    [184, 119, 253],
    [188, 120, 253],
    [191, 121, 253],
    [195, 122, 253],
    [198, 123, 254],
    [202, 124, 254],
    [205, 125, 254],
    [209, 126, 255],
    [212, 127, 255],
    [216, 128, 255],
    [219, 129, 255],
    [222, 130, 255],
    [225, 130, 255],
    [228, 131, 255],
    [231, 132, 255],
    [234, 132, 255],
    [237, 133, 254],
    [239, 133, 253],
    [241, 133, 253],
    [243, 133, 252],
    [245, 133, 250],
    [247, 133, 249],
];