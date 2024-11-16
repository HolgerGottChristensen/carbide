use std::fmt::Debug;
use indexmap::IndexMap;
use indexmap::map::Keys;
use carbide::state::{AnyReadState, AnyState, StateContract};
use carbide::widget::{AnyWidget, BuildWidgetIdHasher, Widget, WidgetExt, WidgetId, WidgetSequence};
use crate::identifiable::Identifiable;

pub trait AnyIdentifiableWidget<T>: AnyWidget + Identifiable<T> where T: StateContract + PartialEq {
    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>));
    fn as_widget(&self) -> Box<dyn AnyWidget>;
}

impl<T: StateContract + PartialEq, W> AnyIdentifiableWidget<T> for W where W: AnyWidget + Identifiable<T> + Clone {
    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>)) {
        todo!()
    }

    fn as_widget(&self) -> Box<dyn AnyWidget> {
        Box::new(self.clone())
    }
}

dyn_clone::clone_trait_object!(<T> AnyIdentifiableWidget<T>);

pub trait IdentifiableWidget<T>: AnyWidget + Identifiable<T> + WidgetExt + Clone where T: StateContract + PartialEq  {
    fn has_changed_child(&self, existing: &[WidgetId]) -> bool;
}

impl<T: StateContract + PartialEq, W> IdentifiableWidget<T> for W where W: AnyWidget + Identifiable<T> + WidgetExt + Clone {
    fn has_changed_child(&self, existing: &[WidgetId]) -> bool {
        let mut changed = false;

        /*self.foreach_child(&mut |child| {
            if !changed {
                if let Some(id) = existing.next() {
                    changed = changed | (child.id() == id)
                } else {
                    changed = true;
                }
            }
        });*/

        changed
    }
}

pub enum ExistingOrNew<T> where T: StateContract + PartialEq {
    New(Box<dyn AnyIdentifiableWidget<T>>),
    Existing(WidgetId)
}

pub trait IdentifiableWidgetSequence<T>: WidgetSequence where T: StateContract + PartialEq {
    fn has_changed(&self, existing: &mut Keys<'_, WidgetId, Box<dyn AnyWidget>>) -> bool;
    fn update(&self, f: &mut dyn FnMut(Box<dyn AnyWidget>, Box<dyn AnyReadState<T=T>>));
}

impl<S: IdentifiableWidget<T>, T: StateContract + PartialEq> IdentifiableWidgetSequence<T> for Vec<S> {
    fn has_changed(&self, existing: &mut Keys<'_, WidgetId, Box<dyn AnyWidget>>) -> bool {
        for element in self {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                /*if element.has_changed_child(existing) {
                    return true;
                }
                continue;*/
                todo!()
            }

            if let Some(id) = existing.next() {
                if *id != element.id() {
                    // The element is different.
                    return true;
                }
            } else {
                // More values in sequence than existing.
                return true;
            }
        }

        // Return true if any elements were left after the iteration of all children
        existing.next().is_some()
    }

    fn update(&self, f: &mut dyn FnMut(Box<dyn AnyWidget>, Box<dyn AnyReadState<T=T>>)) {
        for element in self {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                //element.foreach_child_mut(f);
                //continue;
                todo!()
            }

            f(element.as_widget(), element.identifier());
        }
    }
}

macro_rules! tuple_identifiable_impl {
    ($($generic:ident),*) => {
        #[allow(non_snake_case)]
        #[allow(unused_parens)]
        impl<T: StateContract + PartialEq, $($generic: IdentifiableWidget<T>),*> IdentifiableWidgetSequence<T> for ($($generic),*) {

        }
    };
}

//impl<S: IdentifiableWidget<T>, T: StateContract + PartialEq> IdentifiableWidgetSequence<T> for S {}


//tuple_identifiable_impl!(W1);
//tuple_identifiable_impl!(W1, W2);
//tuple_identifiable_impl!(W1, W2, W3);
//tuple_identifiable_impl!(W1, W2, W3, W4);
//tuple_identifiable_impl!(W1, W2, W3, W4, W5);
//tuple_identifiable_impl!(W1, W2, W3, W4, W5, W6);
//tuple_identifiable_impl!(W1, W2, W3, W4, W5, W6, W7);
//tuple_identifiable_impl!(W1, W2, W3, W4, W5, W6, W7, W8);
//tuple_identifiable_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9);
//tuple_identifiable_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9, W10);
//tuple_identifiable_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9, W10, W11);
//tuple_identifiable_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9, W10, W11, W12);