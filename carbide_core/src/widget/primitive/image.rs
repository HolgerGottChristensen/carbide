//! A simple, non-interactive widget for drawing an `Image`.

use crate::prelude::*;
use crate::image_map;
use crate::widget::types::scale_mode::ScaleMode;
use crate::render::primitive_kind::PrimitiveKind;
use crate::render::util::new_primitive;

/// A primitive and basic widget for drawing an `Image`.
#[derive(Debug, Clone, Widget)]
pub struct Image {
    id: Uuid,
    /// The unique identifier for the image that will be drawn.
    pub image_id: image_map::Id,
    /// The rectangle area of the original source image that should be used.
    pub src_rect: Option<Rect>,
    /// Unique styling.
    pub style: Style,
    position: Point,
    dimension: Dimensions,
    scale_mode: ScaleMode,
    resizeable: bool,
    requested_size: Dimensions
}

impl<GS: GlobalState> WidgetExt<GS> for Image {}

impl<S: GlobalState> Layout<S> for Image {
    fn flexibility(&self) -> u32 {
        10
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment<S>) -> Dimensions {
        self.requested_size = requested_size;

        let image_information = env.get_image_information(&self.image_id).unwrap();

        if !self.resizeable {

            self.dimension = [image_information.width as f64, image_information.height as f64];
        } else {
            let width_factor = requested_size[0] / (image_information.width as f64) ;
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

    fn position_children(&mut self) {

    }
}

impl<GS: GlobalState> Render<GS> for Image {

    fn get_primitives(&mut self, _: &Environment<GS>, _: &GS) -> Vec<Primitive> {
        let kind = PrimitiveKind::Image {
            color: None,
            image_id: self.image_id,
            source_rect: self.src_rect,
        };

        let rect = Rect::new(self.position, self.dimension);

        let mut prims: Vec<Primitive> = vec![new_primitive(kind, rect)];
        prims.extend(Rectangle::<GS>::debug_outline(rect.clone(), 1.0));
        return prims;
    }
}

impl<S: GlobalState> CommonWidget<S> for Image {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = id
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<S> {
        WidgetIter::Empty
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
        WidgetIterMut::Empty
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<S> {
        WidgetIterMut::Empty
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut<S> {
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

/// Unique `State` to be stored between updates for the `Image`.
#[derive(Copy, Clone)]
pub struct State {
    /// The rectangular area of the image that we wish to display.
    ///
    /// If `None`, the entire image will be used.
    pub src_rect: Option<Rect>,
    /// The unique identifier for the image's associated data that will be drawn.
    pub image_id: image_map::Id,
}

/// Unique styling for the `Image` widget.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Style {
    /// Optionally specify a single color to use for the image.
    pub maybe_color: Option<Option<Color>>,
}


impl Image {

    /// Construct a new `Image`.
    ///
    /// Note that the `Image` widget does not require borrowing or owning any image data directly.
    /// Instead, image data is stored within a `carbide::image::Map` where `image::Id`s are mapped
    /// to their associated data.
    ///
    /// This is done for a few reasons:
    ///
    /// - To avoid requiring that the widget graph owns an instance of each image
    /// - To avoid requiring that the user passes the image data to the `Image` every update
    /// unnecessarily
    /// - To make it easier for users to borrow and mutate their images without needing to index
    /// into the `Ui`'s widget graph (which also requires casting render).
    ///
    /// During rendering, carbide will take the `image::Map`, retrieve the data associated with each
    /// image and yield it via the `render::Primitive::Image` variant.
    ///
    /// Note: this implies that the type must be the same for all `Image` widgets instantiated via
    /// the same `Ui`. In the case that you require multiple different render of images, we
    /// recommend that you either:
    ///
    /// 1. use an enum with a variant for each type
    /// 2. use a trait object, where the trait is implemented for each of your image render or
    /// 3. use an index type which may be mapped to your various image render.
    pub fn old_new(image_id: image_map::Id) -> Self {
        Image {
            id: Uuid::new_v4(),
            image_id,
            src_rect: None,
            style: Style::default(),
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            scale_mode: ScaleMode::Fit,
            resizeable: false,
            requested_size: [0.0, 0.0]
        }
    }

    pub fn new(id: image_map::Id) -> Box<Self> {
        Box::new(Image {
            id: Uuid::new_v4(),
            image_id: id,
            src_rect: None,
            style: Default::default(),
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            scale_mode: ScaleMode::Fit,
            resizeable: false,
            requested_size: [0.0, 0.0]
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

    /*builder_methods!{
        pub color { style.maybe_color = Some(Option<Color>) }
    }*/

}

/*impl<S> OldWidget<S> for Image<S> {
    type State = State;
    type Style = Style;
    type Event = ();

    fn init_state(&self, _: widget::id::Generator) -> Self::State {
        State {
            src_rect: None,
            image_id: self.image_id,
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn default_x_dimension(&self, ui: &Ui<S>) -> Dimension {
        match self.src_rect.as_ref() {
            Some(rect) => Dimension::Absolute(rect.w()),
            None => widget::default_x_dimension(self, ui),
        }
    }

    fn default_y_dimension(&self, ui: &Ui<S>) -> Dimension {
        match self.src_rect.as_ref() {
            Some(rect) => Dimension::Absolute(rect.h()),
            None => widget::default_y_dimension(self, ui),
        }
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { state, .. } = args;
        let Image { image_id, src_rect, .. } = self;

        if state.image_id != image_id {
            state.update(|state| state.image_id = image_id);
        }
        if state.src_rect != src_rect {
            state.update(|state| state.src_rect = src_rect);
        }
    }

}
*/