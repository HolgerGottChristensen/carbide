use cocoa::base::{id, nil};
use cocoa::foundation::{NSAutoreleasePool, NSString as InnerNSString};
use crate::id::Id;

pub struct NSString(pub id);

impl From<String> for NSString {
    fn from(s: String) -> Self {
        NSString::from(s.as_str())
    }
}

impl From<&str> for NSString {
    fn from(s: &str) -> Self {
        let id = unsafe { InnerNSString::alloc(nil).init_str(s).autorelease() };
        NSString(id)
    }
}

impl From<NSString> for String {
    fn from(s: NSString) -> Self {
        unsafe {
            let slice = std::slice::from_raw_parts(s.id().UTF8String() as *const _, s.id().len());
            let result = std::str::from_utf8_unchecked(slice);
            result.into()
        }
    }
}

impl Id for NSString {
    fn id(&self) -> id {
        self.0
    }
}