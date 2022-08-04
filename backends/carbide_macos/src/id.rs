use cocoa::base::id;

pub trait Id {
    fn id(&self) -> id;
}