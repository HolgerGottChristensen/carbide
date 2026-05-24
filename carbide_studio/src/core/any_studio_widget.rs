use std::ops::Deref;
use std::rc::Rc;
use carbide::render::Style;
use carbide::state::ReadState;
use carbide::widget::{AnyWidget, Rectangle, VStack, WidgetExt};

pub trait AnyStudioWidget: AnyWidget {
    fn editor(&self) -> Box<dyn AnyWidget>;
}

dyn_clone::clone_trait_object!(AnyStudioWidget);

impl AnyStudioWidget for Rc<dyn AnyStudioWidget> {
    fn editor(&self) -> Box<dyn AnyWidget> {
        self.deref().editor()
    }
}

impl AnyWidget for Box<dyn AnyStudioWidget> {
    fn as_widget(&self) -> &dyn AnyWidget {
        self
    }

    fn as_widget_mut(&mut self) -> &mut dyn AnyWidget {
        self
    }
}