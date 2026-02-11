use std::{collections::HashMap, fmt::Display, hash::Hash, sync::Arc};

use tokio::sync::Mutex;
#[derive(Clone, Debug)]
pub struct KVStore<K, V> {
    map: HashMap<K, V>,
}

impl<K, V> KVStore<K, V>
where
    K: Hash + Eq + Clone + Display + std::fmt::Debug,
    V: Clone + Display + std::fmt::Debug,
{
    pub fn new() -> Self {
        Self {
            map: Default::default(),
        }
    }

    pub fn add_item(&mut self, k: K, value: V) {
        println!("Inserting value: {value}");
        self.map.insert(k, value);

        println!("Map now contains: {:#?}", self.map);
    }

    pub fn read_item(&self, k: K) -> Option<&V> {
        println!("Getting value for key: {k}");
        println!("Map now contains: {:#?}", self.map);
        self.map.get(&k)
    }
}
