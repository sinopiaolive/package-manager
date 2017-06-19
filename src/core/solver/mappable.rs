use immutable_map::map::{TreeMap, TreeMapIter};

pub trait Mappable where Self: Sized {
    type K: Clone + Ord;
    type V: Clone;

    fn as_map(&self) -> &TreeMap<Self::K, Self::V>;
    fn wrap(m: TreeMap<Self::K, Self::V>) -> Self;

    fn insert(&self, key: Self::K, value: Self::V) -> Self {
        Self::wrap(self.as_map().insert(key, value))
    }

    fn remove(&self, key: &Self::K) -> Option<(Self, &Self::V)> {
        match self.as_map().remove(key) {
            None => None,
            Some((map, v)) => Some((Self::wrap(map), v))
        }
    }

    fn get<'a>(&'a self, key: &'a Self::K) -> Option<&'a Self::V> {
        self.as_map().get(key)
    }

    fn contains_key(&self, key: &Self::K) -> bool {
        self.as_map().contains_key(key)
    }

    fn iter<'r>(&'r self) -> TreeMapIter<'r, Self::K, Self::V> {
        self.as_map().iter()
    }

    fn is_empty(&self) -> bool {
        self.as_map().is_empty()
    }

    fn len(&self) -> usize {
        self.as_map().len()
    }
}
