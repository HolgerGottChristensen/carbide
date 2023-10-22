use std::fmt::Debug;
use crate::widget::Widget;

pub trait WidgetSequence: Clone + Debug + 'static {
    fn foreach<'a>(&'a self, f: &mut dyn FnMut(&'a dyn Widget));
    fn foreach_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget));
    fn foreach_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget));
    fn foreach_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget));
    fn foreach_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget));
}

impl WidgetSequence for () {
    fn foreach<'a>(&'a self, _f: &mut dyn FnMut(&'a dyn Widget)) {}
    fn foreach_mut<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn Widget)) {}
    fn foreach_rev<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn Widget)) {}
    fn foreach_direct<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn Widget)) {}
    fn foreach_direct_rev<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn Widget)) {}
}

impl<W: Widget + Clone + 'static> WidgetSequence for Vec<W> {
    fn foreach<'a>(&'a self, f: &mut dyn FnMut(&'a dyn Widget)) {
        for element in self {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                element.foreach_child(f);
                continue;
            }

            f(element);
        }
    }

    fn foreach_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        for element in self {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                element.foreach_child_mut(f);
                continue;
            }

            f(element);
        }
    }

    fn foreach_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        for element in &mut self.iter_mut().rev() {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                element.foreach_child_rev(f);
                continue;
            }

            f(element);
        }
    }

    fn foreach_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        for element in &mut self.iter_mut() {
            f(element);
        }
    }

    fn foreach_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        for element in &mut self.iter_mut().rev() {
            f(element);
        }
    }
}

macro_rules! reverse {
    ([] $($reversed:tt)*) => {
        ($($reversed),*)  // base case
    };
    ([$first:tt $($rest:tt)*] $($reversed:tt)*) => {
        reverse!([$($rest)*] $first $($reversed)*)  // recursion
    };
}

macro_rules! tuple_sequence_impl {
    ($($generic:ident),*) => {
        #[allow(non_snake_case)]
        #[allow(unused_parens)]
        impl<$($generic: Widget + Clone + 'static),*> WidgetSequence for ($($generic),*) {
            fn foreach<'a>(&'a self, f: &mut dyn FnMut(&'a dyn Widget)) {
                let ($($generic),*) = self;
                $(
                    if $generic.is_ignore() {

                    } else if $generic.is_proxy() {
                        $generic.foreach_child(f);
                    } else {
                        f($generic);
                    }
                )*
            }

            fn foreach_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
                let ($($generic),*) = self;
                $(
                    if $generic.is_ignore() {

                    } else if $generic.is_proxy() {
                        $generic.foreach_child_mut(f);
                    } else {
                        f($generic);
                    }
                )*
            }

            fn foreach_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
                let reverse!([$($generic)*]) = self;
                $(
                    if $generic.is_ignore() {

                    } else if $generic.is_proxy() {
                        $generic.foreach_child_rev(f);
                    } else {
                        f($generic);
                    }
                )*
            }

            fn foreach_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
                let ($($generic),*) = self;
                $(
                    f($generic);
                )*
            }

            fn foreach_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
                let reverse!([$($generic)*]) = self;
                $(
                    f($generic);
                )*
            }
        }
    };
}

tuple_sequence_impl!(W1);
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