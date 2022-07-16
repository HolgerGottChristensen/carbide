use std::borrow::Borrow;
use std::fs::File;
use std::io::{BufWriter, Read};
use std::path::Path;
use carbide_core::draw::image::{ImageId, ImageMap};
use carbide_core::image::DynamicImage;
use carbide_core::prelude::{ImageInformation, Menu, Primitive, PrimitiveKind, Widget};
use carbide_core::text::{FontFamily, FontId};
use carbide_core::{locate_folder, Ui};
use carbide_core::window::TWindow;
use printpdf::{Color, Image as PdfImage, ImageTransform, Line, Mm, OP_PATH_CONST_LINE_TO, OP_PATH_CONST_MOVE_TO, OP_PATH_PAINT_FILL_NZ, PdfDocument, PdfDocumentReference, PdfLayerIndex, PdfPageIndex, Point, Px, Rgb};
use printpdf::lopdf::content::Operation;
use carbide_core::draw::{Dimension, Position};
use carbide_core::event::NoopEventSink;
use carbide_core::layout::BasicLayouter;
use carbide_core::render::PrimitiveWalker;

pub struct Pdf {
    pub(crate) ui: Ui,
    pub(crate) image_map: ImageMap<DynamicImage>,
    pub title: String,
    pub document: PdfDocumentReference,
}

impl Pdf {
    pub fn new(title: impl Into<String>) -> Self {
        let title = title.into();
        let mut ui = Ui::new(
            Dimension::new(210.0, 297.0),
            0.5,
            None,
            Box::new(NoopEventSink)
        );

        ui.environment.set_root_alignment(BasicLayouter::Top);

        let (document, _, _) =
            PdfDocument::new(&title, Mm(210.0), Mm(297.0), "Layer 1");

        let image_map = ImageMap::new();

        Pdf {
            ui,
            image_map,
            title,
            document
        }
    }

    pub fn render(mut self) -> String {
        let current_layer = self.document.get_page(PdfPageIndex(0)).get_layer(PdfLayerIndex(0));
        let page_dimensions = Dimension::new(210.0, 297.0);

        let mut primitives = self.ui.draw();

        while let Some(primitive) = primitives.next_primitive() {
            let rectangle = primitive.bounding_box;
            match primitive.kind {
                PrimitiveKind::TrianglesSingleColor { color, triangles } => {
                    current_layer.set_fill_color(Color::Rgb(Rgb::new(
                        color.red() as f64,
                        color.green() as f64,
                        color.blue() as f64,
                        None
                    )));

                    for triangle in triangles {
                        let point1 = convert_position_to_point(triangle.points()[0], page_dimensions);
                        let point2 = convert_position_to_point(triangle.points()[1], page_dimensions);
                        let point3 = convert_position_to_point(triangle.points()[2], page_dimensions);
                        current_layer.add_operation(Operation::new(
                            OP_PATH_CONST_MOVE_TO,
                            vec![point1.x.into(), point1.y.into()],
                        ));

                        current_layer.add_operation(Operation::new(
                            OP_PATH_CONST_LINE_TO,
                            vec![point2.x.into(), point2.y.into()],
                        ));
                        current_layer.add_operation(Operation::new(
                            OP_PATH_CONST_LINE_TO,
                            vec![point3.x.into(), point3.y.into()],
                        ));
                        current_layer.add_operation(Operation::new(
                            OP_PATH_CONST_LINE_TO,
                            vec![point1.x.into(), point1.y.into()],
                        ));
                    }

                    current_layer.add_operation(Operation::new(OP_PATH_PAINT_FILL_NZ, vec![]));
                }
                PrimitiveKind::Image { image_id, color, source_rect, mode } => {
                    println!("Image");
                    let image = self.image_map.get(&image_id).unwrap();
                    dbg!(image.color());
                    let pdf_image = PdfImage::from_dynamic_image(image);

                    let width_px = Px(image.width() as usize * 2);
                    let height_px = Px(image.height() as usize * 2);

                    let width = Mm::from(width_px.into_pt(300.0));
                    let height = Mm::from(height_px.into_pt(300.0));

                    let requested_width = rectangle.width();
                    let requested_height = rectangle.height();

                    let scaling_width = requested_width / width.0;
                    let scaling_height = requested_height / height.0;

                    dbg!(width, height);
                    dbg!(requested_width, requested_height);
                    dbg!(scaling_width, scaling_height);

                    let x = rectangle.left() / 2.0;
                    let y = page_dimensions.height - rectangle.top() / 2.0;

                    pdf_image.add_to_layer(current_layer.clone(), ImageTransform {
                        translate_x: Some(Mm(x)),
                        translate_y: Some(Mm(y)),
                        scale_x: Some(scaling_width),
                        scale_y: Some(scaling_height),
                        ..Default::default()
                    })
                }
                PrimitiveKind::Text { color, text } => {
                    /*current_layer.set_fill_color(Color::Rgb(Rgb::new(1.0, 1.0, 0.0, None)));

                    for glyph in &text {
                        if let Some(bb) = glyph.bb() {
                            let point1 = convert_position_to_point(bb.top_left(), page_dimensions);
                            let point2 = convert_position_to_point(bb.top_right(), page_dimensions);
                            let point3 = convert_position_to_point(bb.bottom_right(), page_dimensions);
                            let point4 = convert_position_to_point(bb.bottom_left(), page_dimensions);
                            current_layer.add_operation(Operation::new(
                                OP_PATH_CONST_MOVE_TO,
                                vec![point1.x.into(), point1.y.into()],
                            ));

                            current_layer.add_operation(Operation::new(
                                OP_PATH_CONST_LINE_TO,
                                vec![point2.x.into(), point2.y.into()],
                            ));
                            current_layer.add_operation(Operation::new(
                                OP_PATH_CONST_LINE_TO,
                                vec![point3.x.into(), point3.y.into()],
                            ));
                            current_layer.add_operation(Operation::new(
                                OP_PATH_CONST_LINE_TO,
                                vec![point4.x.into(), point3.y.into()],
                            ));
                            current_layer.add_operation(Operation::new(
                                OP_PATH_CONST_LINE_TO,
                                vec![point1.x.into(), point1.y.into()],
                            ));
                        }
                    }

                    current_layer.add_operation(Operation::new(OP_PATH_PAINT_FILL_NZ, vec![]));*/


                    let inner_font = self.ui.environment.get_font(0);
                    let font_path = inner_font.path();
                    let path = Path::new(&font_path);
                    let mut file = std::fs::File::open(path).unwrap();

                    let font = self.document.add_external_font(&mut file).unwrap();

                    current_layer.set_fill_color(Color::Rgb(Rgb::new(
                        color.red() as f64,
                        color.green() as f64,
                        color.blue() as f64,
                        None
                    )));

                    current_layer.begin_text_section();

                    for glyph in text.iter() {
                        let point_x = glyph.position().x();
                        let point_y = page_dimensions.height - glyph.position().y();
                        dbg!(glyph.font_size());
                        current_layer.set_font(&font, (glyph.font_size()) as f64);
                        current_layer.set_text_cursor(Mm(point_x), Mm(point_y));
                        current_layer.write_text(glyph.character(), &font);
                        current_layer.set_text_cursor(Mm(-point_x), Mm(-point_y));
                    }

                    current_layer.end_text_section();
                }
                _ => ()
            }
        }

        let path = format!("target/{}.pdf", self.title);
        self.document.save(&mut BufWriter::new(File::create(path.clone()).unwrap()))
            .unwrap();

        path
    }
}

fn convert_position_to_point(position: Position, page_dimensions: Dimension) -> Point {
    let x = Mm(position.x() / 2.0);
    let y = Mm((page_dimensions.height - position.y() / 2.0));
    Point::new(x, y)
}


impl TWindow for Pdf {
    fn add_font_family(&mut self, family: FontFamily) -> String {
        let family_name = family.name.clone();
        self.ui.environment.add_font_family(family);
        family_name
    }

    fn add_font<P: AsRef<Path>>(&mut self, path: P) -> FontId {
        let assets = locate_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .unwrap();
        let font_path = assets.join(path.as_ref());

        self.ui.environment.insert_font_from_file(font_path).0
    }

    fn add_image(
        &mut self,
        id: ImageId,
        image: carbide_core::image::DynamicImage,
    ) -> Option<ImageId> {

        let information = ImageInformation {
            width: image.width(),
            height: image.height(),
        };

        let id = self.image_map.insert_with_id(id, image);

        self.ui.environment.insert_image(id, information);

        Some(id)
    }

    fn set_widgets(&mut self, base_widget: Box<dyn Widget>) {
        self.ui.widgets = base_widget;
    }

    fn set_menu(&mut self, menu: Vec<Menu>) {
        unimplemented!()
    }
}