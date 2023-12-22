use smallvec::{SmallVec, smallvec};

use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::layout::{BasicLayouter, Layout, LayoutContext, Layouter};
use crate::widget::{AnyWidget, CommonWidget, Widget, WidgetExt, WidgetId, WidgetSequence};

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct ZStack<W> where W: WidgetSequence {
    id: WidgetId,
    children: W,
    position: Position,
    dimension: Dimension,
    alignment: Box<dyn Layouter>,
}

impl<W: WidgetSequence> ZStack<W> {

    #[carbide_default_builder2]
    pub fn new(children: W) -> Self {
        ZStack {
            id: WidgetId::new(),
            children,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            alignment: Box::new(BasicLayouter::Center),
        }
    }

    pub fn with_alignment(mut self, layouter: BasicLayouter) -> Self {
        self.alignment = Box::new(layouter);
        self
    }
}

impl<W: WidgetSequence> Layout for ZStack<W> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let mut children_flexibility: SmallVec<[(u32, &mut dyn AnyWidget); 5]> = smallvec![];

        self.foreach_child_mut(&mut |child| {
            children_flexibility.push((child.flexibility(), child));
        });

        children_flexibility.sort_by(|(a, _), (b, _)| a.cmp(&b));
        children_flexibility.reverse();

        let mut max_width = 0.0;
        let mut max_height = 0.0;

        for (_, child) in children_flexibility {
            let new_requested_size = Dimension::new(
                requested_size.width.max(max_width),
                requested_size.height.max(max_height),
            );
            let chosen_size = child.calculate_size(new_requested_size, ctx);

            if chosen_size.width > max_width {
                max_width = chosen_size.width;
            }

            if chosen_size.height > max_height {
                max_height = chosen_size.height;
            }
        }

        self.dimension = Dimension::new(max_width, max_height);
        self.dimension
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let positioning = self.alignment.positioner();
        let position = self.position;
        let dimension = self.dimension;

        self.foreach_child_mut(&mut |child| {
            positioning(position, dimension, child);
            child.position_children(ctx);
        });
    }
}

impl<W: WidgetSequence> CommonWidget for ZStack<W> {
    CommonWidgetImpl!(self, id: self.id, child: self.children, position: self.position, dimension: self.dimension, flexibility: 1, alignment: self.alignment);
}

impl<W: WidgetSequence> WidgetExt for ZStack<W> {}
