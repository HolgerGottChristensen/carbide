use carbide::accessibility::{Accessibility, AccessibilityContext};
use carbide::CommonWidgetImpl;
use carbide::draw::{Alignment, Dimension, Position};
use carbide::environment::Environment;
use carbide::event::{AccessibilityEvent, AccessibilityEventContext, AccessibilityEventHandler, ApplicationEvent, ApplicationEventContext, ApplicationEventHandler, EventHandler, KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseEvent, MouseEventContext, MouseEventHandler, OtherEvent, OtherEventContext, OtherEventHandler, WindowEvent, WindowEventContext, WindowEventHandler};
use carbide::focus::{FocusContext, Focusable};
use carbide::identifiable::Identifiable;
use carbide::layout::{Layout, LayoutContext};
use carbide::lifecycle::{InitializationContext, Initialize, Update, UpdateContext};
use carbide::render::{Render, RenderContext};
use carbide::state::{LocalState, ReadState, State};
use carbide::widget::{AnySequence, AnyWidget, CommonWidget, Widget, WidgetExt, WidgetId, WidgetProperties};
use carbide::widget::properties::WidgetKindSimple;

#[derive(Clone, Debug)]
pub struct WidgetViewer {
    id: WidgetId,
    position: Position,
    dimension: Dimension,

    child: LocalState<Box<dyn AnyWidget>>,
}

impl WidgetViewer {
    pub fn new(child: LocalState<Box<dyn AnyWidget>>) -> WidgetViewer {
        WidgetViewer {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            child,
        }
    }
}

impl CommonWidget for WidgetViewer {
    CommonWidgetImpl!(self, position: self.position, dimension: self.dimension);

    fn foreach_child(& self, f: &mut dyn FnMut(& dyn AnyWidget)) {
        unimplemented!()
    }

    fn foreach_child_mut(& mut self, f: &mut dyn FnMut(& mut dyn AnyWidget)) {
        unimplemented!()
    }

    fn foreach_child_rev(& mut self, f: &mut dyn FnMut(& mut dyn AnyWidget)) {
        unimplemented!()
    }

    fn foreach_child_direct(& mut self, f: &mut dyn FnMut(& mut dyn AnyWidget)) {
        unimplemented!()
    }

    fn foreach_child_direct_rev(& mut self, f: &mut dyn FnMut(& mut dyn AnyWidget)) {
        unimplemented!()
    }

    fn child(&self, index: usize) -> &dyn AnyWidget {
        unimplemented!()
    }

    fn child_mut(&mut self, index: usize) -> &mut dyn AnyWidget {
        unimplemented!()
    }

    fn child_count(&mut self) -> usize {
        unimplemented!()
    }
}

impl Initialize for WidgetViewer {
    fn process_initialization(&mut self, ctx: &mut InitializationContext) {

    }
}

impl KeyboardEventHandler for WidgetViewer {
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {

    }
}

impl Accessibility for WidgetViewer {
    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {

    }
}

impl Identifiable for WidgetViewer {
    type Id = WidgetId;

    fn id(&self) -> Self::Id { self.id }
}

impl AccessibilityEventHandler for WidgetViewer {
    fn process_accessibility_event(&mut self, event: &AccessibilityEvent, ctx: &mut AccessibilityEventContext) {

    }
}

impl Render for WidgetViewer {
    fn render(&mut self, ctx: &mut RenderContext) {
        self.child.value_mut().render(ctx);
    }
}

impl MouseEventHandler for WidgetViewer {
    fn process_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {

    }
}

impl WindowEventHandler for WidgetViewer {
    fn process_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {

    }
}

impl Layout for WidgetViewer {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.child.value_mut().calculate_size(requested_size, ctx);
        self.dimension = requested_size;
        requested_size
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let inner_dimension = self.child.value().dimension();
        self.child.value_mut().set_position(Alignment::Center.position(self.position, self.dimension, inner_dimension));
        self.child.value_mut().position_children(ctx);
    }
}

impl carbide::widget::WidgetSync for WidgetViewer {
    fn sync(&mut self, env: &mut Environment) {

    }
}

impl Focusable for WidgetViewer {
    fn process_focus_next(&mut self, ctx: &mut FocusContext) {

    }

    fn process_focus_previous(&mut self, ctx: &mut FocusContext) {

    }

    fn process_focus_request(&mut self, ctx: &mut FocusContext) {

    }
}

impl Update for WidgetViewer {
    fn process_update(&mut self, ctx: &mut UpdateContext) {

    }
}

impl OtherEventHandler for WidgetViewer {
    fn process_other_event(&mut self, event: &OtherEvent, ctx: &mut OtherEventContext) {

    }
}

impl ApplicationEventHandler for WidgetViewer {
    fn process_application_event(&mut self, event: &ApplicationEvent, ctx: &mut ApplicationEventContext) {
        
    }
}

impl AnyWidget for WidgetViewer {
    fn as_widget(&self) -> &dyn AnyWidget { self }
    fn as_widget_mut(&mut self) -> &mut dyn AnyWidget { self }
}

impl WidgetProperties for WidgetViewer { type Kind = WidgetKindSimple; }

impl WidgetExt for WidgetViewer {}