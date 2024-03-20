//! A simple, non-interactive widget for drawing an `Image`.
use std::ops::Deref;

use carbide_core::render::RenderContext;
use carbide_core::state::StateSync;
use carbide_core::widget::CommonWidget;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, MODE_ICON, MODE_IMAGE, Position, Rect, Scalar, Texture, TextureFormat};
use crate::draw::image::ImageId;
use crate::environment::{Environment, EnvironmentColor,};
use crate::layout::{Layout, LayoutContext};
use crate::draw::pre_multiply::PreMultiply;
use crate::render::{Render, Style};
use crate::state::{IntoReadState, ReadState};
use crate::widget::{Widget, WidgetExt, WidgetId};
use crate::widget::types::ScaleMode;

/// A primitive and basic widget for drawing an `Image`.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout)]
pub struct Image<Id, C> where Id: ReadState<T=Option<ImageId>>, C: ReadState<T=Style> {
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
}

impl Image<Option<ImageId>, Style> {
    #[carbide_default_builder2]
    pub fn new<Id: IntoReadState<Option<ImageId>>>(id: Id) -> Image<Id::Output, Style> {
        Image {
            id: WidgetId::new(),
            image_id: id.into_read_state(),
            src_rect: None,
            color: None,
            mode: MODE_IMAGE,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            scale_mode: ScaleMode::Fit,
            resizeable: false,
        }
    }

    pub fn new_icon<Id: IntoReadState<Option<ImageId>>>(id: Id) -> Image<Id::Output, impl ReadState<T=Style>> {
        Image {
            id: WidgetId::new(),
            image_id: id.into_read_state(),
            src_rect: None,
            color: Some(EnvironmentColor::Label.style()),
            mode: MODE_ICON,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            scale_mode: ScaleMode::Fit,
            resizeable: false,
        }
    }
}

impl<Id: ReadState<T=Option<ImageId>>, C: ReadState<T=Style>> Image<Id, C> {
    /// Set the source rectangle of the image to use. The rect is given in image pixel coordinates.
    /// A source rect outside the size of the image will result in a larger image, but where the
    /// bottom right is blank.
    pub fn source_rectangle(mut self, rect: Rect) -> Self {
        self.src_rect = Some(rect);
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
        }
    }
}

impl<Id: ReadState<T=Option<ImageId>>, C: ReadState<T=Style>> Layout for Image<Id, C> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {

        if let Some(image_id) = &*self.image_id.value() {
            if !ctx.image.texture_exist(image_id) {
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

                let texture = Texture {
                    width: image.width(),
                    height: image.height(),
                    bytes_per_row: image.width() * 4,
                    format: TextureFormat::RGBA8,
                    data: &image.to_rgba8().into_raw(),
                };

                ctx.image.update_texture(image_id.clone(), texture);

                //env.image_map.insert(image_id.clone(), image);
            }
        }

        let image_information = if let Some(source_rect) = self.src_rect {
            source_rect.dimension
        } else {
            let image_dimensions = self.image_id.value().as_ref().map(|id| {
                ctx.image.texture_dimensions(id)
            }).flatten().unwrap_or((100, 100));

            Dimension::new(image_dimensions.0 as Scalar, image_dimensions.1 as Scalar)
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

impl<Id: ReadState<T=Option<ImageId>>, C: ReadState<T=Style>> Render for Image<Id, C> {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        self.capture_state(env);

        if let Some(color) = &mut self.color {
            color.sync(env);
        }

        if let Some(id) = self.image_id.value().deref() {
            let source_rect = match self.src_rect {
                None => Rect::from_corners(Position::new(0.0, 1.0), Position::new(1.0, 0.0)),
                Some(src_rect) => {
                    let (image_w, image_h) = context.image.texture_dimensions(id).unwrap();
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
            //println!("Missing else")
        }
    }
}

impl<Id: ReadState<T=Option<ImageId>>, C: ReadState<T=Style>> CommonWidget for Image<Id, C> {
    CommonWidgetImpl!(self, id: self.id, child: (), position: self.position, dimension: self.dimension, flexibility: 10);
}

impl<Id: ReadState<T=Option<ImageId>>, C: ReadState<T=Style>> WidgetExt for Image<Id, C> {}
