use crate::event::{Key, ModifierKey};

#[derive(Debug, Copy, Clone)]
pub struct HotKey {
    pub key: Key,
    pub modifier: ModifierKey,
}

impl HotKey {
    pub fn new(key: Key, modifier: ModifierKey) -> Self {
        HotKey {
            key,
            modifier
        }
    }
}
