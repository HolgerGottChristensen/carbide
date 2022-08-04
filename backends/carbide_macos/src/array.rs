use cocoa::base::{id, nil};
use cocoa::foundation::NSArray as InnerNSArray;
use crate::id::Id;

pub struct NSArray {
    inner: id,
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
}

impl Id for NSArray {
    fn id(&self) -> id {
        self.inner
    }
}