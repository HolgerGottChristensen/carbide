extern crate carbide_core;
#[macro_use]
extern crate carbide_derive;
#[macro_use]
extern crate carbide_macro;
extern crate copypasta;
extern crate unicode_segmentation;

pub use list::List;
pub use plain::PlainButton;
pub use plain::PlainPopUpButton;
pub use plain::PlainTextInput;

mod plain;
mod list;

#[test]
fn test1() {
    body!(
        Spacer(hejsa: 42.0)
    );
}