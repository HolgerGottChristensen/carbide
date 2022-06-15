//! A simple, non-interactive widget for drawing an `Image`.

use std::path::PathBuf;
use crate::color::WHITE;
use crate::draw::{Dimension, Position, Rect};
use crate::image_map;
use crate::mesh::{MODE_ICON, MODE_IMAGE};
use crate::prelude::*;
use crate::render::PrimitiveKind;
use crate::widget::types::ScaleMode;

/// A primitive and basic widget for drawing an `Image`.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout)]
pub struct Image {
    id: Uuid,
    /// The unique identifier for the image that will be drawn.
    #[state] pub image_id: TState<Option<image_map::ImageId>>,
    /// The rectangle area of the original source image that should be used.
    src_rect: Option<Rect>,
    color: Option<ColorState>,
    mode: u32,
    position: Position,
    dimension: Dimension,
    scale_mode: ScaleMode,
    resizeable: bool,
    requested_size: Dimension,
}

impl Image {
    pub fn new<I: Into<TState<Option<image_map::ImageId>>>>(id: I) -> Box<Self> {
        Box::new(Image {
            id: Uuid::new_v4(),
            image_id: id.into(),
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

    pub fn new_icon<I: Into<TState<Option<image_map::ImageId>>>>(id: I) -> Box<Self> {
        Box::new(Image {
            id: Uuid::new_v4(),
            image_id: id.into(),
            src_rect: None,
            color: Some(EnvironmentColor::Accent.into()),
            mode: MODE_ICON,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            scale_mode: ScaleMode::Fit,
            resizeable: false,
            requested_size: Dimension::new(0.0, 0.0),
        })
    }

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

impl Layout for Image {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        self.requested_size = requested_size;

        let image_information = if let Some(source_rect) = self.src_rect {
            source_rect.dimension
        } else {
            env.get_image_information(&self.image_id.value())
                .map(|i| Dimension::new(i.width as f64, i.height as f64))
                .unwrap_or(Dimension::new(100.0, 100.0))
        };

        if !self.resizeable {
            self.dimension = Dimension::new(
                image_information.width,
                image_information.height,
            );
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

impl Render for Image {
    fn get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        if let Some(color) = &mut self.color {
            color.sync(env);
            //color.release_state(env);
        }

        if let Some(id) = self.image_id.value().deref() {
            let kind = PrimitiveKind::Image {
                color: self.color.as_ref().map(|col| *col.value()),
                image_id: *id,
                source_rect: self.src_rect,
                mode: self.mode,
            };
            let rect = Rect::new(self.position, self.dimension);

            primitives.push(Primitive { kind, bounding_box: rect });
        } else {
            let color = if let Some(color) = &self.color {
                *color.value()
            } else {
                WHITE
            };
            let kind = PrimitiveKind::RectanglePrim { color };

            let rect = Rect::new(self.position, self.dimension);

            primitives.push(Primitive { kind, bounding_box: rect });
        }
    }
}

impl CommonWidget for Image {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn set_id(&mut self, id: WidgetId) {
        self.id = id
    }

    fn children(&self) -> WidgetIter {
        WidgetIter::Empty
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn flexibility(&self) -> u32 {
        10
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl WidgetExt for Image {}
