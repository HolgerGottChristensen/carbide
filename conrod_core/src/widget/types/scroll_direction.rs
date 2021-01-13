

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ScrollDirection {
    Vertical,
    Horizontal,
    Both,
    NoScroll,
}