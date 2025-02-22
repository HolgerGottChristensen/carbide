use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position, Scalar};
use crate::layout::{calculate_size_vstack, Layout, LayoutContext, position_children_vstack};
use crate::widget::{CommonWidget, CrossAxisAlignment, Widget, WidgetId, Sequence};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct VStack<W> where W: Sequence
{
    #[id] id: WidgetId,
    children: W,
    position: Position,
    dimension: Dimension,
    spacing: Scalar,
    cross_axis_alignment: CrossAxisAlignment,
}

impl<W: Sequence> VStack<W> {
    pub fn new(children: W) -> VStack<W> {
        VStack {
            id: WidgetId::new(),
            children,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            spacing: 10.0,
            cross_axis_alignment: CrossAxisAlignment::Center,
        }
    }

    pub fn cross_axis_alignment(mut self, alignment: CrossAxisAlignment) -> Self {
        self.cross_axis_alignment = alignment;
        self
    }

    pub fn spacing(mut self, spacing: f64) -> Self {
        self.spacing = spacing;
        self
    }
}

impl<W: Sequence> Layout for VStack<W> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let spacing = self.spacing;
        calculate_size_vstack(self, spacing, requested_size, ctx);
        self.dimension
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let spacing = self.spacing;
        let cross_axis_alignment = self.cross_axis_alignment;
        position_children_vstack(self, spacing, cross_axis_alignment, ctx)
    }
}

impl<W: Sequence> CommonWidget for VStack<W> {
    CommonWidgetImpl!(self, child: self.children, position: self.position, dimension: self.dimension, flexibility: 1);
}