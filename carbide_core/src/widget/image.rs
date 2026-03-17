//! A simple, non-interactive widget for drawing an `Image`.

use std::fs::File;
use std::io::{BufReader, Read};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use accesskit::{Node, Point, Role, Size};
use cgmath::{Matrix4, Vector3};
use url::Url;
use carbide::draw::{DrawOptions, DrawShape, DrawStyle};
use carbide::draw::fill::FillOptions;
use carbide::draw::stroke::{LineCap, LineJoin};
use carbide::render::RenderInstruction;
use crate::accessibility::AccessibilityContext;
use crate::scene::SceneManager;
use carbide_macro::carbide_default_builder2;
use carbide_usvg::{Document, Options, Paint, PaintOrder, Tree};
use carbide_usvg::tiny_skia_path::PathSegment;
use crate::accessibility::Accessibility;
use crate::color::{BLUE, RED};
use crate::CommonWidgetImpl;
use crate::draw::{Dimension, ImageId, ImageIdFormat, ImageMetrics, ImageMode, ImageOptions, Position, Rect, Scalar, Texture, TextureFormat};
use crate::draw::path::PathInstruction;
use crate::draw::pre_multiply::PreMultiply;
use crate::draw::stroke::StrokeOptions;
use crate::environment::EnvironmentColor;
use crate::identifiable::Identifiable;
use crate::layout::{Layout, LayoutContext};
use crate::render::{Render, Style, RenderContext};
use crate::state::{IntoReadState, ReadState, ReadStateExtNew};
use crate::widget::{Widget, WidgetId, CommonWidget, WidgetSync, CornerRadii};
use crate::widget::types::ScaleMode;

/// A primitive and basic widget for drawing an `Image`.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout, Accessibility)]
pub struct Image<Id, C> where Id: ReadState<T=ImageId>, C: ReadState<T=Style> {
    #[id] id: WidgetId,
    /// The unique identifier for the image that will be drawn.
    #[state] image_id: Id,

    mode: ImageMode,
    position: Position,
    dimension: Dimension,
    scale_mode: ScaleMode,
    resizeable: bool,
    decorative: bool,

    color: Option<C>,

    /// The rectangle area of the original source image that should be used.
    src_rect: Option<Rect>,
}

impl Image<ImageId, Style> {
    pub fn new<Id: IntoReadState<ImageId>>(id: Id) -> Image<Id::Output, Style> {
        Image {
            id: WidgetId::new(),
            image_id: id.into_read_state(),
            color: None,
            mode: ImageMode::Image,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            scale_mode: ScaleMode::Fit,
            resizeable: false,
            decorative: false,
            src_rect: None,
        }
    }

    pub fn new_icon<Id: IntoReadState<ImageId>>(id: Id) -> Image<Id::Output, impl ReadState<T=Style>> {
        Image {
            id: WidgetId::new(),
            image_id: id.into_read_state(),
            src_rect: None,
            color: Some(EnvironmentColor::Label.style()),
            mode: ImageMode::Icon,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            scale_mode: ScaleMode::Fit,
            resizeable: false,
            decorative: false
        }
    }
}

impl<Id: ReadState<T=ImageId>, C: ReadState<T=Style>> Image<Id, C> {
    /// Set the source rectangle of the image to use. The rect is given in image pixel coordinates.
    /// A source rect outside the size of the image will result in a larger image, but where the
    /// bottom right is blank.
    pub fn source_rectangle(mut self, rect: Rect) -> Self {
        self.src_rect  = Some(rect);
        self
    }

    pub fn decorative(mut self) -> Self {
        self.decorative = true;
        self
    }

    pub fn resizeable(mut self) -> Self {
        self.resizeable = true;
        self
    }

    pub fn scaled_to_fit(mut self) -> Self {
        self.resizeable = true;
        self.scale_mode = ScaleMode::Fit;
        self
    }

    pub fn scaled_to_fill(mut self) -> Self {
        self.resizeable = true;
        self.scale_mode = ScaleMode::Fill;
        self
    }

    pub fn aspect_ratio(mut self, mode: ScaleMode) -> Self {
        self.scale_mode = mode;
        self
    }

    pub fn color<C2: IntoReadState<Style>>(self, color: C2) -> Image<Id, C2::Output> {
        Image {
            id: self.id,
            image_id: self.image_id,
            src_rect: self.src_rect,
            color: Some(color.into_read_state()),
            mode: self.mode,
            position: self.position,
            dimension: self.dimension,
            scale_mode: self.scale_mode,
            resizeable: self.resizeable,
            decorative: self.decorative,
        }
    }
}

impl<Id: ReadState<T=ImageId>, C: ReadState<T=Style>> Layout for Image<Id, C> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let image_id = &*self.image_id.value();

        // If we have a raster image, but we have not yet loaded it into the context
        match (image_id, image_id.format()) {
            (ImageId::Local(path, ..), ImageIdFormat::Raster) => {
                if !ctx.image.exist(image_id, ctx.env) {
                    Self::load_local_raster(ctx, path);
                }
            }
            (ImageId::Remote(url, ..), ImageIdFormat::Raster) => {
                if !ctx.image.exist(image_id, ctx.env) {
                    Self::load_remote_raster(ctx, url);
                }
            }
            (ImageId::Local(path, ..), ImageIdFormat::Vector) => {
                if !ctx.image.exist(image_id, ctx.env) {
                    let full_path = if path.is_relative() {
                        let assets = crate::locate_folder::Search::KidsThenParents(3, 5)
                            .for_folder("assets")
                            .unwrap();

                        assets.join(path.as_ref())
                    } else {
                        path.to_path_buf()
                    };


                    let mut file = BufReader::new(File::open(full_path).unwrap());

                    let mut string = String::new();
                    file.read_to_string(&mut string);

                    let options = Options::default();

                    let doc = Document::from_str(&string, &options);

                    let tree = doc.to_tree(&options);

                    println!("{:?}", doc);
                    println!("{:#?}", tree);

                    let mut description = vec![];

                    for child in tree.root().children() {
                        match child {
                            carbide_usvg::Node::Group(_) => {}
                            carbide_usvg::Node::Path(p) => {
                                let mut path_instructions = vec![];

                                for segment in p.data().segments() {
                                    match segment {
                                        PathSegment::MoveTo(point) => path_instructions.push(PathInstruction::MoveTo { to: Position::new(point.x as f64 * 2.0, point.y as f64 * 2.0) }),
                                        PathSegment::LineTo(point) => path_instructions.push(PathInstruction::LineTo { to: Position::new(point.x as f64 * 2.0, point.y as f64 * 2.0) }),
                                        PathSegment::QuadTo(ctrl, point) => path_instructions.push(PathInstruction::QuadraticBezierTo { ctrl: Position::new(ctrl.x as f64 * 2.0, ctrl.y as f64 * 2.0), to: Position::new(point.x as f64 * 2.0, point.y as f64 * 2.0) }),
                                        PathSegment::CubicTo(ctrl1, ctrl2, point) => path_instructions.push(PathInstruction::CubicBezierTo { ctrl1: Position::new(ctrl1.x as f64 * 2.0, ctrl1.y as f64 * 2.0), ctrl2: Position::new(ctrl2.x as f64 * 2.0, ctrl2.y as f64 * 2.0), to: Position::new(point.x as f64 * 2.0, point.y as f64 * 2.0) }),
                                        PathSegment::Close => path_instructions.push(PathInstruction::Close),
                                    }
                                }

                                if let Some(s) = p.stroke() {
                                    let style = match s.paint() {
                                        Paint::CurrentColor => self.color.as_ref().map(|a| a.as_dyn_read()).unwrap_or(Style::Color(BLUE).as_dyn_read()),
                                        Paint::Color(_) => Style::Color(BLUE).as_dyn_read(),
                                        Paint::LinearGradient(_) => Style::Color(BLUE).as_dyn_read(),
                                        Paint::RadialGradient(_) => Style::Color(BLUE).as_dyn_read(),
                                        Paint::Pattern(_) => Style::Color(BLUE).as_dyn_read()
                                    };

                                    description.push(RenderInstruction::PushStyle { style });

                                    description.push(RenderInstruction::Shape {
                                        shape: DrawShape::Path(crate::draw::path::Path { instructions: path_instructions }),
                                        options: DrawOptions::Stroke(StrokeOptions::default().with_stroke_cap(LineCap::Round).with_stroke_join(LineJoin::Round).with_stroke_width(4.0))
                                    });

                                    description.push(RenderInstruction::PopStyle);
                                }
                            }
                            carbide_usvg::Node::Image(_) => {}
                            carbide_usvg::Node::Text(_) => {}
                        }
                    }

                    ctx.image.update_vector(
                        &ImageId::Local(path.clone(), ImageIdFormat::Vector),
                        description,
                        Dimension::new(tree.size().width() as Scalar, tree.size().height() as Scalar),
                        ctx.env
                    );
                }

            }
            _ => {}
        }


        let image_information = if let Some(source_rect) = self.src_rect {
            source_rect.dimension
        } else {
            match ctx.image.metrics(image_id, ctx.env) {
                ImageMetrics::Unknown => Dimension::new(100.0, 100.0),
                ImageMetrics::Raster { width, height } => Dimension::new(width as Scalar, height as Scalar),
                ImageMetrics::Vector { dimension } => dimension
            }
        };

        if !self.resizeable {
            self.dimension = Dimension::new(image_information.width, image_information.height);
        } else {
            let width_factor = requested_size.width / image_information.width;
            let height_factor = requested_size.height / image_information.height;

            match self.scale_mode {
                ScaleMode::Fit => {
                    let scale_factor = width_factor.min(height_factor);

                    self.dimension = Dimension::new(
                        image_information.width * scale_factor,
                        image_information.height * scale_factor,
                    )
                }
                ScaleMode::Fill => {
                    let scale_factor = width_factor.max(height_factor);

                    self.dimension = Dimension::new(
                        image_information.width * scale_factor,
                        image_information.height * scale_factor,
                    )
                }
                ScaleMode::Stretch => self.dimension = requested_size,
            }
        }

        self.dimension
    }
}

impl<Id: ReadState<T=ImageId>, C: ReadState<T=Style>> Image<Id, C> {
    fn load_remote_raster(ctx: &mut LayoutContext, url: &Url) {
        todo!("Load image from url: {}", url);
    }

    fn load_local_raster(ctx: &mut LayoutContext, path: &PathBuf) {
        let full_path = if path.is_relative() {
            let assets = crate::locate_folder::Search::KidsThenParents(3, 5)
                .for_folder("assets")
                .unwrap();

            assets.join(path)
        } else {
            path.to_path_buf()
        };

        let image = image::open(&full_path)
            .expect("Couldn't load image")
            .pre_multiplied();

        let texture = Texture {
            width: image.width(),
            height: image.height(),
            bytes_per_row: image.width() * 4,
            format: TextureFormat::RGBA8,
            data: &image.to_rgba8().into_raw(),
        };

        ctx.image.update_texture(&ImageId::Local(Arc::new(path.clone()), ImageIdFormat::Raster), texture, ctx.env);
    }
}

impl<Id: ReadState<T=ImageId>, C: ReadState<T=Style>> Render for Image<Id, C> {
    fn render(&mut self, ctx: &mut RenderContext) {
        self.sync(ctx.env);

        if let Some(color) = &mut self.color {
            color.sync(ctx.env);
        }

        let image_id = &*self.image_id.value();

        let source_rect = match self.src_rect {
            None => Rect::from_corners(Position::new(0.0, 1.0), Position::new(1.0, 0.0)),
            Some(src_rect) => {
                let image_dimensions = match ctx.image.metrics(image_id, ctx.env) {
                    ImageMetrics::Unknown => Dimension::new(100.0, 100.0),
                    ImageMetrics::Raster { width, height } => Dimension::new(width as Scalar, height as Scalar),
                    ImageMetrics::Vector { dimension } => dimension
                };

                let (l, r, b, t) = src_rect.l_r_b_t();

                Rect::from_corners(
                    Position::new(l / image_dimensions.width, b / image_dimensions.height),
                    Position::new(r / image_dimensions.width, t / image_dimensions.height),
                )
            }
        };

        if let Some(color) = self.color.as_ref().map(|col| col.value().clone()) {
            ctx.style(color.convert(self.position, self.dimension), |this| {
                this.image(image_id, Rect::new(self.position, self.dimension), ImageOptions { source_rect: Some(source_rect), mode: self.mode })
            })
        } else {
            ctx.image(image_id, Rect::new(self.position, self.dimension), ImageOptions { source_rect: Some(source_rect), mode: self.mode })
        }
    }
}



impl<Id: ReadState<T=ImageId>, C: ReadState<T=Style>> Accessibility for Image<Id, C> {
    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {
        self.sync(ctx.env);

        let mut builder = Node::new(Role::Label);

        let scale_factor = ctx.env.get_mut::<SceneManager>()
            .map(|a| a.scale_factor())
            .unwrap_or(1.0);

        builder.set_bounds(accesskit::Rect::from_origin_size(
            Point::new(self.x() * scale_factor, self.y() * scale_factor),
            Size::new(self.width() * scale_factor, self.height() * scale_factor),
        ));

        if ctx.hidden {
            builder.set_hidden();
        }

        if let Some(label) = ctx.inherited_label {
            builder.set_label(label);
        } /*else if !self.decorative {
            if let Some(id) = self.image_id.value().as_ref() {
                if let Some(file_name) = id.file_stem() {
                    builder.set_label(file_name);
                }
            }
        }*/

        if let Some(hint) = ctx.inherited_hint {
            builder.set_description(hint);
        }

        if let Some(value) = ctx.inherited_value {
            builder.set_value(value);
        }

        builder.set_author_id(format!("{:?}", self.id()));

        ctx.nodes.push(self.id(), builder);

        ctx.children.push(self.id());
    }
}

impl<Id: ReadState<T=ImageId>, C: ReadState<T=Style>> CommonWidget for Image<Id, C> {
    CommonWidgetImpl!(self, child: (), position: self.position, dimension: self.dimension, flexibility: 10);
}