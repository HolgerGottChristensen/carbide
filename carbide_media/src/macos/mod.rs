mod video;

use std::cell::OnceCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::time::Duration;
use bitflags::bitflags;
use cocoa::base::id;
use cocoa::foundation::NSString;
use objc::{class, msg_send, sel, sel_impl};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
pub use video::*;


// The below is manually extracted from the API: https://developer.apple.com/documentation/corevideo/cvpixelbuffer?language=objc
// and from running the following command when inside the AVFoundation header directory:
// bindgen --objc-extern-crate --generate-block AVFoundation.h  -o bindings.rs -- -F/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/System/Library/Frameworks -x objective-c

#[link(name = "AVFoundation", kind = "framework")]
extern "C" {
    pub static AVAudioSessionCategoryPlayback: id;
    pub fn CVPixelBufferGetWidth(theString: id) -> usize;
    pub fn CVPixelBufferGetBytesPerRow(theString: id) -> usize;
    pub fn CVPixelBufferGetPixelFormatType(theString: id) -> [u8; 4];
    pub fn CVPixelBufferGetHeight(theString: id) -> usize;
    pub fn CVPixelBufferGetDataSize(theString: id) -> usize;
    pub fn CVPixelBufferGetBaseAddress(theString: id) -> *mut u8;
    pub fn CVPixelBufferLockBaseAddress(theString: id, flags: u64) -> i32;
    pub fn CVPixelBufferUnlockBaseAddress(theString: id, flags: u64) -> i32;
    pub static mut kCVPixelBufferPixelFormatTypeKey: id;
}

#[link(name = "CoreMedia", kind = "framework")]
extern "C" {
    pub fn CMTimeGetSeconds(time: CMTime) -> f64;
    pub fn CMTimeMakeWithSeconds(seconds: f64, preferredTimescale: i32) -> CMTime;

    pub static kCMTimeIndefinite: CMTime;
}

extern "C" {
    pub static NSKeyValueChangeKindKey: id;
    pub static NSKeyValueChangeNotificationIsPriorKey: id;
    pub static NSKeyValueChangeNewKey: id;
    pub static NSKeyValueChangeOldKey: id;
    pub static NSKeyValueChangeIndexesKey: id;
}

pub type CMTimeEpoch = i64;
pub type CMTimeScale = i32;
pub type CMTimeValue = i64;

#[derive(Debug, PartialEq, Hash, Eq, Copy, Clone)]
pub enum NSKeyValueChangeKey {
    // https://developer.apple.com/documentation/foundation/nskeyvaluechangeindexeskey?language=objc
    Indexes,
    // https://developer.apple.com/documentation/foundation/nskeyvaluechangekindkey?language=objc
    Kind,
    // https://developer.apple.com/documentation/foundation/nskeyvaluechangenewkey?language=objc
    New,
    // https://developer.apple.com/documentation/foundation/nskeyvaluechangenotificationispriorkey?language=objc
    NotificationIsPrior,
    // https://developer.apple.com/documentation/foundation/nskeyvaluechangeoldkey?language=objc
    Old,
}

impl From<id> for NSKeyValueChangeKey {
    fn from(value: id) -> Self {
        unsafe {
            if value == NSKeyValueChangeIndexesKey {
                NSKeyValueChangeKey::Indexes
            } else if value == NSKeyValueChangeKindKey {
                NSKeyValueChangeKey::Kind
            } else if value == NSKeyValueChangeNewKey {
                NSKeyValueChangeKey::New
            } else if value == NSKeyValueChangeNotificationIsPriorKey {
                NSKeyValueChangeKey::NotificationIsPrior
            } else if value == NSKeyValueChangeOldKey {
                NSKeyValueChangeKey::Old
            } else {
                unreachable!("You have compared with an id that is not a NSKeyValueChange")
            }
        }
    }
}

bitflags! {
    struct NSKeyValueObservingOptions: u64 {
        const NSKeyValueObservingOptionNew = 1;
        const NSKeyValueObservingOptionOld = 2;
        const NSKeyValueObservingOptionInitial = 4;
        const NSKeyValueObservingOptionPrior = 8;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct CMTimeFlags: u32 {
        const Valid = 1;
        const HasBeenRounded = 2;
        const PositiveInfinity = 4;
        const NegativeInfinity = 8;
        const Indefinite = 16;
        const ImpliedValueFlagsMask = 28;
    }
}

#[repr(C, packed(4))]
#[derive(Debug, Copy, Clone)]
pub struct CMTime {
    /// The value of the CMTime. value/timescale = seconds
    pub value: CMTimeValue,
    /// The timescale of the CMTime. value/timescale = seconds.
    pub timescale: CMTimeScale,
    /// The flags, eg. kCMTimeFlags_Valid, kCMTimeFlags_PositiveInfinity, etc.
    pub flags: CMTimeFlags,
    /// Differentiates between equal timestamps that are actually different because
    /// of looping, multi-item sequencing, etc.
    /// Will be used during comparison: greater epochs happen after lesser ones.
    /// Additions/subtraction is only possible within a single epoch,
    /// however, since epoch length may be unknown/variable
    pub epoch: CMTimeEpoch,
}

impl From<CMTime> for Duration {
    fn from(value: CMTime) -> Self {
        let seconds = unsafe { CMTimeGetSeconds(value) };
        Duration::from_secs_f64(seconds)
    }
}

impl From<Duration> for CMTime {
    fn from(value: Duration) -> Self {
        unsafe { CMTimeMakeWithSeconds(value.as_secs_f64(), 1000) }
    }
}


fn listener<F: Fn(String, HashMap<NSKeyValueChangeKey, id>) + 'static>(f: F) -> id {

    struct ListenerWrapper(Box<dyn Fn(String, HashMap<NSKeyValueChangeKey, id>) + 'static>);

    let cell = OnceCell::new();
    let LISTENER_CLASS = cell.get_or_init(|| unsafe {

        let super_class = class!(NSObject);
        let mut listener_declaration = ClassDecl::new("CarbideListener", super_class).unwrap();

        // Add a field to the listener class, pointing to
        listener_declaration.add_ivar::<*const c_void>("callback");

        listener_declaration.add_class_method(sel!(new), new as extern "C" fn(&Class, Sel) -> id);

        listener_declaration.add_method(sel!(setFunctionPointer:), set_function_pointer as extern "C" fn(&mut Object, Sel, *const c_void));
        listener_declaration.add_method(
            sel!(observeValueForKeyPath:ofObject:change:context:),
            changed as extern "C" fn(&mut Object, Sel, id, id, id, id)
        );
        listener_declaration.add_method(sel!(dealloc), dealloc as extern "C" fn(&Object, Sel));

        extern "C" fn new(class: &Class, _: Sel) -> id {
            unsafe {
                let this: id = msg_send![class, alloc];
                let this: id = msg_send![this, init];
                this
            }
        }

        extern "C" fn changed(this: &mut Object, _: Sel, string: id, object: id, change: id, context: id) {
            let string = unsafe {
                let slice = std::slice::from_raw_parts(string.UTF8String() as *const _, string.len());
                let result = std::str::from_utf8_unchecked(slice);
                result.to_string()
            };

            unsafe {
                let changes: u64 = msg_send![change, count];
                let mut keys: Vec<id> = Vec::with_capacity(changes as usize);
                let mut objs: Vec<id> = Vec::with_capacity(changes as usize);

                unsafe {
                    let _: () = msg_send![change, getObjects:objs.as_mut_ptr() andKeys:keys.as_mut_ptr()];
                    keys.set_len(changes as usize);
                    objs.set_len(changes as usize);
                }

                let callback: *mut c_void = *this.get_ivar("callback");
                let f = std::mem::transmute::<*mut c_void, *mut ListenerWrapper>(callback);

                let mut map = HashMap::new();

                for (key, obj) in keys.into_iter().zip(objs) {
                    let key = NSKeyValueChangeKey::from(key);

                    map.insert(key, obj);
                }

                // Call the function with the string and the map.
                ((*f).0)(string.clone(), map)
            }


            //println!("Changed called: {}", string)
        }

        extern "C" fn set_function_pointer(this: &mut Object, _: Sel, f: *const c_void) {
            unsafe {
                (*this).set_ivar("callback", f);
            }
        }

        extern "C" fn dealloc(_this: &Object, _: Sel) {
            println!("Dealloc called");
        }

        //println!("Initialized listener");
        listener_declaration.register()
    });

    let mut wrapper = Box::pin(ListenerWrapper(Box::new(f)));


    let listener: id = unsafe { msg_send![*LISTENER_CLASS, new] };
    let _: () = unsafe { msg_send![listener, setFunctionPointer:wrapper.as_mut().get_unchecked_mut()]};
    std::mem::forget(wrapper);

    listener
}