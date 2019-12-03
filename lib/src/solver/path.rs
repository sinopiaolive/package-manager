use crate::package::PackageName;
use crate::version::Version;
use std::fmt;
use std::ops::Index;
use std::slice::Iter;
use std::sync::Arc;

/// A dependency chain of packages.
pub struct Path(Arc<Vec<(Arc<PackageName>, Arc<Version>)>>);

impl Path {
    pub fn new() -> Path {
        Path(Arc::new(Vec::new()))
    }

    pub fn from_vec(vec: Vec<(Arc<PackageName>, Arc<Version>)>) -> Path {
        Path(Arc::new(vec))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn last(&self) -> Option<&(Arc<PackageName>, Arc<Version>)> {
        self.0.last()
    }

    pub fn push(&self, item: (Arc<PackageName>, Arc<Version>)) -> Path {
        let mut vec = (*self.0).clone();
        vec.push(item);
        Path(Arc::new(vec))
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, (Arc<PackageName>, Arc<Version>)> {
        self.0.iter()
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Path {
    fn clone(&self) -> Path {
        Path(self.0.clone())
    }
}

impl Index<usize> for Path {
    type Output = (Arc<PackageName>, Arc<Version>);

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl fmt::Debug for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}

impl PartialEq<Path> for Path {
    fn eq(&self, other: &Path) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for Path {}
