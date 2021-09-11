use std::iter::Rev;
use std::slice::{Iter, IterMut};

use crate::widget::Widget;

pub enum WidgetIterMut<'a> {
    Empty,
    Ref(&'a mut Box<dyn Widget>),
    Vec(IterMut<'a, Box<dyn Widget>>),
    VecRev(Rev<IterMut<'a, Box<dyn Widget>>>),
    Single(&'a mut Box<dyn Widget>, Box<WidgetIterMut<'a>>),
    Multi(Box<WidgetIterMut<'a>>, Box<WidgetIterMut<'a>>),
}

impl<'a> WidgetIterMut<'a> {
    pub fn single(widget: &'a mut Box<dyn Widget>) -> WidgetIterMut<'a> {
        WidgetIterMut::Ref(widget)
    }
}

impl<'a> Iterator for WidgetIterMut<'a> {
    type Item = &'a mut Box<dyn Widget>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut i = WidgetIterMut::Empty;

        std::mem::swap(self, &mut i);

        match i {
            WidgetIterMut::Empty => None,
            WidgetIterMut::Single(n, mut b) => {
                std::mem::swap(self, &mut *b);
                Some(n)
            }
            WidgetIterMut::Multi(mut iter, mut b) => match iter.next() {
                Some(n) => {
                    std::mem::swap(self, &mut WidgetIterMut::Multi(iter, b));
                    Some(n)
                }
                None => {
                    std::mem::swap(self, &mut *b);
                    self.next()
                }
            },
            WidgetIterMut::Vec(mut vec) => {
                let h = vec.next();
                std::mem::swap(self, &mut WidgetIterMut::Vec(vec));
                h
            }
            WidgetIterMut::Ref(w) => {
                Some(w)
            }
            WidgetIterMut::VecRev(mut vec) => {
                let h = vec.next();
                std::mem::swap(self, &mut WidgetIterMut::VecRev(vec));
                h
            }
        }
    }
}

pub enum WidgetIter<'a> {
    Empty,
    Ref(&'a Box<dyn Widget>),
    Vec(Iter<'a, Box<dyn Widget>>),
    VecRev(Rev<Iter<'a, Box<dyn Widget>>>),
    Single(&'a Box<dyn Widget>, Box<WidgetIter<'a>>),
    Multi(Box<WidgetIter<'a>>, Box<WidgetIter<'a>>),
}

impl<'a> WidgetIter<'a> {
    pub fn single(widget: &'a Box<dyn Widget>) -> WidgetIter<'a> {
        WidgetIter::Ref(widget)
    }
}

impl<'a> Iterator for WidgetIter<'a> {
    type Item = &'a Box<dyn Widget>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut i = WidgetIter::Empty;

        std::mem::swap(self, &mut i);

        match i {
            WidgetIter::Empty => None,
            WidgetIter::Single(n, mut b) => {
                std::mem::swap(self, &mut *b);
                Some(n)
            }
            WidgetIter::Multi(mut iter, mut b) => match iter.next() {
                Some(n) => {
                    std::mem::swap(self, &mut WidgetIter::Multi(iter, b));
                    Some(n)
                }
                None => {
                    std::mem::swap(self, &mut *b);
                    self.next()
                }
            },
            WidgetIter::Ref(w) => {
                Some(w)
            }
            WidgetIter::Vec(mut vec) => {
                let h = vec.next();
                std::mem::swap(self, &mut WidgetIter::Vec(vec));
                h
            }
            WidgetIter::VecRev(mut vec) => {
                let h = vec.next();
                std::mem::swap(self, &mut WidgetIter::VecRev(vec));
                h
            }
        }
    }
}
