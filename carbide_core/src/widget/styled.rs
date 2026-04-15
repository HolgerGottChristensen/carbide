use std::any::{type_name, TypeId};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::rc::Rc;
use carbide::environment::Environment;
use carbide::widget::properties::WidgetKindSimple;
use carbide::widget::WidgetProperties;
use crate::draw::{Dimension, Position};
use crate::environment::{EnvironmentColorAccent, EnvironmentKey, EnvironmentKeyDefault};
use crate::widget::{AnyWidget, CommonWidget, Empty, IntoWidget, Widget, WidgetExt, WidgetId, WidgetStyle, WidgetSync};
use crate::CommonWidgetImpl;
use crate::widget::scroll::style::HorizontalScrollBarStyleKey;

#[derive(Clone, Widget)]
#[carbide_exclude(Sync)]
pub struct Styled<S, W>
where
    S: EnvironmentKey + EnvironmentKeyDefault + Clone,
    W: Widget,
    S::Value: WidgetStyle + Clone
{
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,
    original: W,
    child: Box<dyn AnyWidget>,
    style_id: TypeId,
    f: Rc<dyn Fn(&S::Value, W) -> Box<dyn AnyWidget> + 'static>,
    phantom_data: PhantomData<S>,
}

impl<
    S: EnvironmentKey + EnvironmentKeyDefault + Clone,
    W: Widget + WidgetProperties<Kind=WidgetKindSimple>
> Styled<S, W> where S::Value: WidgetStyle + Clone {
    pub fn new(child: W, f: impl Fn(&S::Value, W) -> Box<dyn AnyWidget> + Clone + 'static) -> Styled<S, W>  {

        Styled {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            original: child.clone(),
            child: child.boxed(),
            style_id: TypeId::of::<()>(),
            f: Rc::new(f),
            phantom_data: Default::default(),
        }
    }
}

impl<
    S: EnvironmentKey + EnvironmentKeyDefault + Clone,
    W: Widget
> WidgetSync for Styled<S, W> where S::Value: WidgetStyle + Clone {
    fn sync(&mut self, env: &mut Environment) {
        if let Some(style) = env.get::<S>() {
            if style.key() != self.style_id {
                self.child = (self.f)(style, self.original.clone());
                self.style_id = style.key();
            }
        } else {
            let style = &S::default();
            if style.key() != self.style_id {
                self.child = (self.f)(style, self.original.clone());
                self.style_id = style.key();
            }
        }
    }
}

impl<
    S: EnvironmentKey + EnvironmentKeyDefault + Clone,
    W: Widget,
> CommonWidget for Styled<S, W> where S::Value: WidgetStyle + Clone {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension);
}

impl<
    S: EnvironmentKey + EnvironmentKeyDefault + Clone,
    W: Widget
> Debug for Styled<S, W> where S::Value: WidgetStyle + Clone {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Styled")
            .field("child", &self.child)
            .field("key", &type_name::<S>())
            .finish_non_exhaustive()
    }
}