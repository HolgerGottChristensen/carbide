use std::marker::PhantomData;
use carbide::environment::EnvironmentKey;
use carbide::event::{AccessibilityEvent, AccessibilityEventContext, OtherEvent, KeyboardEvent, KeyboardEventContext, MouseEvent, MouseEventContext, OtherEventContext, WindowEvent, WindowEventContext, WindowEventHandler};
use carbide::lifecycle::InitializationContext;
use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::environment::{Environment};
use crate::event::{AccessibilityEventHandler, KeyboardEventHandler, MouseEventHandler, OtherEventHandler};
use crate::lifecycle::{Initialize, Update, UpdateContext};
use crate::widget::{AnyWidget, CommonWidget, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone)]
pub struct NavigationManager {
    stack: Vec<StackItem>
}

impl NavigationManager {
    pub fn push(&mut self, value: Box<dyn AnyWidget>) {
        self.stack.push(StackItem::Other(value));
    }

    pub fn extend(&mut self, iter: impl IntoIterator<Item=Box<dyn AnyWidget>>) {
        self.stack.extend(iter.into_iter().map(|item| StackItem::Other(item)));
    }

    pub fn pop(&mut self) -> bool {
        if self.stack.len() > 1 {
            self.stack.pop();
            true
        } else {
            false
        }
    }

    pub fn pop_all(&mut self) {
        while self.pop() {}
    }

    pub fn pop_n(&mut self, n: usize) {
        for _ in 0..n {
            self.pop();
        }
    }

    pub fn replace(&mut self, value: Box<dyn AnyWidget>) {
        self.pop();
        self.push(value);
    }

    pub fn replace_all(&mut self, value: Box<dyn AnyWidget>) {
        self.stack = vec![StackItem::Other(value)]
    }
}

impl NavigationManager {
    pub fn root(env: &mut Environment, f: impl FnOnce(&mut NavigationManager)) {
        env.get_mut::<NavigationRootKey>().map(|navigation_manager| {
            f(navigation_manager)
        });
    }

    pub fn specific<K: EnvironmentKey<Value=NavigationManager>>(env: &mut Environment, f: impl FnOnce(&mut NavigationManager)) {
        env.get_mut::<K>().map(|navigation_manager| {
            f(navigation_manager)
        });
    }

    pub fn get(env: &mut Environment, f: impl FnOnce(&mut NavigationManager)) {
        env.get_mut::<NavigationKey>().map(|navigation_manager| {
            f(navigation_manager)
        });
    }
}

#[derive(Clone, Debug)]
enum StackItem {
    Current,
    Other(Box<dyn AnyWidget>)
}

#[derive(Copy, Clone, Debug)]
pub struct NavigationKey;
impl EnvironmentKey for NavigationKey {
    type Value = NavigationManager;
}

#[derive(Copy, Clone, Debug)]
struct NavigationRootKey;
impl EnvironmentKey for NavigationRootKey {
    type Value = NavigationManager;
}

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Initialize, Update, MouseEvent, KeyboardEvent, OtherEvent, WindowEvent, AccessibilityEvent)]
pub struct NavigationStack<K> where K: EnvironmentKey<Value=NavigationManager> + Clone {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,

    key: PhantomData<K>,
    navigation_manager: NavigationManager,
    current: Box<dyn AnyWidget>,
}

impl NavigationStack<NavigationKey> {
    pub fn new(initial: impl Widget) -> NavigationStack<impl EnvironmentKey<Value=NavigationManager> + Clone> {
        NavigationStack::<NavigationKey> {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            key: Default::default(),
            navigation_manager: NavigationManager {
                stack: vec![StackItem::Current],
            },
            current: initial.boxed(),
        }
    }

    pub fn new_specific<K: EnvironmentKey<Value=NavigationManager> + Clone>(initial: impl Widget) -> NavigationStack<K> {
        NavigationStack::<K> {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            key: Default::default(),
            navigation_manager: NavigationManager {
                stack: vec![StackItem::Current],
            },
            current: initial.boxed(),
        }
    }

    pub fn new_root(initial: impl Widget) -> NavigationStack<impl EnvironmentKey<Value=NavigationManager> + Clone> {
        NavigationStack::<NavigationRootKey> {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            key: Default::default(),
            navigation_manager: NavigationManager {
                stack: vec![StackItem::Current],
            },
            current: initial.boxed(),
        }
    }
}

impl<K: EnvironmentKey<Value=NavigationManager> + Clone> NavigationStack<K> {
    fn with(&mut self, env: &mut Environment, f: impl FnOnce(&mut Environment, &mut Box<dyn AnyWidget>)) {
        env.with_mut::<K>(&mut self.navigation_manager, |env| {
            f(env, &mut self.current)
        });

        // Get a reference to the last stack item in the stack
        // We expect there is always at least 1 element in the stack.
        let last = self.navigation_manager.stack.last_mut().expect("The stack should never be empty. This invariant should be kept by the methods of the manager.");

        match last {
            StackItem::Current => {
                // If the latest item is already Current, the stack is up to date
                return;
            }
            StackItem::Other(last) => {
                // replaces the last current element with the last element.
                std::mem::swap(&mut self.current, last);

                self.current.process_initialization(&mut InitializationContext {
                    env,
                })
            }
        }

        // Last is now the previously current element
        let old_current = self.navigation_manager.stack.pop().unwrap();

        // If any stack item has variant current, replace with the old_current.
        // At most 1 element could have variant current.
        let current = self.navigation_manager.stack.iter_mut().find(|a| matches!(a, StackItem::Current));
        if let Some(val) = current {
            let _ = std::mem::replace(val, old_current);
        }

        // Since the latest element is now current, add the Current element to the top of the stack.
        self.navigation_manager.stack.push(StackItem::Current);
    }
}

impl<K: EnvironmentKey<Value=NavigationManager> + Clone> Initialize for NavigationStack<K> {
    fn process_initialization(&mut self, ctx: &mut InitializationContext) {
        self.with(ctx.env, |env, child| {
            child.process_initialization(&mut InitializationContext {
                env,
            })
        })
    }
}

impl<K: EnvironmentKey<Value=NavigationManager> + Clone> Update for NavigationStack<K> {
    fn process_update(&mut self, ctx: &mut UpdateContext) {
        self.with(ctx.env, |env, child| {
            child.process_update(&mut UpdateContext {
                text: ctx.text,
                image: ctx.image,
                env,
            })
        })
    }
}

impl<K: EnvironmentKey<Value=NavigationManager> + Clone> MouseEventHandler for NavigationStack<K> {
    fn process_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        self.with(ctx.env, |env, child| {
            child.process_mouse_event(event, &mut MouseEventContext {
                text: ctx.text,
                image: ctx.image,
                is_current: ctx.is_current,
                window_id: ctx.window_id,
                consumed: ctx.consumed,
                env,
            })
        })
    }
}

impl<K: EnvironmentKey<Value=NavigationManager> + Clone> KeyboardEventHandler for NavigationStack<K> {
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        self.with(ctx.env, |env, child| {
            child.process_keyboard_event(event, &mut KeyboardEventContext {
                text: ctx.text,
                image: ctx.image,
                env,
                is_current: ctx.is_current,
                window_id: ctx.window_id,
                prevent_default: ctx.prevent_default,
            })
        })
    }
}

impl<K: EnvironmentKey<Value=NavigationManager> + Clone> OtherEventHandler for NavigationStack<K> {
    fn process_other_event(&mut self, event: &OtherEvent, ctx: &mut OtherEventContext) {
        self.with(ctx.env, |env, child| {
            child.process_other_event(event, &mut OtherEventContext {
                text: ctx.text,
                image: ctx.image,
                env,
                is_current: ctx.is_current,
                is_consumed: ctx.is_consumed,
            })
        })
    }
}

impl<K: EnvironmentKey<Value=NavigationManager> + Clone> WindowEventHandler for NavigationStack<K> {
    fn process_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        self.with(ctx.env, |env, child| {
            child.process_window_event(event, &mut WindowEventContext {
                text: ctx.text,
                image: ctx.image,
                env,
                is_current: ctx.is_current,
                window_id: ctx.window_id,
            })
        })
    }
}

impl<K: EnvironmentKey<Value=NavigationManager> + Clone> AccessibilityEventHandler for NavigationStack<K> {
    fn process_accessibility_event(&mut self, event: &AccessibilityEvent, ctx: &mut AccessibilityEventContext) {
        self.with(ctx.env, |env, child| {
            child.process_accessibility_event(event, &mut AccessibilityEventContext {
                env,
            })
        })
    }
}

impl<K: EnvironmentKey<Value=NavigationManager> + Clone> CommonWidget for NavigationStack<K> {
    CommonWidgetImpl!(self, child: self.current, position: self.position, dimension: self.dimension);
}