use carbide::state::StateContract;
use carbide::widget::{WidgetId, Sequence, AnyWidget};
use crate::identifiable::{AnyIdentifiableWidget, IdentifiableWidget};

impl<T: PartialEq + StateContract, S: IdentifiableWidget<T>> Sequence<dyn AnyIdentifiableWidget<T>> for Vec<S> {
    fn foreach<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyIdentifiableWidget<T>)) {
        for element in self {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                AnyIdentifiableWidget::foreach_child(element, f);
                continue;
            }

            f(element);
        }
    }

    fn foreach_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>)) {
        for element in self {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                AnyIdentifiableWidget::foreach_child_mut(element, f);
                continue;
            }

            f(element);
        }
    }

    fn foreach_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>)) {
        for element in &mut self.iter_mut().rev() {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                AnyIdentifiableWidget::foreach_child_rev(element, f);
                continue;
            }

            f(element);
        }
    }

    fn foreach_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>)) {
        for element in &mut self.iter_mut() {
            f(element);
        }
    }

    fn foreach_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyIdentifiableWidget<T>)) {
        for element in &mut self.iter_mut().rev() {
            f(element);
        }
    }
}



/*pub trait IdentifiableWidgetSequence<T>: Sequence
where T: StateContract + PartialEq {
    fn has_changed(&self, existing: &mut dyn Iterator<Item=WidgetId>) -> bool;
    fn update(&self, f: &mut dyn FnMut(&dyn AnyIdentifiableWidget<T>));
}

impl<S: IdentifiableWidget<T>, T: StateContract + PartialEq> IdentifiableWidgetSequence<T> for Vec<S> {
    fn has_changed(&self, existing: &mut dyn Iterator<Item=WidgetId>) -> bool {
        for element in self {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                if element.has_changed_child(existing) {
                    return true;
                }
            }

            if let Some(id) = existing.next() {
                if id != element.id() {
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

    fn update(&self, f: &mut dyn FnMut(&dyn AnyIdentifiableWidget<T>)) {
        for element in self {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                element.foreach_identifiable_child(&mut |child| {
                    f(child);
                });
                continue;
            }

            f(element);
        }
    }
}*/

macro_rules! tuple_identifiable_impl {
    ($($generic:ident),*) => {
        #[allow(non_snake_case)]
        #[allow(unused_parens)]
        impl<T: StateContract + PartialEq, $($generic: IdentifiableWidget<T>),*> IdentifiableWidgetSequence<T> for ($($generic),*) {
            fn has_changed(&self, existing: &mut dyn Iterator<Item=WidgetId>) -> bool {
                let ($($generic),*) = self;
                $(
                    if $generic.is_ignore() {

                    } else if $generic.is_proxy() {
                        if $generic.has_changed_child(existing) {
                            return true;
                        }
                    } else {
                        if let Some(id) = existing.next() {
                            if id != $generic.identifier() {
                                // The element is different.
                                return true;
                            }
                        } else {
                            // More values in sequence than existing.
                            return true;
                        }
                    }
                )*

                // Return true if any elements were left after the iteration of all children
                existing.next().is_some()
            }

            fn update(&self, f: &mut dyn FnMut(&dyn AnyIdentifiableWidget<T>)) {
                let ($($generic),*) = self;
                $(
                    if $generic.is_ignore() {

                    } else if $generic.is_proxy() {
                        $generic.foreach_identifiable_child(&mut |child| {
                            f(child);
                        });
                    } else {
                        f($generic);
                    }
                )*
            }
        }
    };
}

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