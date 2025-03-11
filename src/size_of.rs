use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use uuid::Uuid;

pub trait SizeOf {
    fn size_of(&self) -> usize;
}

impl SizeOf for i32 {
    fn size_of(&self) -> usize {
        0
    }
}

impl SizeOf for Uuid {
    fn size_of(&self) -> usize {
        0
    }
}

impl SizeOf for String {
    fn size_of(&self) -> usize {
        size_of::<String>() + self.capacity()
    }
}

impl<T> SizeOf for Vec<T>
where
    T: SizeOf,
{
    fn size_of(&self) -> usize {
        let vec_size = size_of::<Vec<T>>();
        let elements_size: usize = self.iter().map(|v| v.size_of()).sum();

        vec_size + elements_size
    }
}
impl<T, V> SizeOf for BTreeMap<T, V>
where
    T: SizeOf,
    V: SizeOf,
{
    fn size_of(&self) -> usize {
        let btree_map_size = size_of::<BTreeMap<T, V>>();
        let elements_size: usize = self.iter().map(|(k, v)| k.size_of() + v.size_of()).sum();

        btree_map_size + elements_size
    }
}

impl<T> SizeOf for BTreeSet<T>
where
    T: SizeOf,
{
    fn size_of(&self) -> usize {
        let btree_map_size = size_of::<BTreeSet<T>>();
        let elements_size: usize = self.iter().map(|v| v.size_of()).sum();

        btree_map_size + elements_size
    }
}

impl<T> SizeOf for Arc<T>
where
    T: SizeOf,
{
    fn size_of(&self) -> usize {
        size_of::<Arc<T>>() + self.as_ref().size_of()
    }
}
