use manifest::{PackageName, DependencySet};
use solver::path::Path;
use immutable_map::map::TreeMap as Map;
use std::sync::Arc;
use version::Version;



pub type Constraint = Map<Arc<Version>, Arc<Path>>;





// impl fmt::Debug for Constraint {
//     fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
//         write!(f, "{} [{:?}]", self.sum, self.paths)
//     }
// }

pub struct ConstraintSet(Map<Arc<PackageName>, Constraint>);

impl ConstraintSet {
    pub fn pop(&self) -> (Option<(Arc<PackageName>, Constraint)>, ConstraintSet) {
        // FIXME obvs
        (None, ConstraintSet(Map::new()))
    }
    pub fn add(&self, path: &Path, facts: DependencySet) -> ConstraintSet {
        // FIXME obvs
        ConstraintSet(Map::new())
    }
}

// impl fmt::Debug for Constraints {
//     fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
//         match self {
//             &Constraints::Constraints(ref m) => {
//                 write!(f, "Constraints[")?;
//                 for (pkg, constraint) in m.iter() {
//                     write!(f, ":: {} {:?} ", pkg, constraint)?;
//                 }
//                 write!(f, "::]")
//             }
//         }
//     }
// }

impl Default for ConstraintSet {
    fn default() -> Self {
        ConstraintSet(Map::new())
    }
}
