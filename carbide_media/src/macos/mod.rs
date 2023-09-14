mod video;

use cocoa::base::id;
pub use video::*;


// The below is manually extracted from the API: https://developer.apple.com/documentation/corevideo/cvpixelbuffer?language=objc
// and from running the following command when inside the AVFoundation header directory:
// bindgen --objc-extern-crate --generate-block AVFoundation.h  -o bindings.rs -- -F/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/System/Library/Frameworks -x objective-c

#[link(name = "AVFoundation", kind = "framework")]
extern {
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

pub type CMTimeFlags = u32;
pub type CMTimeEpoch = i64;
pub type CMTimeScale = i32;
pub type CMTimeValue = i64;


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