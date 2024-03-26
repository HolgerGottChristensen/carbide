use std::collections::HashMap;
use std::path::Path;
use printpdf::{Color, ImageTransform, IndirectFontRef, Mm, OP_PATH_CONST_LINE_TO, OP_PATH_CONST_MOVE_TO, OP_PATH_PAINT_FILL_NZ, PdfLayerReference, Point, Pt, Px, Rgb};
use printpdf::lopdf::content::Operation;
use carbide_core::draw::{Dimension, InnerImageContext, Position, Rect, Texture, TextureFormat};
use carbide_core::draw::DrawStyle;
use carbide_core::draw::ImageId;
use carbide_core::draw::shape::triangle::Triangle;
use carbide_core::render::{CarbideTransform, InnerRenderContext};
use carbide_core::text::{InnerTextContext, TextId};
use carbide_core::widget::FilterId;
use printpdf::Image as PdfImage;
use carbide_core::image::{DynamicImage, RgbaImage};
use crate::image_context::{IMAGES, PDFImageContext};

pub struct PDFRenderContext {
    /*style_stack: Vec<WGPUStyle>,
    stencil_stack: Vec<Range<u32>>,
    scissor_stack: Vec<BoundingBox>,
    transform_stack: Vec<(Matrix4<f32>, usize)>,

    transforms: Vec<Matrix4<f32>>,
    gradients: Vec<Gradient>,
    vertices: Vec<Vertex>,

    render_pass: Vec<RenderPass>,
    render_pass_inner: Vec<RenderPassCommand>,
    current_bind_group: Option<WGPUBindGroup>,

    state: State,
    window_bounding_box: Rect,
    frame_count: usize,*/
    pub(crate) pdf_layer_reference: PdfLayerReference,
    pub(crate) color_stack: Vec<Color>,
    pub(crate) page_dimensions: Dimension,
    pub(crate) font: IndirectFontRef,
}

impl InnerRenderContext for PDFRenderContext {
    fn transform(&mut self, transform: CarbideTransform) {
        todo!()
    }

    fn pop_transform(&mut self) {
        todo!()
    }

    fn color_filter(&mut self, hue_rotation: f32, saturation_shift: f32, luminance_shift: f32, color_invert: bool) {
        todo!()
    }

    fn pop_color_filter(&mut self) {
        todo!()
    }

    fn clip(&mut self, bounding_box: Rect) {
        println!("Clip");
        let point1 = convert_position_to_point(bounding_box.top_left(), self.page_dimensions);
        let point2 = convert_position_to_point(bounding_box.top_right(), self.page_dimensions);
        let point3 = convert_position_to_point(bounding_box.bottom_right(), self.page_dimensions);
        let point4 = convert_position_to_point(bounding_box.bottom_left(), self.page_dimensions);

        self.pdf_layer_reference.add_operation(Operation::new(
            OP_PATH_CONST_MOVE_TO,
            vec![point1.x.into(), point1.y.into()],
        ));
        self.pdf_layer_reference.add_operation(Operation::new(
            OP_PATH_CONST_LINE_TO,
            vec![point2.x.into(), point2.y.into()],
        ));
        self.pdf_layer_reference.add_operation(Operation::new(
            OP_PATH_CONST_LINE_TO,
            vec![point3.x.into(), point3.y.into()],
        ));
        self.pdf_layer_reference.add_operation(Operation::new(
            OP_PATH_CONST_LINE_TO,
            vec![point4.x.into(), point4.y.into()],
        ));
        self.pdf_layer_reference.add_operation(Operation::new(
            OP_PATH_CONST_LINE_TO,
            vec![point1.x.into(), point1.y.into()],
        ));

        //current_layer.add_operation(Operation::new(OP_PATH_CONST_RECT, vec![100.0.into(), 100.0.into(), 100.0.into(), 100.0.into()]));
        self.pdf_layer_reference.add_operation(Operation::new("q W n", vec![]));
    }

    fn pop_clip(&mut self) {
        println!("Pop clip");
        self.pdf_layer_reference.add_operation(Operation::new("Q", vec![]));
    }

    fn filter(&mut self, id: FilterId, bounding_box: Rect) {
        todo!()
    }

    fn filter2d(&mut self, id1: FilterId, bounding_box1: Rect, id2: FilterId, bounding_box2: Rect) {
        todo!()
    }

    fn stencil(&mut self, geometry: &[Triangle<Position>]) {
        println!("Stencil");
        for triangle in geometry {
            let point1 = convert_position_to_point(triangle.points()[0], self.page_dimensions);
            let point2 = convert_position_to_point(triangle.points()[1], self.page_dimensions);
            let point3 = convert_position_to_point(triangle.points()[2], self.page_dimensions);

            self.pdf_layer_reference.add_operation(Operation::new(
                OP_PATH_CONST_MOVE_TO,
                vec![point1.x.into(), point1.y.into()],
            ));

            self.pdf_layer_reference.add_operation(Operation::new(
                OP_PATH_CONST_LINE_TO,
                vec![point2.x.into(), point2.y.into()],
            ));
            self.pdf_layer_reference.add_operation(Operation::new(
                OP_PATH_CONST_LINE_TO,
                vec![point3.x.into(), point3.y.into()],
            ));
            self.pdf_layer_reference.add_operation(Operation::new(
                OP_PATH_CONST_LINE_TO,
                vec![point1.x.into(), point1.y.into()],
            ));
        }

        self.pdf_layer_reference.add_operation(Operation::new("q W n", vec![]));
    }

    fn pop_stencil(&mut self) {
        println!("DeStencil");
        self.pdf_layer_reference.add_operation(Operation::new("Q", vec![]));
    }

    fn geometry(&mut self, geometry: &[Triangle<Position>]) {
        println!("Draw geometry: {:#?}", geometry);
        for triangle in geometry {
            let point1 = convert_position_to_point(triangle.points()[0], self.page_dimensions);
            let point2 = convert_position_to_point(triangle.points()[1], self.page_dimensions);
            let point3 = convert_position_to_point(triangle.points()[2], self.page_dimensions);

            self.pdf_layer_reference.add_operation(Operation::new(
                OP_PATH_CONST_MOVE_TO,
                vec![point1.x.into(), point1.y.into()],
            ));

            self.pdf_layer_reference.add_operation(Operation::new(
                OP_PATH_CONST_LINE_TO,
                vec![point2.x.into(), point2.y.into()],
            ));
            self.pdf_layer_reference.add_operation(Operation::new(
                OP_PATH_CONST_LINE_TO,
                vec![point3.x.into(), point3.y.into()],
            ));
            self.pdf_layer_reference.add_operation(Operation::new(
                OP_PATH_CONST_LINE_TO,
                vec![point1.x.into(), point1.y.into()],
            ));
        }

        self.pdf_layer_reference.add_operation(Operation::new(OP_PATH_PAINT_FILL_NZ, vec![]));
    }

    fn style(&mut self, style: DrawStyle) {
        println!("Set style to: {:?}", style);
        match style {
            DrawStyle::Color(color) => {
                let color = Color::Rgb(Rgb::new(
                    color.red() as f64,
                    color.green() as f64,
                    color.blue() as f64,
                    None
                ));
                self.color_stack.push(color.clone());

                self.pdf_layer_reference.set_fill_color(color);
            }
            DrawStyle::Gradient(_) => todo!(),
            DrawStyle::MultiGradient(_) => todo!(),
        }
    }

    fn pop_style(&mut self) {
        println!("Pop style");
        self.color_stack.pop();
        if let Some(color) = self.color_stack.last() {
            self.pdf_layer_reference.set_fill_color(color.clone());
        }
    }

    fn image(&mut self, id: ImageId, bounding_box: Rect, source_rect: Rect, mode: u32) {
        IMAGES.with(|images| {
            let images = &*images.borrow();
            let (image, width, height) = images.get(&id).unwrap();

            let width_px = Px(*width as usize * 2);
            let height_px = Px(*height as usize * 2);

            let width = Mm::from(width_px.into_pt(300.0));
            let height = Mm::from(height_px.into_pt(300.0));

            let requested_width = bounding_box.width();
            let requested_height = bounding_box.height();

            let scaling_width = requested_width / width.0;
            let scaling_height = requested_height / height.0;

            /*dbg!(width, height);
            dbg!(requested_width, requested_height);
            dbg!(scaling_width, scaling_height);*/

            let x = bounding_box.left() / 2.0;
            let y = self.page_dimensions.height - bounding_box.top() / 2.0;

            let owned_image = PdfImage { image: image.image.clone() };

            /*dbg!(
                Mm(x), Mm(y), scaling_width, scaling_height
            );*/

            owned_image.add_to_layer(self.pdf_layer_reference.clone(), ImageTransform {
                translate_x: Some(Mm(x)),
                translate_y: Some(Mm(y)),
                scale_x: Some(scaling_width),
                scale_y: Some(scaling_height),
                ..Default::default()
            });
        })
    }

    fn text(&mut self, text: TextId, ctx: &mut dyn InnerTextContext) {
        /*self.pdf_layer_reference.begin_text_section();

        for glyph in text.iter() {
            let point_x = glyph.position().x();
            let point_y = self.page_dimensions.height - glyph.position().y();
            //dbg!(glyph.font_size(), Mm::from(point1.x), Mm::from(point1.y));
            self.pdf_layer_reference.set_font(&self.font, (glyph.font_size()) as f64);
            self.pdf_layer_reference.set_text_cursor(Mm(point_x), Mm(point_y));
            self.pdf_layer_reference.write_text(glyph.character(), &self.font);
            self.pdf_layer_reference.set_text_cursor(Mm(-point_x), Mm(-point_y));
        }

        self.pdf_layer_reference.end_text_section();*/
    }

    fn layer(&mut self, index: u32) {
        todo!()
    }

    fn pop_layer(&mut self) {
        todo!()
    }

    fn filter_new(&mut self) {
        todo!()
    }

    fn filter_new_pop(&mut self, id: FilterId, color: carbide_core::color::Color, post_draw: bool) {
        todo!()
    }

    fn filter_new_pop2d(&mut self, id: FilterId, id2: FilterId, color: carbide_core::color::Color, post_draw: bool) {
        todo!()
    }

    fn mask_start(&mut self) {
        todo!()
    }

    fn mask_in(&mut self) {
        todo!()
    }

    fn mask_end(&mut self) {
        todo!()
    }
}


fn convert_position_to_point(position: Position, page_dimensions: Dimension) -> Point {
    let x = Mm(position.x / 2.0);
    let y = Mm(page_dimensions.height - position.y / 2.0);
    //println!("{:?}, {:?}", x, y);
    Point::new(x, y)
}