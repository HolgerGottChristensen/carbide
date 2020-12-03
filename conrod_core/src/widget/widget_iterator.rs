use widget::primitive::Widget;
use std::slice::IterMut;
use flags::Flags;

pub enum WidgetIter<'a> {
    Empty,
    Single(&'a mut Box<dyn Widget>, Box<WidgetIter<'a>>),
    Multi(IterMut<'a, Box<dyn Widget>>, Box<WidgetIter<'a>>)
}

impl<'a> Iterator for WidgetIter<'a> {
    type Item = &'a mut Box<dyn Widget>;

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