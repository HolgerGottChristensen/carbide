//! A simple, non-interactive widget for drawing an `Image`.

use crate::image_map;
use crate::prelude::*;
use crate::render::primitive_kind::PrimitiveKind;
use crate::render::util::new_primitive;
use crate::widget::types::scale_mode::ScaleMode;

/// A primitive and basic widget for drawing an `Image`.
#[derive(Debug, Clone, Widget)]
pub struct Image {
    id: Uuid,
    /// The unique identifier for the image that will be drawn.
    pub image_id: image_map::Id,
    /// The rectangle area of the original source image that should be used.
    pub src_rect: Option<OldRect>,
    position: Point,
    dimension: Dimensions,
    scale_mode: ScaleMode,
    resizeable: bool,
    requested_size: Dimensions,
}

impl Image {
    pub fn new(id: image_map::Id) -> Box<Self> {
        Box::new(Image {
            id: Uuid::new_v4(),
            image_id: id,
            src_rect: None,
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            scale_mode: ScaleMode::Fit,
            resizeable: false,
            requested_size: [0.0, 0.0],
        })
    }

    /// The rectangular area of the image that we wish to display.
    ///
    /// If this method is not called, the entire image will be used.
    pub fn source_rectangle(mut self, rect: OldRect) -> Self {
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
    fn flexibility(&self) -> u32 {
        10
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment) -> Dimensions {
        self.requested_size = requested_size;

        let image_information = env.get_image_information(&self.image_id).unwrap();

        if !self.resizeable {
            self.dimension = [image_information.width as f64, image_information.height as f64];
        } else {
            let width_factor = requested_size[0] / (image_information.width as f64);
            let height_factor = requested_size[1] / (image_information.height as f64);

            match self.scale_mode {
                ScaleMode::Fit => {
                    let scale_factor = width_factor.min(height_factor);

                    self.dimension = [(image_information.width as f64) * scale_factor, (image_information.height as f64) * scale_factor]
                }
                ScaleMode::Fill => {
                    let scale_factor = width_factor.max(height_factor);

                    self.dimension = [(image_information.width as f64) * scale_factor, (image_information.height as f64) * scale_factor]
                }
                ScaleMode::Stretch => {
                    self.dimension = requested_size
                }
            }
        }

        self.dimension
    }

    fn position_children(&mut self) {}
}

impl Render for Image {
    fn get_primitives(&mut self, _: &mut Environment) -> Vec<Primitive> {
        let kind = PrimitiveKind::Image {
            color: None,
            image_id: self.image_id,
            source_rect: self.src_rect,
        };

        let rect = OldRect::new(self.position, self.dimension);

        return vec![new_primitive(kind, rect)];
    }
}

impl CommonWidget for Image {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = id
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter {
        WidgetIter::Empty
    }

    fn get_children_mut(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn get_position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimensions {
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl WidgetExt for Image {}