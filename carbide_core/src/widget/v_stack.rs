
use carbide_macro::carbide_default_builder;

use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::flags::Flags;
use crate::layout::{calculate_size_vstack, Layout, position_children_vstack};
use crate::{CommonWidgetImpl, Scalar};
use crate::widget::{CommonWidget, CrossAxisAlignment, Widget, WidgetExt, WidgetId, WidgetIter, WidgetIterMut};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct VStack {
    id: WidgetId,
    children: Vec<Box<dyn Widget>>,
    position: Position,
    dimension: Dimension,
    spacing: Scalar,
    cross_axis_alignment: CrossAxisAlignment,
}

impl VStack {

    #[carbide_default_builder]
    pub fn new(children: Vec<Box<dyn Widget>>) -> Box<Self> {}

    pub fn new(children: Vec<Box<dyn Widget>>) -> Box<Self> {
        Box::new(VStack {
            id: WidgetId::new(),
            children,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            spacing: 10.0,
            cross_axis_alignment: CrossAxisAlignment::Center,
        })
    }

    pub fn cross_axis_alignment(mut self, alignment: CrossAxisAlignment) -> Box<Self> {
        self.cross_axis_alignment = alignment;
        Box::new(self)
    }

    pub fn spacing(mut self, spacing: f64) -> Box<Self> {
        self.spacing = spacing;
        Box::new(self)
    }
}

impl Layout for VStack {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let spacing = self.spacing;
        calculate_size_vstack(self, spacing, requested_size, env);
        self.dimension
    }

    fn position_children(&mut self, env: &mut Environment) {
        let spacing = self.spacing;
        let cross_axis_alignment = self.cross_axis_alignment;
        position_children_vstack(self, spacing, cross_axis_alignment, env)
    }
}

impl CommonWidget for VStack {
    CommonWidgetImpl!(self, id: self.id, children: self.children, position: self.position, dimension: self.dimension, flexibility: 1);
}

impl WidgetExt for VStack {}
