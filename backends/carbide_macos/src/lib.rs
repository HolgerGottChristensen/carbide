mod string;
mod url;
mod id;
mod dialogs;
mod array;
mod menu;
mod menu_item;

use cocoa::base::nil;
use cocoa::foundation::NSProcessInfo;
pub use dialogs::*;
pub use menu::*;
pub use menu_item::*;

/*pub fn process_name() -> id {
    unsafe { NSProcessInfo::processInfo(nil).processName() }
}*/