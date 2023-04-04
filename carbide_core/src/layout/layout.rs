use std::ops::DerefMut;

use crate::draw::Dimension;
use crate::environment::Environment;
use crate::widget::CommonWidget;

pub trait Layout: CommonWidget {
    /// This method is used to calculate and set the size of a widget. The parent widget provides
    /// a requested size. This is often the available space for the widget. The widget is not
    /// forced to use the requested size, and may be any size it wants. For example a non-resizable
    /// image will have the size of the image, even if it is larger than the requested size.
    /// The widget should return the chosen dimensions back to the caller.
    /// The default behavior is to calculate the size of the first child and return that as the
    /// chosen size. If no child are present, the widget will chose the requested size.
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let chosen = if let Some(mut first_child) = self.children_mut().next() {
            let dimension = first_child.calculate_size(requested_size, env);
            dimension
        } else {
            requested_size
        };
        self.set_dimension(chosen);
        chosen
    }

    /// This method positions the children of the widget. When positioning, we use the alignment of
    /// the widget to position. The default alignment is Center.
    /// The default behavior is to position the first child using the alignment of the widget. If
    /// no child are present the default is a no-op.
    fn position_children(&mut self, env: &mut Environment) {
        let positioning = self.alignment().positioner();
        let position = self.position();
        let dimension = self.dimension();
        if let Some(mut first_child) = self.children_mut().next() {
            positioning(position, dimension, first_child.deref_mut());
            first_child.position_children(env);
        }
    }
}
