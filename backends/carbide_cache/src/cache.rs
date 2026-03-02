use std::collections::HashMap;
use std::hash::Hash;
use carbide_core::application::ApplicationManager;

pub struct Cache<K, V> where K: Eq + Hash {
    inner: HashMap<K, CacheEntry<V>>
}

pub struct CacheEntry<V> {
    value: V,
    frame_last_used: u32,
    frame_retention: u32
}

impl<K: Eq + Hash, V> Cache<K, V> {
    pub fn new() -> Cache<K, V> {
        Cache {
            inner: HashMap::new(),
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.inner.get_mut(key).map(|entry| {
            entry.frame_last_used = ApplicationManager::application_frame();
            &entry.value
        })
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.inner.get_mut(key).map(|entry| {
            entry.frame_last_used = ApplicationManager::application_frame();
            &mut entry.value
        })
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, CacheEntry {
            value,
            frame_last_used: ApplicationManager::application_frame(),
            frame_retention: 180, // TODO: Allow inserting with different retentions based on "cache" in the rendering context
        }).map(|entry| { entry.value })
    }

    pub fn sweep(&mut self) {
        let current_frame = ApplicationManager::application_frame();
        self.inner.retain(|a, v| current_frame - v.frame_last_used < v.frame_retention);
    }
}