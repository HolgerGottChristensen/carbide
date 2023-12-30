use bitflags::bitflags;


bitflags! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct WidgetFlag: u8 {
        const EMPTY =  0b00000000;
        const PROXY =  0b00000001;
        const SPACER = 0b00000010;
        const FOCUSABLE = 0b00000100;
        const IGNORE = 0b00001000;
        const USEMAXCROSSAXIS = 0b00010000;
    }
}
