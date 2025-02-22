use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position, Scalar};
use crate::layout::{calculate_size_hstack, Layout, LayoutContext, position_children_hstack};
use crate::widget::{CommonWidget, CrossAxisAlignment, Widget, WidgetId, Sequence};

/// # HStack
/// The horizontal stack in Carbide is one of the main layout components.
/// It is used to layout a number of other Carbide widgets horizontally in a stack.
/// In some frameworks like Qt, HStack is known as a Row. <br><br><br>
///
///
/// ## Layout explanation with fixed width widgets
/// The layout mechanism of the HStack is inspired by the HStack from SwiftUI.
///
/// In the below visualisations the symbols are used as follows.
/// '-' indicates empty space
/// 'a', 'b' and 'c' indicates different widgets
/// '=' indicates space taken up by Spacers
///
/// The most trivial case is a HStack that doesnt contain any widgets or spacers.
/// In this case the HStack will be 0 width and 0 height.
/// ```
/// use carbide_core::widget::HStack;
///
/// fn main() {
///     HStack::new(vec![]);
/// }
/// ```
///
/// This instantiation will result in a stack with the following layout:
/// | - - - - - - - - - - - |
///
/// The second case is a HStack that contains a single widget. The widget has a fixed width and
/// will by default be centered in the HStack. The HStack will have the same height and width as
/// the widget.
/// ```
/// use carbide_core::widget::{HStack, Rectangle, WidgetExt};
///
/// fn main() {
///     HStack::new(vec![
///         Rectangle::new().frame(10.0, 10.0)
///     ]);
/// }
/// ```
///
/// This instantiation will result in a stack with the following layout:
/// | - - - - - a - - - - - |
///
/// Even if we add multiple widgets to the stack they will be centered, and the width of the stack
/// will be the same as the widgets width combined (and some spacing, that will be described later).
/// ```
/// use carbide_core::widget::{HStack, Rectangle, WidgetExt};
///
/// fn main() {
///     HStack::new(vec![
///         Rectangle::new().frame(10.0, 10.0),
///         Rectangle::new().frame(10.0, 10.0),
///         Rectangle::new().frame(10.0, 10.0),
///     ]);
/// }
/// ```
/// This instantiation will result in a stack with the following layout:
/// | - - - - a b c - - - - |
///
///
/// Now if we want the widget to be in the start of the stack, we need something that will fill
/// the rest of the space. This is the `Spacer` widget. `Spacer`s will be layed out after all
/// widgets are layed out, and the space is divided evenly between all `Spacer`s.
/// ```
/// use carbide_core::widget::{HStack, Rectangle, WidgetExt, Spacer};
///
/// fn main() {
///     HStack::new(vec![
///         Rectangle::new().frame(10.0, 10.0),
///         Spacer::new()
///     ]);
/// }
/// ```
///
/// Notice that if a stack contains any spacers, the stack will take up all the space possible.
/// The above code gives us the following layout as we wanted:
/// | a = = = = = = = = = = |
///
/// Spacers are very flexible (pun intended). We can use a spacer to achieve the space-between
/// layout from the flexbox model used by css. Since the spacer will fill the remaining space,
/// it matters where you place it.
/// ```
/// use carbide_core::widget::{HStack, Rectangle, WidgetExt, Spacer};
///
/// fn main() {
///     HStack::new(vec![
///         Rectangle::new().frame(10.0, 10.0),
///         Spacer::new(),
///         Rectangle::new().frame(10.0, 10.0),
///     ]);
/// }
/// ```
/// The instantiation above will result in the following layout:
/// | a = = = = = = = = = b |
///
/// Multiple spacers can also be used and the remaining space will be equally divided between the spacers.
/// For example:
/// ```
/// use carbide_core::widget::{HStack, Rectangle, WidgetExt, Spacer};
///
/// fn main() {
///     HStack::new(vec![
///         Rectangle::new().frame(10.0, 10.0),
///         Spacer::new(),
///         Rectangle::new().frame(10.0, 10.0),
///         Spacer::new(),
///         Rectangle::new().frame(10.0, 10.0),
///     ]);
/// }
/// ```
/// will result in the following layout:
/// | a = = = = b = = = = c |
///
/// You can also get the behavior of the space-around from the flex-box layout in css using spacers.
/// Hopefully at this point you are able to guess that by placing the spacers before the first and
/// after the last widget:
/// ```
/// use carbide_core::widget::{HStack, Rectangle, WidgetExt, Spacer};
///
/// fn main() {
///     HStack::new(vec![
///         Spacer::new(),
///         Rectangle::new().frame(10.0, 10.0),
///         Spacer::new(),
///         Rectangle::new().frame(10.0, 10.0),
///         Spacer::new(),
///         Rectangle::new().frame(10.0, 10.0),
///         Spacer::new(),
///     ]);
/// }
/// ```
/// This will result in the correct layout:
/// | = = a = = b = = c = = |
///
/// You can even do more funky things that are not always easily accessible in other layout models,
/// like:
/// ```
/// use carbide_core::widget::{HStack, Rectangle, WidgetExt, Spacer};
///
/// fn main() {
///     HStack::new((
///         Spacer::new(),
///         Spacer::new(),
///         Rectangle::new().frame(10.0, 10.0),
///         Spacer::new(),
///         Rectangle::new().frame(10.0, 10.0),
///         Spacer::new(),
///     ));
/// }
/// ```
/// That will result in the following layout:
/// | = = = = b = = c = = |
/// In the above, the spacers each take up two ´=´s
///
/// ## The cross axis direction
/// The cross axis for the HStack is the height.
/// Stacks are very simple in how they layout the widgets on the cross axis.
/// To calculate the stack's height, it takes the max height of the containing widgets.
///
/// There are three different options for aligning widgets on the cross-axis.
/// The default is *Center*, but *Start* and *End* also exist.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct HStack<W> where W: Sequence
{
    #[id] id: WidgetId,
    children: W,
    position: Position,
    dimension: Dimension,
    spacing: Scalar,
    cross_axis_alignment: CrossAxisAlignment,
}

impl<W: Sequence> HStack<W> {

    #[carbide_default_builder2]
    pub fn new(children: W) -> Self {
        HStack {
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

/*#[cfg(feature = "macro")]
gen_optionals!(
    HStack,
    spacing: f64,
);*/

impl<W: Sequence> Layout for HStack<W> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let spacing = self.spacing;
        calculate_size_hstack(self, spacing, requested_size, ctx);
        self.dimension
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let spacing = self.spacing;
        let cross_axis_alignment = self.cross_axis_alignment;
        position_children_hstack(self, spacing, cross_axis_alignment, ctx)
    }
}

impl<W: Sequence> CommonWidget for HStack<W> {
    CommonWidgetImpl!(self, child: self.children, position: self.position, dimension: self.dimension, flexibility: 1);
}