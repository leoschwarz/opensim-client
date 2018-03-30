pub mod bimap {
    use std::collections::HashMap;
    use std::hash::Hash;

    pub struct BiMap<K, V> {
        fwd: HashMap<K, V>,
        bwd: HashMap<V, K>,
    }

    impl<K: Clone + Hash + Eq, V: Clone + Hash + Eq> BiMap<K, V> {
        /// Create a new instance.
        pub fn new() -> Self {
            BiMap {
                fwd: HashMap::new(),
                bwd: HashMap::new(),
            }
        }

        pub fn clear(&mut self) {
            self.fwd.clear();
            self.bwd.clear();
        }

        pub fn len(&self) -> usize {
            self.fwd.len()
        }

        pub fn insert(&mut self, key: K, value: V) {
            self.fwd.insert(key.clone(), value.clone());
            self.bwd.insert(value, key);
        }

        pub fn remove_key(&mut self, key: &K) -> Option<V> {
            if let Some(v) = self.fwd.remove(key) {
                self.bwd.remove(&v).unwrap();
                Some(v)
            } else {
                None
            }
        }

        pub fn remove_val(&mut self, val: &V) -> Option<K> {
            if let Some(k) = self.bwd.remove(val) {
                self.fwd.remove(&k).unwrap();
                Some(k)
            } else {
                None
            }
        }

        pub fn contains_key(&self, key: &K) -> bool {
            self.fwd.contains_key(key)
        }

        pub fn contains_val(&self, val: &V) -> bool {
            self.bwd.contains_key(val)
        }

        pub fn get_val(&self, key: &K) -> Option<&V> {
            self.fwd.get(key)
        }

        pub fn get_key(&self, val: &V) -> Option<&K> {
            self.bwd.get(val)
        }
    }
}
