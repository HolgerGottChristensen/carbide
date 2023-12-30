#[derive(Clone, PartialEq, PartialOrd, Ord, Eq, Hash, Debug)]
pub enum Ime {
    PreEdit(String, Option<(usize, usize)>),
    Commit(String),
}