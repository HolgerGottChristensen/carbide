use carbide_derive::StateValue;

#[derive(Copy, Clone, Debug, PartialEq, StateValue)]
pub enum FontStyle {
    Normal,
    Italic,
}
