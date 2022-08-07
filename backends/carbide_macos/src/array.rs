use cocoa::base::{id, nil};
use cocoa::foundation::{NSArray as InnerNSArray, NSInteger};
use crate::id::Id;
use objc::{msg_send, class, sel, sel_impl};

pub struct NSArray {
    pub inner: id,
}

impl NSArray {
    pub fn new(items: &[id]) -> NSArray {
        let id = unsafe {
            InnerNSArray::arrayWithObjects(nil, items)
        };

        NSArray {
            inner: id
        }
    }

    pub fn len(&self) -> usize {
        unsafe {
            let count: NSInteger = msg_send![self.inner, count];
            count as usize
        }
    }

    pub fn at(&self, index: usize) -> id {
        unsafe {
            let item: id = msg_send![self.inner, objectAtIndex: index as NSInteger];
            item
        }
    }
}

impl Id for NSArray {
    fn id(&self) -> id {
        self.inner
    }
}