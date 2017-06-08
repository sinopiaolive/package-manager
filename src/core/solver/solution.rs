use std::sync::Arc;
use std::iter::{FromIterator, IntoIterator};
use std::convert::From;
use manifest::PackageName;
use version::Version;
use solver::path::Path;
use immutable_map::map::TreeMap as Map;
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
pub struct PartialSolution(Map<Arc<PackageName>, JustifiedVersion>);

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
pub struct Solution(Map<Arc<PackageName>, Arc<Version>>);

impl Mappable for Solution {
    type K = Arc<PackageName>;
    type V = Arc<Version>;

    fn as_map(&self) -> &Map<Self::K, Self::V> {
        &self.0
    }

    fn wrap(m: Map<Self::K, Self::V>) -> Self {
        Solution(m)
    }
}

impl FromIterator<(Arc<PackageName>, Arc<Version>)> for Solution {
    fn from_iter<T>(iter: T) -> Solution
        where T: IntoIterator<Item = (Arc<PackageName>, Arc<Version>)>
    {
        Solution(Map::<Arc<PackageName>, Arc<Version>>::from_iter(iter))
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

// impl fmt::Debug for Solution {
//     fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
//         write!(f, "Solution( ")?;
//         match self {
//             &Solution::Solution(ref m) => {
//                 for (k, v) in m.into_iter() {
//                     write!(f, "{}: {}", k, v)?;
//                 }
//             }
//         }
//         write!(f, ")")
//     }
// }
