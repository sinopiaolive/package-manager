use std::borrow::Borrow;

use im::ordmap::{Iter, OrdMap as Map};

pub trait Mappable
where
    Self: Sized,
{
    type K: Clone + Ord;
    type V: Clone;

    fn as_map(&self) -> &Map<Self::K, Self::V>;
    fn wrap(m: Map<Self::K, Self::V>) -> Self;

    fn insert(&self, key: Self::K, value: Self::V) -> Self {
        Self::wrap(self.as_map().update(key, value))
    }

    fn uncons(&self, key: &Self::K) -> Option<(Self::V, Self)> {
        match self.as_map().extract(key) {
            None => None,
            Some((v, map)) => Some((v, Self::wrap(map))),
        }
    }

    fn get<BK>(&self, key: &BK) -> Option<&Self::V>
    where
        BK: Ord + ?Sized,
        Self::K: Borrow<BK>,
    {
        self.as_map().get(key)
    }

    fn contains_key<BK>(&self, key: &BK) -> bool
    where
        BK: Ord + ?Sized,
        Self::K: Borrow<BK>,
    {
        self.as_map().contains_key(key)
    }

    fn iter(&self) -> Iter<'_, (Self::K, Self::V)> {
        self.as_map().iter()
    }

    fn is_empty(&self) -> bool {
        self.as_map().is_empty()
    }

    fn len(&self) -> usize {
        self.as_map().len()
    }

    fn get_min(&self) -> Option<&(Self::K, Self::V)> {
        self.as_map().get_min()
    }
}
