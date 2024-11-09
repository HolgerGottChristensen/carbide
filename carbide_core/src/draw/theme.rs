use crate::environment::Key;

#[derive(Copy, Clone, Debug)]
pub enum Theme {
    Light,
    Dark
}

impl Key for Theme {
    type Value = Theme;
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dark
    }
}
