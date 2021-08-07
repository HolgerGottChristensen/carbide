pub use std::ops::{Deref, DerefMut};

pub use uuid::Uuid;

pub use crate::{Color, Colorable, OldRect, Point};
pub use crate::environment::*;
pub use crate::flags::Flags;
pub use crate::layout::*;
pub use crate::position::Dimensions;
pub use crate::position::Scalar;
pub use crate::render::primitive::Primitive;
pub use crate::state::*;
pub use crate::widget::common_widget::CommonWidget;
pub use crate::widget::primitive::Widget;
pub use crate::widget::primitive::widget::WidgetExt;
pub use crate::widget::Rectangle;
pub use crate::widget::render::Render;
pub use crate::widget::widget_iterator::{WidgetIter, WidgetIterMut};

