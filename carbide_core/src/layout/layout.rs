use crate::draw::{Dimension, InnerImageContext};
use crate::environment::Environment;
use crate::text::InnerTextContext;
use crate::widget::CommonWidget;

pub trait Layout: CommonWidget {
    /// This method is used to calculate and set the size of a widget. The parent widget provides
    /// a requested size. This is often the available space for the widget. The widget is not
    /// forced to use the requested size, and may be any size it wants. For example a non-resizable
    /// image will have the size of the image, even if it is larger than the requested size.
    /// The widget should return the chosen dimensions back to the caller.
    /// The default behavior is to calculate the size of the first child and return that as the
    /// chosen size. If no child are present, the widget will chose the requested size.
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {

        let mut chosen = requested_size;
        let mut first = true;

        self.foreach_child_mut(&mut |child| {
            if !first {
                return;
            }
            chosen = child.calculate_size(requested_size, ctx);

            first = false;
        });

        self.set_dimension(chosen);
        chosen
    }

    /// This method positions the children of the widget. When positioning, we use the alignment of
    /// the widget to position. The default alignment is Center.
    /// The default behavior is to position the first child using the alignment of the widget. If
    /// no child are present the default is a no-op.
    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let positioning = self.alignment();
        let position = self.position();
        let dimension = self.dimension();

        let mut first = true;

        self.foreach_child_mut(&mut |child| {
            if !first {
                return;
            }

            child.set_position(positioning.position(position, dimension, child.dimension()));
            child.position_children(ctx);

            first = false;
        });
    }
}

pub struct LayoutContext<'a> {
    pub text: &'a mut dyn InnerTextContext,
    pub image: &'a mut dyn InnerImageContext,
    pub env: &'a mut Environment,
}
