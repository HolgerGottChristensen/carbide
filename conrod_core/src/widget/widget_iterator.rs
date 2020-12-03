use widget::primitive::Widget;
use std::slice::{IterMut, Iter};
use flags::Flags;

pub enum WidgetIterMut<'a> {
    Empty,
    Single(&'a mut Box<dyn Widget>, Box<WidgetIterMut<'a>>),
    Multi(Box<WidgetIterMut<'a>>, Box<WidgetIterMut<'a>>)
}

impl<'a> WidgetIterMut<'a> {
    pub fn single(widget: &'a mut Box<dyn Widget>) -> WidgetIterMut<'a> {
        WidgetIterMut::Single(widget, Box::new(WidgetIterMut::Empty))
    }
}

impl<'a> Iterator for WidgetIterMut<'a> {
    type Item = &'a mut Box<dyn Widget>;

    fn next(&mut self) -> Option<Self::Item> {

        let mut i = WidgetIterMut::Empty;

        std::mem::swap(self, &mut i);

        match i {
            WidgetIterMut::Empty => {
                None
            }
            WidgetIterMut::Single(n, mut b) => {
                std::mem::swap(self, &mut *b);
                Some(n)
            }
            WidgetIterMut::Multi(mut iter, mut b) => {
                match iter.next() {
                    Some(n) => {
                        std::mem::swap(self, &mut WidgetIterMut::Multi(iter, b));
                        Some(n)
                    }
                    None => {
                        std::mem::swap(self, &mut *b);
                        self.next()
                    }
                }
            }
        }
    }
}

pub enum WidgetIter<'a> {
    Empty,
    Single(&'a Box<dyn Widget>, Box<WidgetIter<'a>>),
    Multi(Box<WidgetIter<'a>>, Box<WidgetIter<'a>>)
}

impl<'a> WidgetIter<'a> {
    pub fn single(widget: &'a Box<dyn Widget>) -> WidgetIter<'a> {
        WidgetIter::Single(widget, Box::new(WidgetIter::Empty))
    }
}

impl<'a> Iterator for WidgetIter<'a> {
    type Item = &'a Box<dyn Widget>;

    fn next(&mut self) -> Option<Self::Item> {

        let mut i = WidgetIter::Empty;

        std::mem::swap(self, &mut i);

        match i {
            WidgetIter::Empty => {
                None
            }
            WidgetIter::Single(n, mut b) => {
                std::mem::swap(self, &mut *b);
                Some(n)
            }
            WidgetIter::Multi(mut iter, mut b) => {
                match iter.next() {
                    Some(n) => {
                        std::mem::swap(self, &mut WidgetIter::Multi(iter, b));
                        Some(n)
                    }
                    None => {
                        std::mem::swap(self, &mut *b);
                        self.next()
                    }
                }
            }
        }
    }
}