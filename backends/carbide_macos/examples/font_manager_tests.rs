use cocoa::base::id;
use cocoa::foundation::NSInteger;
use objc::msg_send;
use objc::sel;
use objc::sel_impl;
use objc::class;
use carbide_macos::NSString;


fn main() {
    unsafe {
        let manager: id = msg_send![class!(NSFontManager), sharedFontManager];
        let fonts: id = msg_send![manager, availableFonts];

        let count: NSInteger = msg_send![fonts, count];

        let mut paths = vec![];

        for index in 0..count {
            let url: id = msg_send![fonts, objectAtIndex: index];
            let path = NSString(url);
            let path: String = path.into();
            paths.push(path);
        }

        dbg!(paths);
    }
}