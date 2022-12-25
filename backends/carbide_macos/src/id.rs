use cocoa::base::id;
use objc::{msg_send, sel, sel_impl};

use crate::NSString;

pub trait Id {
    fn id(&self) -> id;

    fn print_description(&self) {
        let id = self.id();
        let desc = NSString(unsafe {msg_send![id, description]});
        println!("{}", String::from(desc));
    }
}

impl Id for id {
    fn id(&self) -> id {
        *self
    }
}