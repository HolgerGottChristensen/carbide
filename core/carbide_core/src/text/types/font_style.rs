use carbide_derive::StateValue;

#[derive(Copy, Clone, Debug, PartialEq, StateValue, Eq, Hash)]
pub enum FontStyle {
    Normal,
    Italic,
}
