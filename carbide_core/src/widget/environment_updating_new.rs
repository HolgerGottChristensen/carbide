use crate::accessibility::Accessibility;
use crate::environment::Key;
use crate::render::Render;
use crate::widget::{CommonWidget, Widget};
use carbide::render::RenderContext;
use carbide::ModifierWidgetImpl;
use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct EnvUpdatingNew<C, K> where C: Widget, K: Key {
    child: C,
    key: PhantomData<K>,
    value: K::Value,
}

impl<C: Widget, K: Key> EnvUpdatingNew<C, K> {
    pub fn new(value: K::Value, child: C) -> EnvUpdatingNew<C, K> {
        EnvUpdatingNew {
            child,
            key: PhantomData::default(),
            value
        }
    }
}

impl<C: Widget, K: Key> Render for EnvUpdatingNew<C, K> {
    fn render(&mut self, context: &mut RenderContext) {
        context.env_new.with::<K>(&self.value, |ctx| {
            self.child.render(&mut RenderContext {
                render: context.render,
                text: context.text,
                image: context.image,
                env: context.env,
                env_new: ctx,
            })
        })
    }
}

impl<C: Widget, K: Key> CommonWidget for EnvUpdatingNew<C, K> {
    ModifierWidgetImpl!(self, child: self.child);
}