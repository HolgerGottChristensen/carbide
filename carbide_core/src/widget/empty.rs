use carbide::draw::DrawOptions;
use carbide_core::CommonWidgetImpl;
use carbide_core::widget::{CommonWidget};
use carbide_macro::carbide_default_builder2;

use crate::draw::{Dimension, DrawShape, Position};
use crate::misc::flags::WidgetFlag;
use crate::scene::AnyScene;
use crate::widget::{AnyShape, Widget, WidgetId};

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

impl AnyShape for Empty {
    fn cache_key(&self) -> Option<WidgetId> {
        todo!()
    }

    fn description(&self) -> DrawShape {
        todo!()
    }

    fn options(&self) -> DrawOptions {
        todo!()
    }
}
