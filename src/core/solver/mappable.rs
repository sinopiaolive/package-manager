use std::sync::Arc;
use im::map::{Map, Iter};

pub trait Mappable
where
    Self: Sized,
{
    type K: Clone + Ord;
    type V: Clone;

    fn as_map(&self) -> &Map<Self::K, Self::V>;
    fn wrap(m: Map<Self::K, Self::V>) -> Self;

    fn insert<RK, RV>(&self, key: RK, value: RV) -> Self
    where
        Arc<Self::K>: From<RK>,
        Arc<Self::V>: From<RV>,
    {
        Self::wrap(self.as_map().insert(key, value))
    }

    fn uncons(&self, key: &Self::K) -> Option<(Arc<Self::V>, Self)> {
        match self.as_map().pop(key) {
            None => None,
            Some((v, map)) => Some((v, Self::wrap(map))),
        }
    }

    fn get(&self, key: &Self::K) -> Option<Arc<Self::V>> {
        self.as_map().get(key)
    }

    fn contains_key(&self, key: &Self::K) -> bool {
        self.as_map().contains_key(key)
    }

    fn iter(&self) -> Iter<Self::K, Self::V> {
        self.as_map().iter()
    }

    fn is_empty(&self) -> bool {
        self.as_map().is_empty()
    }

    fn len(&self) -> usize {
        self.as_map().len()
    }

    fn get_min(&self) -> Option<(Arc<Self::K>, Arc<Self::V>)> {
        self.as_map().get_min()
    }
}
