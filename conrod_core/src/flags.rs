

bitflags! {
    pub struct Flags: u32 {
        const Empty =  0b00000000;
        const Proxy =  0b00000001;
        const Spacer = 0b00000010;
    }
}
