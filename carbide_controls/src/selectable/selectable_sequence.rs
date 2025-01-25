use dyn_clone::DynClone;
use crate::identifiable::{AnySelectableWidget, SelectableWidget};
use carbide::reverse;
use carbide::widget::{AnySequence, Content, Identifiable, Sequence, WidgetSync};
use carbide::widget::foreach_widget::{Delegate, ForEachWidget};

impl<S: SelectableWidget> AnySequence<dyn AnySelectableWidget> for Vec<S> {
    fn foreach<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnySelectableWidget)) {
        for element in self {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                AnySelectableWidget::foreach_child(element, f);
                continue;
            }

            f(element);
        }
    }

    fn foreach_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
        for element in self {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                AnySelectableWidget::foreach_child_mut(element, f);
                continue;
            }

            f(element);
        }
    }

    fn foreach_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
        for element in &mut self.iter_mut().rev() {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                AnySelectableWidget::foreach_child_rev(element, f);
                continue;
            }

            f(element);
        }
    }

    fn foreach_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
        for element in &mut self.iter_mut() {
            f(element);
        }
    }

    fn foreach_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
        for element in &mut self.iter_mut().rev() {
            f(element);
        }
    }
}

impl<W: SelectableWidget> AnySequence<dyn AnySelectableWidget> for Content<W> {
    fn foreach<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnySelectableWidget)) {
        for (_, element) in self.0.iter().take(self.1) {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                AnySelectableWidget::foreach_child(element, f);
                continue;
            }

            f(element);
        }
    }

    fn foreach_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
        for (_, element) in self.0.iter_mut().take(self.1) {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                AnySelectableWidget::foreach_child_mut(element, f);
                continue;
            }

            f(element);
        }
    }

    fn foreach_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
        for (_, element) in self.0.iter_mut().take(self.1).rev() {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                AnySelectableWidget::foreach_child_rev(element, f);
                continue;
            }

            f(element);
        }
    }

    fn foreach_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
        for (_, element) in self.0.iter_mut().take(self.1) {
            f(element);
        }
    }

    fn foreach_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
        for (_, element) in self.0.iter_mut().take(self.1).rev() {
            f(element);
        }
    }
}

macro_rules! tuple_sequence_impl {
    ($($generic:ident),*) => {
        #[allow(non_snake_case)]
        #[allow(unused_parens)]
        impl<$($generic: SelectableWidget),*> AnySequence<dyn AnySelectableWidget> for ($($generic),*) {
            fn foreach<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnySelectableWidget)) {
                let ($($generic),*) = self;
                $(
                    if $generic.is_ignore() {

                    } else if $generic.is_proxy() {
                        AnySelectableWidget::foreach_child($generic, f);
                    } else {
                        f($generic);
                    }
                )*
            }

            fn foreach_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
                let ($($generic),*) = self;
                $(
                    if $generic.is_ignore() {

                    } else if $generic.is_proxy() {
                        AnySelectableWidget::foreach_child_mut($generic, f);
                    } else {
                        f($generic);
                    }
                )*
            }

            fn foreach_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
                let reverse!([$($generic)*]) = self;
                $(
                    if $generic.is_ignore() {

                    } else if $generic.is_proxy() {
                        AnySelectableWidget::foreach_child_rev($generic, f);
                    } else {
                        f($generic);
                    }
                )*
            }

            fn foreach_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
                let ($($generic),*) = self;
                $(
                    f($generic);
                )*
            }

            fn foreach_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
                let reverse!([$($generic)*]) = self;
                $(
                    f($generic);
                )*
            }
        }
    };
}

#[allow(non_snake_case)]
#[allow(unused_parens)]
impl<
    T: ?Sized + Identifiable + WidgetSync + DynClone + 'static,
    W: Sequence<T>,
    O: SelectableWidget,
    D: Delegate<T, O>
> AnySequence<dyn AnySelectableWidget> for ForEachWidget<W, O, D, T> {
    fn foreach<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnySelectableWidget)) {
        AnySelectableWidget::foreach_child(self, f);
    }

    fn foreach_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
        AnySelectableWidget::foreach_child_mut(self, f);
    }

    fn foreach_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
        AnySelectableWidget::foreach_child_rev(self, f);
    }

    fn foreach_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
        f(self);
    }

    fn foreach_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnySelectableWidget)) {
        f(self);
    }
}

//tuple_sequence_impl!(W1);
tuple_sequence_impl!(W1, W2);
tuple_sequence_impl!(W1, W2, W3);
tuple_sequence_impl!(W1, W2, W3, W4);
tuple_sequence_impl!(W1, W2, W3, W4, W5);
tuple_sequence_impl!(W1, W2, W3, W4, W5, W6);
tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7);
tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8);
tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9);
tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9, W10);
tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9, W10, W11);
tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9, W10, W11, W12);