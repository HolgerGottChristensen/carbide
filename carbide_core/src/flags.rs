use bitflags::bitflags;

bitflags! {
    pub struct Flags: u32 {
        const EMPTY =  0b00000000;
        const PROXY =  0b00000001;
        const SPACER = 0b00000010;
        const FOCUSABLE = 0b00000100;
    }
}
