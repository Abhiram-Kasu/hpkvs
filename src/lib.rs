use std::{collections::HashMap, hash::Hash, sync::Arc};

use tokio::sync::Mutex;
#[derive(Clone, Debug)]
pub struct KVStore<K, V> {
    map: HashMap<K, V>,
}

impl<K, V> KVStore<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            map: Default::default(),
        }
    }

    pub fn add_item(&mut self, k: K, value: V) {
        self.map.insert(k, value);
    }

    pub fn read_item(&self, k: K) -> Option<&V> {
        self.map.get(&k)
    }
}
