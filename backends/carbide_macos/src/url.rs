use std::path::{Path, PathBuf};
use cocoa::base::{id, nil};
use cocoa::foundation::{NSAutoreleasePool, NSURL as InnerNSURL};
use objc::runtime::YES;
use crate::id::Id;
use crate::string::NSString;

pub struct NSURL(pub id);

impl From<PathBuf> for NSURL {
    fn from(p: PathBuf) -> Self {

        let string = p.to_str().expect("Could not convert pathbuf to &str");
        let ns_string: NSString = string.to_string().into();

        let id = unsafe {
            InnerNSURL::alloc(nil)
                .initFileURLWithPath_isDirectory_(
                    ns_string.id(),
                    YES,
                )
                .autorelease()
        };

        NSURL(id)
    }
}

impl From<&Path> for NSURL {
    fn from(p: &Path) -> Self {

        let string = p.to_str().expect("Could not convert pathbuf to &str");
        let ns_string = NSString::from(string);

        let id = unsafe {
            InnerNSURL::alloc(nil)
                .initFileURLWithPath_isDirectory_(
                    ns_string.id(),
                    YES,
                )
                .autorelease()
        };

        NSURL(id)
    }
}


impl Id for NSURL {
    fn id(&self) -> id {
        self.0
    }
}