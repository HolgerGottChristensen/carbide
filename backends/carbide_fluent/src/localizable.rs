use std::fmt::Debug;

pub trait Localizable: Debug + Clone + 'static {
    fn get(&self) -> &str;
}

impl Localizable for &'static str {
    fn get(&self) -> &str {
        *self
    }
}
