use manifest::PackageName;
use solver::path::Path;
use immutable_map::map::{TreeMap as Map, TreeMapIter};
use std::sync::Arc;
use version::Version;
use solver::solution::{PartialSolution, JustifiedVersion};
use solver::failure::Failure;
use std::iter::IntoIterator;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Constraint(Map<Arc<Version>, Path>);

impl Constraint {
    pub fn new() -> Constraint {
        Constraint(Map::new())
    }

    pub fn iter<'r>(&'r self) -> TreeMapIter<'r, Arc<Version>, Path> {
        self.0.iter()
    }

    pub fn insert(&self, key: Arc<Version>, value: Path) -> Constraint {
        Constraint(self.0.insert(key, value))
    }

    pub fn get(&self, key: &Version) -> Option<&Path> {
        self.0.get(key)
    }

    pub fn contains_key(&self, key: &Version) -> bool {
        self.0.contains_key(key)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'r> IntoIterator for &'r Constraint {
    type Item = (&'r Arc<Version>, &'r Path);
    type IntoIter = TreeMapIter<'r, Arc<Version>, Path>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// impl fmt::Debug for Constraint {
//     fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
//         write!(f, "{} [{:?}]", self.sum, self.paths)
//     }
// }

#[derive(Clone, PartialEq, Eq)]
pub struct ConstraintSet(Map<Arc<PackageName>, Constraint>);

impl ConstraintSet {
    pub fn new() -> ConstraintSet {
        ConstraintSet(Map::new())
    }

    pub fn pop(&self) -> Option<(ConstraintSet, (Arc<PackageName>, Constraint))> {
        match self.0.delete_min() {
            None => None,
            Some((new_set, (k, v))) =>
                Some((ConstraintSet(new_set), (k.clone(), v.clone())))
        }
    }

    pub fn insert(&self, key: Arc<PackageName>, value: Constraint) -> ConstraintSet {
        ConstraintSet(self.0.insert(key, value))
    }

    pub fn get(&self, key: &PackageName) -> Option<&Constraint> {
        self.0.get(key)
    }

    pub fn contains_key(&self, key: &PackageName) -> bool {
        self.0.contains_key(key)
    }

    pub fn merge(&self,
                 new: &ConstraintSet,
                 solution: &PartialSolution)
                 -> Result<ConstraintSet, Failure> {
        let mut out = self.clone();
        for (package, new_constraint) in new {
            if contained_in(package.clone(), new_constraint, solution)? {
                continue;
            }
            out = match out.get(package) {
                None => out.insert(package.clone(), new_constraint.clone()),
                Some(ref existing_constraint) => {
                    out.insert(package.clone(),
                               merge_constraints(package.clone(),
                                                 &existing_constraint,
                                                 &new_constraint)?)
                }
            }
        }
        Ok(out)
    }
}

impl<'r> IntoIterator for &'r ConstraintSet {
    type Item = (&'r Arc<PackageName>, &'r Constraint);
    type IntoIter = TreeMapIter<'r, Arc<PackageName>, Constraint>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
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

fn contained_in(package: Arc<PackageName>,
                constraint: &Constraint,
                solution: &PartialSolution)
                -> Result<bool, Failure> {
    match solution.get(&package.clone()) {
        None => Ok(false),
        Some(&JustifiedVersion {
            ref version,
            ref path,
        }) if !constraint.contains_key(&version.clone()) => {
            let exact_constraint = Constraint::new().insert(version.clone(), path.clone());
            Err(Failure::conflict(package.clone(), exact_constraint, constraint.clone()))
        }
        _ => Ok(true),
    }
}

fn merge_constraints(package: Arc<PackageName>,
                     a: &Constraint,
                     b: &Constraint)
                     -> Result<Constraint, Failure> {
    let mut out = Constraint::new();
    for (version, a_path) in a {
        if let Some(ref b_path) = b.get(version) {
            let shortest_path = if a_path.length() <= b_path.length() {
                a_path
            } else {
                b_path
            };
            out = out.insert(version.clone(), shortest_path.clone());
        }
    }

    if out.is_empty() {
        Err(Failure::conflict(package.clone(), a.clone(), b.clone()))
    } else {
        Ok(out)
    }
}
