//! A simple, non-interactive widget for drawing an `Image`.

use {Color, OldWidget, Ui};
use ::{image, Point};
use position::{Dimension, Rect, Dimensions};
use ::{widget, text};
use widget::render::Render;
use render::primitive::Primitive;
use graph::Container;
use widget::{Id, Rectangle};
use text::font::Map;
use render::primitive_kind::PrimitiveKind;
use render::util::new_primitive;
use daggy::petgraph::graph::node_index;
use widget::primitive::Widget;
use widget::common_widget::CommonWidget;
use uuid::Uuid;
use widget::layout::Layout;
use widget::envelope_editor::EnvelopePoint;
use Scalar;
use layout::basic_layouter::BasicLayouter;
use event::event::Event;
use event_handler::{WidgetEvent, MouseEvent, KeyboardEvent};
use widget::primitive::widget::WidgetExt;


/// A primitive and basic widget for drawing an `Image`.
#[derive(Debug, WidgetCommon_)]
pub struct Image {
    /// Data necessary and common for all widget builder render.
    #[conrod(common_builder)]
    pub common: widget::CommonBuilder,
    /// The unique identifier for the image that will be drawn.
    pub image_id: image::Id,
    /// The rectangle area of the original source image that should be used.
    pub src_rect: Option<Rect>,
    /// Unique styling.
    pub style: Style,
    position: Point,
    dimension: Dimensions,

    pub children: Vec<Box<dyn Widget>>,
}

impl Event for Image {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool) {
        unimplemented!()
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent) {
        ()
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        unimplemented!()
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool) {
        self.process_mouse_event_default(event, consumed);
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent) {
        self.process_keyboard_event_default(event);
    }
}

impl Layout for Image {
    fn flexibility(&self) -> u32 {
        10
    }

    fn calculate_size(&mut self, _: Dimensions, fonts: &Map) -> Dimensions {

        for child in &mut self.children {
            child.calculate_size(self.dimension, fonts);
        }

        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = self.dimension;

        for child in &mut self.children {
            positioning(position, dimension, child);
            child.position_children();
        }
    }
}

impl Render for Image {
    fn layout(&mut self, proposed_size: Dimensions, fonts: &text::font::Map, positioner: &dyn Fn(&mut dyn CommonWidget, Dimensions)) {
        unimplemented!()
    }

    fn render(self, id: Id, clip: Rect, container: &Container) -> Option<Primitive> {
        //let color = Color::random();
        let kind = PrimitiveKind::Image {
            color: None,
            image_id: self.image_id,
            source_rect: self.src_rect,
        };
        return Some(new_primitive(id, kind, clip, container.rect));
    }

    fn get_primitives(&self, proposed_dimensions: Dimensions, fonts: &Map) -> Vec<Primitive> {
        //let color = Color::random();
        let kind = PrimitiveKind::Image {
            color: None,
            image_id: self.image_id,
            source_rect: self.src_rect,
        };

        let rect = Rect::new(self.position, self.dimension);
        let mut prims: Vec<Primitive> = vec![new_primitive(node_index(0), kind, rect, rect)];
        prims.extend(Rectangle::rect_outline(rect.clone(), 1.0));
        let children: Vec<Primitive> = self.get_children().iter().flat_map(|f| f.get_primitives(proposed_dimensions, fonts)).collect();
        prims.extend(children);

        return prims;
    }
}

impl CommonWidget for Image {
    fn get_id(&self) -> Uuid {
        unimplemented!()
    }

    fn get_children(&self) -> &Vec<Box<dyn Widget>> {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut Vec<Box<dyn Widget>> {
        &mut self.children
    }

    fn get_position(&self) -> Dimensions {
        unimplemented!()
    }

    fn get_x(&self) -> Scalar {
        self.position[0]
    }

    fn set_x(&mut self, x: f64) {
        self.position = Point::new(x, self.position.get_y());
    }

    fn get_y(&self) -> Scalar {
        self.position[1]
    }

    fn set_y(&mut self, y: f64) {
        self.position = Point::new(self.position.get_x(), y);
    }


    fn get_size(&self) -> [f64; 2] {
        unimplemented!()
    }

    fn get_width(&self) -> f64 {
        self.dimension[0]
    }

    fn get_height(&self) -> f64 {
        self.dimension[1]
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
    pub image_id: image::Id,
}

/// Unique styling for the `Image` widget.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle_)]
pub struct Style {
    /// Optionally specify a single color to use for the image.
    #[conrod(default = "None")]
    pub maybe_color: Option<Option<Color>>,
}


impl Image {

    /// Construct a new `Image`.
    ///
    /// Note that the `Image` widget does not require borrowing or owning any image data directly.
    /// Instead, image data is stored within a `conrod::image::Map` where `image::Id`s are mapped
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
    /// During rendering, conrod will take the `image::Map`, retrieve the data associated with each
    /// image and yield it via the `render::Primitive::Image` variant.
    ///
    /// Note: this implies that the type must be the same for all `Image` widgets instantiated via
    /// the same `Ui`. In the case that you require multiple different render of images, we
    /// recommend that you either:
    ///
    /// 1. use an enum with a variant for each type
    /// 2. use a trait object, where the trait is implemented for each of your image render or
    /// 3. use an index type which may be mapped to your various image render.
    pub fn old_new(image_id: image::Id) -> Self {
        Image {
            common: widget::CommonBuilder::default(),
            image_id,
            src_rect: None,
            style: Style::default(),
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            children: vec![]
        }
    }

    pub fn new(id: image::Id, dimension: Dimensions, children: Vec<Box<dyn Widget>>) -> Box<Self> {
        Box::new(Image {
            common: Default::default(),
            image_id: id,
            src_rect: None,
            style: Default::default(),
            position: [0.0,0.0],
            dimension,
            children
        })
    }

    /// The rectangular area of the image that we wish to display.
    ///
    /// If this method is not called, the entire image will be used.
    pub fn source_rectangle(mut self, rect: Rect) -> Self {
        self.src_rect = Some(rect);
        self
    }

    builder_methods!{
        pub color { style.maybe_color = Some(Option<Color>) }
    }

}

impl WidgetExt for Image {}

impl OldWidget for Image {
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

    fn default_x_dimension(&self, ui: &Ui) -> Dimension {
        match self.src_rect.as_ref() {
            Some(rect) => Dimension::Absolute(rect.w()),
            None => widget::default_x_dimension(self, ui),
        }
    }

    fn default_y_dimension(&self, ui: &Ui) -> Dimension {
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
