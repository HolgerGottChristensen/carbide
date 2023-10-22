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

impl<W: Widget + Clone + 'static> WidgetSequence for W {
    fn foreach<'a>(&'a self, f: &mut dyn FnMut(&'a dyn Widget)) {
        if self.is_ignore() {
            return;
        }

        if self.is_proxy() {
            self.foreach_child(f);
            return;
        }

        f(self);
    }

    fn foreach_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        if self.is_ignore() {
            return;
        }

        if self.is_proxy() {
            self.foreach_child_mut(f);
            return;
        }

        f(self);
    }

    fn foreach_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        if self.is_ignore() {
            return;
        }

        if self.is_proxy() {
            self.foreach_child_rev(f);
            return;
        }

        f(self);
    }

    fn foreach_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        f(self)
    }

    fn foreach_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        f(self)
    }
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