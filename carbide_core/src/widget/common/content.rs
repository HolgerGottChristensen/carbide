use indexmap::IndexMap;
use carbide::widget::{BuildWidgetIdHasher, Widget, WidgetId};

#[derive(Clone, Debug)]
pub struct Content<O: Widget>(pub IndexMap<WidgetId, O, BuildWidgetIdHasher>, pub usize);

impl<O: Widget> Default for Content<O> {
    fn default() -> Self {
        Content(Default::default(), 0)
    }
}