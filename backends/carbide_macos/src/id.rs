use cocoa::base::id;
use crate::NSString;
use objc::{msg_send, class, sel, sel_impl};

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