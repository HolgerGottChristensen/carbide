//! A simple, non-interactive widget for drawing an `Image`.

use crate::draw::{Dimension, Position, Rect};
use crate::image_map;
use crate::prelude::*;
use crate::render::new_primitive;
use crate::render::PrimitiveKind;
use crate::widget::types::ScaleMode;

/// A primitive and basic widget for drawing an `Image`.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout)]
pub struct Image {
    id: Uuid,
    /// The unique identifier for the image that will be drawn.
    pub image_id: image_map::Id,
    /// The rectangle area of the original source image that should be used.
    pub src_rect: Option<Rect>,
    position: Position,
    dimension: Dimension,
    scale_mode: ScaleMode,
    resizeable: bool,
    requested_size: Dimension,
}

impl Image {
    pub fn new(id: image_map::Id) -> Box<Self> {
        Box::new(Image {
            id: Uuid::new_v4(),
            image_id: id,
            src_rect: None,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            scale_mode: ScaleMode::Fit,
            resizeable: false,
            requested_size: Dimension::new(0.0, 0.0),
        })
    }

    /// The rectangular area of the image that we wish to display.
    ///
    /// If this method is not called, the entire image will be used.
    pub fn source_rectangle(mut self, rect: Rect) -> Self {
        self.src_rect = Some(rect);
        self
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

        let image_information = env.get_image_information(&self.image_id).unwrap();

        if !self.resizeable {
            self.dimension = Dimension::new(
                image_information.width as f64,
                image_information.height as f64,
            );
        } else {
            let width_factor = requested_size.width / (image_information.width as f64);
            let height_factor = requested_size.height / (image_information.height as f64);

            match self.scale_mode {
                ScaleMode::Fit => {
                    let scale_factor = width_factor.min(height_factor);

                    self.dimension = Dimension::new(
                        (image_information.width as f64) * scale_factor,
                        (image_information.height as f64) * scale_factor,
                    )
                }
                ScaleMode::Fill => {
                    let scale_factor = width_factor.max(height_factor);

                    self.dimension = Dimension::new(
                        (image_information.width as f64) * scale_factor,
                        (image_information.height as f64) * scale_factor,
                    )
                }
                ScaleMode::Stretch => self.dimension = requested_size,
            }
        }

        self.dimension
    }
}

impl Render for Image {
    fn get_primitives(&mut self, _: &mut Environment) -> Vec<Primitive> {
        let kind = PrimitiveKind::Image {
            color: None,
            image_id: self.image_id,
            source_rect: self.src_rect,
        };

        let rect = Rect::new(self.position, self.dimension);

        return vec![new_primitive(kind, rect)];
    }
}

impl CommonWidget for Image {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
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
