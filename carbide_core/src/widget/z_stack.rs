use smallvec::{SmallVec, smallvec};

use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position, Alignment};
use crate::layout::{Layout, LayoutContext};
use crate::widget::{AnyWidget, CommonWidget, Widget, WidgetExt, WidgetId, Sequence};

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct ZStack<W> where W: Sequence
{
    #[id] id: WidgetId,
    children: W,
    position: Position,
    dimension: Dimension,
    alignment: Alignment,
}

impl<W: Sequence> ZStack<W> {

    #[carbide_default_builder2]
    pub fn new(children: W) -> Self {
        ZStack {
            id: WidgetId::new(),
            children,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            alignment: Alignment::Center,
        }
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
}

impl<W: Sequence> Layout for ZStack<W> {
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
        let alignment = self.alignment();
        let position = self.position;
        let dimension = self.dimension;

        self.foreach_child_mut(&mut |child| {
            child.set_position(alignment.position(position, dimension, child.dimension()));
            child.position_children(ctx);
        });
    }
}

impl<W: Sequence> CommonWidget for ZStack<W> {
    CommonWidgetImpl!(self, child: self.children, position: self.position, dimension: self.dimension, flexibility: 1, alignment: self.alignment);
}