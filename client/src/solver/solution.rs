use im::OrdMap as Map;
use pm_lib::package::PackageName;
use pm_lib::version::Version;
use solver::mappable::Mappable;
use solver::path::Path;
use std::collections::BTreeMap;
use std::convert::From;
use std::iter::{FromIterator, IntoIterator};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JustifiedVersion {
    pub version: Arc<Version>,
    pub path: Path,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartialSolution(pub Map<Arc<PackageName>, JustifiedVersion>);

impl PartialSolution {
    pub fn new() -> PartialSolution {
        PartialSolution(Map::new())
    }
}

impl Mappable for PartialSolution {
    type K = Arc<PackageName>;
    type V = JustifiedVersion;

    fn as_map(&self) -> &Map<Self::K, Self::V> {
        &self.0
    }

    fn wrap(m: Map<Self::K, Self::V>) -> Self {
        PartialSolution(m)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Solution(pub BTreeMap<PackageName, Version>);

impl FromIterator<(Arc<PackageName>, Arc<Version>)> for Solution {
    fn from_iter<T>(iter: T) -> Solution
    where
        T: IntoIterator<Item = (Arc<PackageName>, Arc<Version>)>,
    {
        let iter_of_owned = iter.into_iter().map(|(package_name, version)| {
            let pn = Arc::try_unwrap(package_name).unwrap_or_else(|p| (*p).clone());
            let ver = Arc::try_unwrap(version).unwrap_or_else(|v| (*v).clone());
            (pn, ver)
        });
        Solution(BTreeMap::<PackageName, Version>::from_iter(iter_of_owned)
    }
}

impl From<PartialSolution> for Solution {
    fn from(partial_solution: PartialSolution) -> Solution {
        // Strip all paths from a PartialSolution to obtain a Solution
        partial_solution
            .iter()
            .map(|(package_name, justified_version)| {
                (package_name.clone(), justified_version.version.clone())
            })
            .collect()
    }
}
