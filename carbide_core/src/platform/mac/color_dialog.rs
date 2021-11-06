use objc::runtime::Class;

pub struct ColorPickerResponder(pub *const Class);

unsafe impl Send for ColorPickerResponder {}

unsafe impl Sync for ColorPickerResponder {}

pub static COLOR_PICKER_RESPONDER: ColorPickerResponder = unsafe {};