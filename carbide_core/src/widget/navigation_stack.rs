use std::marker::PhantomData;
use carbide::environment::Key;
use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::environment::{EnvironmentStack, WidgetTransferAction};
use crate::lifecycle::{Update, UpdateContext};
use crate::widget::{AnyWidget, CommonWidget, Widget, WidgetExt, WidgetId};

#[derive(Debug)]
pub struct NavigationManager {
    stack: Vec<StackItem>
}

#[derive(Clone, Debug)]
enum StackItem {
    Current,
    Other(Box<dyn AnyWidget>)
}

#[derive(Copy, Clone, Debug)]
struct NavigationKey;
impl Key for NavigationKey {
    type Value = NavigationManager;
}

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Update)]
pub struct NavigationStack<K> where K: Key {
    id: WidgetId,
    position: Position,
    dimension: Dimension,

    key: PhantomData<K>,

    stack: Vec<StackItem>,
    current: Box<dyn AnyWidget>,
}

impl<K: Key> NavigationStack<K> {
    pub fn new(initial: impl Widget) -> NavigationStack<K> {
        NavigationStack {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            key: Default::default(),
            stack: vec![],
            current: initial.boxed(),
        }
    }

    fn with(&mut self, env_stack: &mut EnvironmentStack, f: impl FnOnce(&mut EnvironmentStack, &mut Box<dyn Widget>)) {
        let manager = NavigationManager {
            stack: &mut self.stack,
        };

        env_stack.with::<K>(&manager, f);

        // Fix up the stack
        println!("Stack: {:#?}", self.stack)
    }
}

impl<K: Key> Update for NavigationStack<K> {
    fn process_update(&mut self, ctx: &mut UpdateContext) {
        self.with(ctx.env_stack, |env_stack, child| {
            child.process_update(&mut UpdateContext {
                text: ctx.text,
                image: ctx.image,
                env: ctx.env,
                env_stack,
            })
        })
    }

    /*fn update(&mut self, ctx: &mut UpdateContext) {
        // Take out the transferred widget with the key if it exists
        if let Some(action) = ctx.env.transferred_widget(&self.transfer_id) {
            match action {
                WidgetTransferAction::Push(mut widget) => {
                    let top = &mut self.top;
                    std::mem::swap(top, &mut widget);
                    self.stack.push(widget)
                }
                WidgetTransferAction::Pop => {
                    if let Some(new_top) = self.stack.pop() {
                        self.top = new_top
                    }
                }
                WidgetTransferAction::Replace(widget) => self.top = widget,
                WidgetTransferAction::PushVec(vec) => {
                    for mut widget in vec {
                        let top = &mut self.top;
                        std::mem::swap(top, &mut widget);
                        self.stack.push(widget)
                    }
                }
                WidgetTransferAction::PopN(n) => {
                    for _ in 0..n {
                        if let Some(new_top) = self.stack.pop() {
                            self.top = new_top
                        }
                    }
                }
                WidgetTransferAction::PopAll => {
                    if self.stack.len() > 0 {
                        self.top = self.stack.remove(0);
                        self.stack = vec![];
                    }
                }
                WidgetTransferAction::ReplaceAll(widget) => {
                    self.stack = vec![];
                    self.top = widget
                }
            }
        }
    }*/
}

impl<K: Key> CommonWidget for NavigationStack<K> {
    CommonWidgetImpl!(self, id: self.id, child: self.current, position: self.position, dimension: self.dimension);
}