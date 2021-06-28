//! Widgets are the core building blocks for every carbide user interface.
//!
//! This module contains items related to the implementation of the `Widget` trait. It also
//! re-exports all widgets (and their modules) that are provided by carbide.

// Other useful uses when importing widgets.
use uuid::Uuid;

pub use carbide_core::layout::CrossAxisAlignment;
pub use carbide_core::layout::Layout;
pub use carbide_core::layout::layouter::Layouter;
pub use carbide_core::state::state_ext::StateExt;
pub use carbide_core::window::TWindow;

pub use crate::{Color, Colorable, OldRect, Point};
pub use crate::environment::environment::Environment;
pub use crate::environment::environment_color::EnvironmentColor;
pub use crate::environment::environment_font_size::EnvironmentFontSize;
pub use crate::flags::Flags;
pub use crate::focus::Focus;
pub use crate::focus::Focusable;
pub use crate::focus::Refocus;
pub use crate::layout::basic_layouter::BasicLayouter;
pub use crate::layout::layout::SingleChildLayout;
pub use crate::position::Dimensions;
pub use crate::position::Scalar;
pub use crate::state::*;
pub use crate::state::global_state::GlobalState;
pub use crate::state::state::CommonState;

pub use self::common_widget::CommonWidget;
pub use self::primitive::canvas::canvas::Canvas;
pub use self::primitive::canvas::context::Context;
pub use self::primitive::canvas::context::ContextAction;
pub use self::primitive::environment_updating::EnvUpdating;
pub use self::primitive::foreach::ForEach;
pub use self::primitive::frame::*;
pub use self::primitive::h_stack::*;
pub use self::primitive::if_else::IfElse;
pub use self::primitive::image::{self, Image};
pub use self::primitive::offset::Offset;
pub use self::primitive::overlaid_layer::OverlaidLayer;
pub use self::primitive::padding::Padding;
pub use self::primitive::scroll::Scroll;
pub use self::primitive::shape::capsule::Capsule;
pub use self::primitive::shape::ellipse::{self, Ellipse};
pub use self::primitive::shape::polygon::{self, Polygon};
pub use self::primitive::shape::rectangle::{self, Rectangle};
pub use self::primitive::shape::rounded_rectangle::RoundedRectangle;
pub use self::primitive::shared_state::SharedState;
pub use self::primitive::spacer::Spacer;
pub use self::primitive::text::{self, Text};
pub use self::primitive::v_stack::*;
pub use self::primitive::Widget;
pub use self::primitive::widget::WidgetExt;
pub use self::primitive::z_stack::*;
pub use self::render::ChildRender;
pub use self::types::corner_radii::CornerRadii;
pub use self::types::edge_insets::EdgeInsets;
pub use self::types::scale_mode::ScaleMode;
pub use self::types::scroll_direction::ScrollDirection;
pub use self::types::spacer_direction::SpacerDirection;
pub use self::widget_iterator::{WidgetIter, WidgetIterMut};

pub type Id = Uuid;

pub mod render;


// Primitive widget modules.
pub mod primitive;


pub mod common_widget;
pub mod widget_iterator;
pub mod types;
