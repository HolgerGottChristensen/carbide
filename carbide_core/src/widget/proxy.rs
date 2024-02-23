use carbide_core::render::RenderContext;
use carbide_core::state::IntoReadState;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Color, Dimension, Position, Rect};
use crate::draw::draw_style::DrawStyle;
use crate::environment::Environment;
use crate::flags::WidgetFlag;
use crate::layout::{Layout, LayoutContext};
use crate::render::Render;
use crate::state::ReadState;
use crate::widget::{CommonWidget, Empty, Widget, WidgetExt, WidgetId, WidgetSequence};

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
pub struct Proxy<W> where W: WidgetSequence {
    id: WidgetId,
    children: W,
    position: Position,
    dimension: Dimension,
}

impl Proxy<Empty> {
    #[carbide_default_builder2]
    pub fn new<W: WidgetSequence>(children: W) -> Proxy<W> {
        Proxy {
            id: WidgetId::new(),
            children,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        }
    }
}

impl<W: WidgetSequence> CommonWidget for Proxy<W> {
    CommonWidgetImpl!(self, id: self.id, child: self.children, position: self.position, dimension: self.dimension, flag: WidgetFlag::PROXY);
}

impl<W: WidgetSequence> WidgetExt for Proxy<W> {}
