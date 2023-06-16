//! A simple, non-interactive widget for drawing an `Image`.

use std::ops::Deref;
use image::GenericImageView;
use carbide_core::render::RenderContext;
use carbide_core::state::StateSync;
use carbide_core::widget::{CommonWidget};

use carbide_macro::{carbide_default_builder, carbide_default_builder2};

use crate::{Color, CommonWidgetImpl, Scalar};
use crate::color::WHITE;
use crate::draw::{Dimension, Position, Rect};
use crate::draw::image::ImageId;
use crate::environment::{Environment, EnvironmentColor, EnvironmentColorState};
use crate::layout::Layout;
use crate::mesh::{MODE_ICON, MODE_IMAGE};
use crate::mesh::pre_multiply::PreMultiply;
use crate::render::{Primitive, PrimitiveKind, Render, Style};
use crate::state::{IntoReadState, NewStateSync, ReadState, TState};
use crate::widget::{Widget, WidgetExt, WidgetId};
use crate::widget::types::ScaleMode;

/// A primitive and basic widget for drawing an `Image`.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout)]
pub struct Image<Id, C> where Id: ReadState<T=Option<ImageId>> + Clone, C: ReadState<T=Style> + Clone {
    id: WidgetId,
    /// The unique identifier for the image that will be drawn.
    #[state]
    pub image_id: Id,
    /// The rectangle area of the original source image that should be used.
    src_rect: Option<Rect>,
    color: Option<C>,
    mode: u32,
    position: Position,
    dimension: Dimension,
    scale_mode: ScaleMode,
    resizeable: bool,
    requested_size: Dimension,
}

impl Image<Option<ImageId>, Style> {
    #[carbide_default_builder2]
    pub fn new<Id: IntoReadState<Option<ImageId>>>(id: Id) -> Box<Image<Id::Output, Style>> {
        Box::new(Image {
            id: WidgetId::new(),
            image_id: id.into_read_state(),
            src_rect: None,
            color: None,
            mode: MODE_IMAGE,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            scale_mode: ScaleMode::Fit,
            resizeable: false,
            requested_size: Dimension::new(0.0, 0.0),
        })
    }

    pub fn new_icon<Id: IntoReadState<Option<ImageId>>>(id: Id) -> Box<Image<Id::Output, impl ReadState<T=Style>>> {
        Box::new(Image {
            id: WidgetId::new(),
            image_id: id.into_read_state(),
            src_rect: None,
            color: Some(EnvironmentColor::Accent.style()),
            mode: MODE_ICON,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            scale_mode: ScaleMode::Fit,
            resizeable: false,
            requested_size: Dimension::new(0.0, 0.0),
        })
    }
}

impl<Id: ReadState<T=Option<ImageId>> + Clone, C: ReadState<T=Style> + Clone> Image<Id, C> {
    /// Set the source rectangle of the image to use. The rect is given in image pixel coordinates.
    /// A source rect outside the size of the image will result in a larger image, but where the
    /// bottom right is blank.
    pub fn source_rectangle(mut self, rect: Rect) -> Box<Self> {
        self.src_rect = Some(rect);
        Box::new(self)
    }

    pub fn resizeable(mut self) -> Box<Self> {
        self.resizeable = true;
        Box::new(self)
    }

    pub fn scaled_to_fit(mut self) -> Box<Self> {
        self.resizeable = true;
        self.scale_mode = ScaleMode::Fit;
        Box::new(self)
    }

    pub fn scaled_to_fill(mut self) -> Box<Self> {
        self.resizeable = true;
        self.scale_mode = ScaleMode::Fill;
        Box::new(self)
    }

    pub fn aspect_ratio(mut self, mode: ScaleMode) -> Box<Self> {
        self.scale_mode = mode;
        Box::new(self)
    }
}

impl<Id: ReadState<T=Option<ImageId>> + Clone, C: ReadState<T=Style> + Clone> Layout for Image<Id, C> {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        self.requested_size = requested_size;

        if let Some(image_id) = &*self.image_id.value() {
            if !env.image_map.contains_key(image_id) {
                let path = if image_id.is_relative() {
                    let assets = carbide_core::locate_folder::Search::KidsThenParents(3, 5)
                        .for_folder("assets")
                        .unwrap();

                    assets.join(image_id)
                } else {
                    image_id.as_ref().to_path_buf()
                };



                let image = image::open(path)
                    .expect("Couldn't load logo")
                    .pre_multiplied();

                env.image_map.insert(image_id.clone(), image);
            }
        }

        let image_information = if let Some(source_rect) = self.src_rect {
            source_rect.dimension
        } else {
            env.get_image_information(&self.image_id.value())
                .map(|i| Dimension::new(i.width as f64, i.height as f64))
                .unwrap_or(Dimension::new(100.0, 100.0))
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

impl<Id: ReadState<T=Option<ImageId>> + Clone, C: ReadState<T=Style> + Clone> Render for Image<Id, C> {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        self.capture_state(env);

        if let Some(color) = &mut self.color {
            color.sync(env);
        }

        if let Some(id) = self.image_id.value().deref() {
            let source_rect = match self.src_rect {
                None => Rect::from_corners(Position::new(0.0, 1.0), Position::new(1.0, 0.0)),
                Some(src_rect) => {
                    let image = env.image_map.get(id).unwrap();

                    let (image_w, image_h) = image.dimensions();
                    let (image_w, image_h) = (image_w as Scalar, image_h as Scalar);

                    let (l, r, b, t) = src_rect.l_r_b_t();

                    Rect::from_corners(
                        Position::new(l / image_w, b / image_h),
                        Position::new(r / image_w, t / image_h),
                    )
                }
            };

            if let Some(color) = self.color.as_ref().map(|col| col.value().clone()) {
                context.style(color.convert(self.position, self.dimension), |this| {
                    this.image(id.clone(), Rect::new(self.position, self.dimension), source_rect, self.mode)
                })
            } else {
                context.image(id.clone(), Rect::new(self.position, self.dimension), source_rect, self.mode)
            }
        } else {
            println!("Missing else")
        }
    }

    fn get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        if let Some(color) = &mut self.color {
            color.sync(env);
            //color.release_state(env);
        }

        if let Some(id) = self.image_id.value().deref() {
            let kind = PrimitiveKind::Image {
                color: None,//self.color.as_ref().map(|col| *col.value()),
                image_id: id.clone(),
                source_rect: self.src_rect,
                mode: self.mode,
            };
            let rect = Rect::new(self.position, self.dimension);

            primitives.push(Primitive {
                kind,
                bounding_box: rect,
            });
        } else {
            /*let color = if let Some(color) = &self.color {
                *color.value()
            } else {
                WHITE
            };
            let kind = PrimitiveKind::RectanglePrim { color };

            let rect = Rect::new(self.position, self.dimension);

            primitives.push(Primitive {
                kind,
                bounding_box: rect,
            });*/
        }
    }
}

impl<Id: ReadState<T=Option<ImageId>> + Clone, C: ReadState<T=Style> + Clone> CommonWidget for Image<Id, C> {
    CommonWidgetImpl!(self, id: self.id, position: self.position, dimension: self.dimension, flexibility: 10);
}

impl<Id: ReadState<T=Option<ImageId>> + Clone, C: ReadState<T=Style> + Clone> WidgetExt for Image<Id, C> {}
