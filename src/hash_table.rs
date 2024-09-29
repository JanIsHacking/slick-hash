// Copyright (C) 2023 Gerd Augsburg
//
// This file contains code originally written by Gerd Augsburg. Please contact Gerd Augsburg
// for permissions and terms.

pub trait CompleteHashTable:
HashTableBase<u64, u64> + HashTableBulk<u64, u64> + Named + MaybeRemovable<u64, u64>
{
}

impl<T> CompleteHashTable for T where
    T: HashTableBase<u64, u64> + HashTableBulk<u64, u64> + Named + MaybeRemovable<u64, u64>
{
}

pub trait Capacity: Copy {
    fn capacity(self) -> usize;
}

impl Capacity for usize {
    #[inline(always)]
    fn capacity(self) -> usize {
        self
    }
}
#[derive(Copy, Clone)]
pub struct WithMargin(pub usize, pub f64);

impl Capacity for WithMargin {
    #[inline(always)]
    fn capacity(self) -> usize {
        let WithMargin(capacity, epsilon) = self;
        ((1.0 + epsilon) * capacity as f64) as usize
    }
}

pub enum Insertion<'t, V> {
    Inserted(&'t mut V),
    Occupied(&'t mut V),
}
impl<V> Insertion<'_, V> {
    pub fn is_inserted(&self) -> bool {
        match self {
            Insertion::Inserted(_) => true,
            Insertion::Occupied(_) => false,
        }
    }
}

impl<'t, V> AsMut<V> for Insertion<'t, V> {
    fn as_mut(&mut self) -> &mut V {
        match self {
            Insertion::Inserted(v) => v,
            Insertion::Occupied(v) => v,
        }
    }
}

impl<'t, V> AsRef<V> for Insertion<'t, V> {
    fn as_ref(&self) -> &V {
        match self {
            Insertion::Inserted(v) => v,
            Insertion::Occupied(v) => v,
        }
    }
}

pub trait HashTableBase<Key, Value> {
    fn with_capacity(capacity: impl Capacity) -> Self;
    fn try_insert(&mut self, key_value_pair: (Key, Value)) -> Insertion<Value>;
    fn get(&self, key: &Key) -> Option<&Value>;
    fn contains(&self, key: &Key) -> bool {
        self.get(key).is_some()
    }
}

pub trait HashTableRemove<Key, Value> {
    fn remove_entry(&mut self, key: &Key) -> Option<(Key, Value)>;
}

pub trait HashTableBulk<Key, Value> {
    fn bulk_insert(&mut self, key_value_pairs: &[(Key, Value)]);
}

pub trait DefaultHashTableBuild {}

impl<Key, Value, T> HashTableBulk<Key, Value> for T
where
    Key: Copy,
    Value: Copy,
    T: HashTableBase<Key, Value> + DefaultHashTableBuild,
{
    fn bulk_insert(&mut self, key_value_pairs: &[(Key, Value)]) {
        for pair in key_value_pairs.iter().copied() {
            self.try_insert(pair);
        }
    }
}

pub trait MaybeRemovable<Key, Value> {
    const SUPPORTS_REMOVE: bool = false;
    #[allow(unused_variables)]
    fn remove_entry(&mut self, key: &Key) -> Option<(Key, Value)> {
        unimplemented!("remove of entry not implemented")
    }
}

impl<Key, Value, T> MaybeRemovable<Key, Value> for T
where
    T: HashTableRemove<Key, Value>,
{
    const SUPPORTS_REMOVE: bool = true;
    fn remove_entry(&mut self, key: &Key) -> Option<(Key, Value)> {
        HashTableRemove::remove_entry(self, key)
    }
}

pub trait Named {
    fn name() -> String;
}

pub mod std_map {
    use std::collections::{hash_map::Entry, HashMap};

    use super::*;

    impl Named for HashMap<u64, u64> {
        fn name() -> String {
            "std::collection::HashMap".into()
        }
    }

    impl HashTableBase<u64, u64> for HashMap<u64, u64> {
        fn with_capacity(capacity: impl Capacity) -> Self {
            HashMap::with_capacity(capacity.capacity())
        }

        fn try_insert(&mut self, key_value_pair: (u64, u64)) -> Insertion<u64> {
            let (key, value) = key_value_pair;
            match self.entry(key) {
                Entry::Occupied(occ) => Insertion::Occupied(occ.into_mut()),
                Entry::Vacant(vac) => Insertion::Inserted(vac.insert(value)),
            }
        }

        fn get(&self, key: &u64) -> Option<&u64> {
            self.get(key)
        }

        fn contains(&self, key: &u64) -> bool {
            self.contains_key(key)
        }
    }

    impl HashTableRemove<u64, u64> for HashMap<u64, u64> {
        fn remove_entry(&mut self, key: &u64) -> Option<(u64, u64)> {
            self.remove_entry(key)
        }
    }

    impl HashTableBulk<u64, u64> for HashMap<u64, u64> {
        fn bulk_insert(&mut self, key_value_pairs: &[(u64, u64)]) {
            self.extend(key_value_pairs.iter().copied());
        }
    }
}

pub mod std_btree {
    use std::collections::btree_map::Entry;
    use std::collections::BTreeMap;

    use super::*;

    impl Named for BTreeMap<u64, u64> {
        fn name() -> String {
            "std::collection::BTreeMap".into()
        }
    }

    impl HashTableBase<u64, u64> for BTreeMap<u64, u64> {
        fn with_capacity(_capacity: impl Capacity) -> Self {
            BTreeMap::new()
        }

        fn try_insert(&mut self, key_value_pair: (u64, u64)) -> Insertion<u64> {
            let (key, value) = key_value_pair;
            match self.entry(key) {
                Entry::Occupied(occ) => Insertion::Occupied(occ.into_mut()),
                Entry::Vacant(vac) => Insertion::Inserted(vac.insert(value)),
            }
        }

        fn get(&self, key: &u64) -> Option<&u64> {
            self.get(key)
        }

        fn contains(&self, key: &u64) -> bool {
            self.contains_key(key)
        }
    }

    impl HashTableRemove<u64, u64> for BTreeMap<u64, u64> {
        fn remove_entry(&mut self, key: &u64) -> Option<(u64, u64)> {
            self.remove_entry(key)
        }
    }

    impl HashTableBulk<u64, u64> for BTreeMap<u64, u64> {
        fn bulk_insert(&mut self, key_value_pairs: &[(u64, u64)]) {
            self.extend(key_value_pairs.iter().copied());
        }
    }
}
