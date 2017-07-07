use std::sync::Arc;
use std::iter::{FromIterator, IntoIterator};
use std::convert::From;
use pm_lib::manifest::PackageName;
use pm_lib::version::Version;
use solver::path::Path;
use im::map::Map;
use solver::mappable::Mappable;


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JustifiedVersion {
    pub version: Arc<Version>,
    pub path: Path,
}

impl JustifiedVersion {
    pub fn new(version: Arc<Version>, path: Path) -> JustifiedVersion {
        JustifiedVersion {
            version: version.clone(),
            path: path.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartialSolution(pub Map<PackageName, JustifiedVersion>);

impl PartialSolution {
    pub fn new() -> PartialSolution {
        PartialSolution(Map::new())
    }
}

impl Mappable for PartialSolution {
    type K = PackageName;
    type V = JustifiedVersion;

    fn as_map(&self) -> &Map<Self::K, Self::V> {
        &self.0
    }

    fn wrap(m: Map<Self::K, Self::V>) -> Self {
        PartialSolution(m)
    }
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Solution(pub Map<PackageName, Version>);

impl Mappable for Solution {
    type K = PackageName;
    type V = Version;

    fn as_map(&self) -> &Map<Self::K, Self::V> {
        &self.0
    }

    fn wrap(m: Map<Self::K, Self::V>) -> Self {
        Solution(m)
    }
}

impl FromIterator<(Arc<PackageName>, Arc<Version>)> for Solution {
    fn from_iter<T>(iter: T) -> Solution
    where
        T: IntoIterator<Item = (Arc<PackageName>, Arc<Version>)>,
    {
        Solution(Map::<PackageName, Version>::from_iter(iter))
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
