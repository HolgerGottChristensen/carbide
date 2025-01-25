use carbide_core::CommonWidgetImpl;
use carbide_core::widget::{CommonWidget, PrimitiveStore, ShapeStyle, StrokeStyle};
use carbide_macro::carbide_default_builder2;

use crate::draw::{Dimension, Position};
use crate::misc::flags::WidgetFlag;
use crate::scene::AnyScene;
use crate::widget::{Shape, Widget, WidgetExt, WidgetId};

#[derive(Clone, Debug, Widget)]
pub struct Empty {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,
}

impl Empty {
    #[carbide_default_builder2]
    pub fn new() ->Self {
        Empty {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        }
    }
}

impl CommonWidget for Empty {
    CommonWidgetImpl!(self, child: (), position: self.position, dimension: self.dimension, flag: WidgetFlag::IGNORE);
}

impl AnyScene for Empty {
    fn request_redraw(&self) -> bool {
        false
    }

    fn has_application_focus(&self) -> bool {
        false
    }

    fn is_daemon(&self) -> bool {
        true
    }
}

impl Shape for Empty {
    fn get_triangle_store_mut(&mut self) -> &mut PrimitiveStore {
        unimplemented!()
    }

    fn get_stroke_style(&self) -> StrokeStyle {
        unimplemented!()
    }

    fn get_shape_style(&self) -> ShapeStyle {
        unimplemented!()
    }
}
