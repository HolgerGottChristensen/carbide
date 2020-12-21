//! A simple, non-interactive widget for drawing an `Image`.

use {Color, Ui};
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

use Scalar;
use layout::basic_layouter::BasicLayouter;
use event::event::Event;
use event_handler::{WidgetEvent, MouseEvent, KeyboardEvent};
use widget::primitive::widget::WidgetExt;
use state::state::{StateList};
use flags::Flags;
use widget::widget_iterator::{WidgetIter, WidgetIterMut};
use layout::Layout;
use layout::layouter::Layouter;
use state::environment::Environment;


/// A primitive and basic widget for drawing an `Image`.
#[derive(Debug, Clone, WidgetCommon_)]
pub struct Image<S> {
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

    pub children: Vec<Box<dyn Widget<S>>>,
}

impl<S> Event<S> for Image<S> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, global_state: &mut S) {
        ()
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, global_state: &mut S) {
        ()
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        unimplemented!()
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, state: StateList, global_state: &mut S) -> StateList {
        self.process_mouse_event_default(event, consumed, state, global_state)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, state: StateList, global_state: &mut S) -> StateList {
        self.process_keyboard_event_default(event, state, global_state)
    }

    fn get_state(&self, current_state: StateList) -> StateList {
        current_state
    }

    fn apply_state(&mut self, states: StateList, _: &S) -> StateList {
        states
    }

    fn sync_state(&mut self, states: StateList, global_state: &S) {
        self.sync_state_default(states, global_state);
    }
}

impl<S> Layout<S> for Image<S> {
    fn flexibility(&self) -> u32 {
        10
    }

    fn calculate_size(&mut self, _: Dimensions, env: &Environment) -> Dimensions {
        let dim = self.dimension;

        for child in self.get_children_mut() {
            child.calculate_size(dim, env);
        }

        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = self.dimension;

        for child in self.get_children_mut() {
            positioning(position, dimension, child);
            child.position_children();
        }
    }
}

impl<S> Render<S> for Image<S> {

    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        //let color = Color::random();
        let kind = PrimitiveKind::Image {
            color: None,
            image_id: self.image_id,
            source_rect: self.src_rect,
        };

        let rect = Rect::new(self.position, self.dimension);
        let mut prims: Vec<Primitive> = vec![new_primitive(node_index(0), kind, rect, rect)];
        prims.extend(Rectangle::<S>::rect_outline(rect.clone(), 1.0));
        let children: Vec<Primitive> = self.get_children().flat_map(|f| f.get_primitives(fonts)).collect();
        prims.extend(children);

        return prims;
    }
}

impl<S> CommonWidget<S> for Image<S> {
    fn get_id(&self) -> Uuid {
        unimplemented!()
    }

    fn get_flag(&self) -> Flags {
        Flags::Empty
    }

    fn get_children(&self) -> WidgetIter<S> {
        self.children
            .iter()
            .rfold(WidgetIter::Empty, |acc, x| {
                if x.get_flag() == Flags::Proxy {
                    WidgetIter::Multi(Box::new(x.get_children()), Box::new(acc))
                } else {
                    WidgetIter::Single(x, Box::new(acc))
                }
            })
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
        self.children
            .iter_mut()
            .rfold(WidgetIterMut::Empty, |acc, x| {
                if x.get_flag() == Flags::Proxy {
                    WidgetIterMut::Multi(Box::new(x.get_children_mut()), Box::new(acc))
                } else {
                    WidgetIterMut::Single(x, Box::new(acc))
                }
            })
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<S> {
        self.children.iter_mut()
            .rfold(WidgetIterMut::Empty, |acc, x| {
                WidgetIterMut::Single(x, Box::new(acc))
            })
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
    pub image_id: image::Id,
}

/// Unique styling for the `Image` widget.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle_)]
pub struct Style {
    /// Optionally specify a single color to use for the image.
    #[conrod(default = "None")]
    pub maybe_color: Option<Option<Color>>,
}


impl<S> Image<S> {

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

    pub fn new(id: image::Id, dimension: Dimensions, children: Vec<Box<dyn Widget<S>>>) -> Box<Self> {
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

    /*builder_methods!{
        pub color { style.maybe_color = Some(Option<Color>) }
    }*/

}

impl<S: 'static + Clone> WidgetExt<S> for Image<S> {}

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