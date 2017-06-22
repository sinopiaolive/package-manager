use manifest::PackageName;
use solver::path::Path;
use immutable_map::map::TreeMap as Map;
use std::fmt;
use std::sync::Arc;
use version::Version;
use solver::solution::{PartialSolution, JustifiedVersion};
use solver::failure::Failure;
use solver::mappable::Mappable;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Constraint(pub Map<Arc<Version>, Path>);

impl Constraint {
    pub fn new() -> Constraint {
        Constraint(Map::new())
    }

    pub fn and(&self,
               other: &Constraint,
               package: Arc<PackageName>)
               -> Result<(Constraint, bool), Failure> {
        let mut out = Constraint::new();
        let mut modified = false;
        for (version, self_path) in self.iter() {
            if let Some(ref other_path) = other.get(version) {
                let shortest_path = if self_path.length() <= other_path.length() {
                    self_path
                } else {
                    modified = true; // we changed a path
                    other_path
                };
                out = out.insert(version.clone(), shortest_path.clone());
            } else {
                modified = true; // we dropped a version from the set
            }
        }

        if out.is_empty() {
            Err(Failure::conflict(package.clone(), self.clone(), other.clone()))
        } else {
            Ok((out, modified))
        }
    }

    pub fn or(&self, other: &Constraint) -> Constraint {
        let mut out = self.clone();
        for (version, other_path) in other.iter() {
            out = match self.get(&version) {
                Some(self_path) if other_path.length() < self_path.length() => out,
                _ => out.insert(version.clone(), other_path.clone()),
            }
        }
        out
    }
}

impl Mappable for Constraint {
    type K = Arc<Version>;
    type V = Path;

    fn as_map(&self) -> &Map<Self::K, Self::V> {
        &self.0
    }

    fn wrap(m: Map<Self::K, Self::V>) -> Self {
        Constraint(m)
    }
}

// impl fmt::Debug for Constraint {
//     fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
//         write!(f, "{} [{:?}]", self.sum, self.paths)
//     }
// }

pub struct BreadthFirstIter {
    paths: Vec<Path>,
    vec_pos: usize,
}

impl BreadthFirstIter {
    pub fn new(left: &Constraint, right: &Constraint) -> BreadthFirstIter {
        let mut vec = Vec::new();
        vec.extend(left.0.values().cloned());
        vec.extend(right.0.values().cloned());
        BreadthFirstIter {
            vec_pos: vec.len() - 1,
            paths: vec,
        }
    }
}

impl Iterator for BreadthFirstIter {
    type Item = (Arc<PackageName>, Arc<Version>);

    fn next(&mut self) -> Option<Self::Item> {
        let started = self.vec_pos;
        loop {
            self.vec_pos = (self.vec_pos + 1) % self.paths.len();
            if self.vec_pos == started {
                return None;
            }
            let l = self.paths[self.vec_pos].clone();
            match l.uncons() {
                None => continue,
                Some((car, cdr)) => {
                    self.paths[self.vec_pos] = cdr;
                    return Some(car.clone());
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ConstraintSet(pub Map<Arc<PackageName>, Constraint>);

impl ConstraintSet {
    pub fn new() -> ConstraintSet {
        ConstraintSet(Map::new())
    }

    pub fn pop_most_interesting_package
        (&self,
         cheap_conflict: &Option<Failure>)
         -> Option<(ConstraintSet, Arc<PackageName>, Constraint)> {
        let path_iter: Box<Iterator<Item = (Arc<PackageName>, Arc<Version>)>> =
            match cheap_conflict {
                &Some(Failure::Conflict(ref conflict)) => {
                    Box::new(BreadthFirstIter::new(&conflict.existing, &conflict.conflicting))
                }
                &Some(Failure::PackageMissing(ref pkg_missing)) => {
                    Box::new(pkg_missing.path.iter())
                }
                &Some(Failure::UninhabitedConstraint(ref pkg_missing)) => {
                    Box::new(pkg_missing.path.iter())
                }
                &None => Box::new(::std::iter::empty()),
            };
        for (package, _version) in path_iter {
            if let Some((cdr, constraint)) = self.remove(&package) {
                return Some((cdr, package.clone(), constraint.clone()));
            }
        }
        // Fall back to popping alphabetically.
        match self.0.delete_min() {
            None => None,
            Some((cdr, (k, v))) => Some((ConstraintSet(cdr), k.clone(), v.clone())),
        }
    }
    // TODONEXT test this

    pub fn and(&self,
               new: &ConstraintSet,
               solution: &PartialSolution)
               -> Result<(ConstraintSet, bool), Failure> {
        let mut out = self.clone();
        let mut modified = false;
        for (package, new_constraint) in new.iter() {
            if contained_in(package.clone(), new_constraint, solution)? {
                continue;
            }
            out = match out.get(package) {
                None => {
                    modified = true;
                    out.insert(package.clone(), new_constraint.clone())
                }
                Some(ref existing_constraint) => {
                    let (updated_constraint, constraint_modified) =
                        existing_constraint.and(&new_constraint, package.clone())?;
                    modified = modified || constraint_modified;
                    out.insert(package.clone(), updated_constraint)
                }
            }
        }
        Ok((out, modified))
    }

    pub fn or(&self, other: &ConstraintSet) -> ConstraintSet {
        let mut out = ConstraintSet::new();
        for (package, self_constraint) in self.iter() {
            if let Some(other_constraint) = other.get(&package) {
                out = out.insert(package.clone(), self_constraint.or(other_constraint))
            }
        }
        out
    }
}

impl Mappable for ConstraintSet {
    type K = Arc<PackageName>;
    type V = Constraint;

    fn as_map(&self) -> &Map<Self::K, Self::V> {
        &self.0
    }

    fn wrap(m: Map<Self::K, Self::V>) -> Self {
        ConstraintSet(m)
    }
}

impl fmt::Debug for ConstraintSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "ConstraintSet(\n")?;
        for (package, constraint) in self.iter() {
            write!(f, "    {:?}: {:?}\n", package, constraint)?;
        }
        write!(f, ")")
    }
}

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

#[cfg(test)]
mod test {
    use super::*;
    use test::pkg;
    use solver::test::{constraint, constraint_set, partial_sln};

    #[test]
    fn constraint_merge() {
        let c1 = constraint(&[("1.0.0", &[("A", "1.0.0")]),
                              ("1.0.1", &[("A", "2.0.0"), ("B", "2.0.0")])]);
        let c2 = constraint(&[("1.0.1", &[("C", "1.0.0")]), ("1.0.2", &[("C", "2.0.0")])]);
        let expected = constraint(&[("1.0.1", &[("C", "1.0.0")])]);
        let merged = c1.and(&c2, Arc::new(pkg("X")));
        assert_eq!(merged, Ok((expected, true)));
    }

    #[test]
    fn constraint_merge_unmodified() {
        // c2 can have additional versions, and different paths (that are not
        // shorter)
        let c1 = constraint(&[("1.0.0", &[("A", "1.0.0")])]);
        let c2 = constraint(&[("1.0.0", &[("C", "1.0.0")]), ("2.0.0", &[])]);
        let merged = c1.and(&c2, Arc::new(pkg("X")));
        assert_eq!(merged, Ok((c1, false)));
    }

    #[test]
    fn constraint_merge_modified_due_to_shorter_path() {
        let c1 = constraint(&[("1.0.0", &[("A", "1.0.0")])]);
        let c2 = constraint(&[("1.0.0", &[])]);
        let expected = constraint(&[("1.0.0", &[])]);
        let merged = c1.and(&c2, Arc::new(pkg("X")));
        assert_eq!(merged, Ok((expected, true)));
    }

    #[test]
    fn constraint_merge_modified_due_to_version() {
        let c1 = constraint(&[("1.0.0", &[]), ("2.0.0", &[])]);
        let c2 = constraint(&[("1.0.0", &[])]);
        let expected = constraint(&[("1.0.0", &[])]);
        let merged = c1.and(&c2, Arc::new(pkg("X")));
        assert_eq!(merged, Ok((expected, true)));
    }

    #[test]
    fn constraint_merge_conflict() {
        let c1 = constraint(&[("1.0.0", &[("A", "1.0.0")])]);
        let c2 = constraint(&[("2.0.0", &[("B", "1.0.0")])]);
        let expected_failure = Failure::conflict(Arc::new(pkg("X")), c1.clone(), c2.clone());
        let merged = c1.and(&c2, Arc::new(pkg("X")));
        assert_eq!(merged, Err(expected_failure));
    }

    #[test]
    fn constraint_set_merge() {
        let existing = constraint_set(&[("A", &[("1.0.0", &[])]),
                                        ("B", &[("1.0.0", &[]), ("2.0.0", &[])])]);
        let new = constraint_set(&[("B", &[("2.0.0", &[]), ("3.0.0", &[])]),
                                   ("C", &[("1.0.0", &[])]),
                                   ("S", &[("1.0.0", &[])])]);
        let ps = partial_sln(&[("S", ("1.0.0", &[]))]);
        let expected = constraint_set(&[("A", &[("1.0.0", &[])]),
                                        ("B", &[("2.0.0", &[])]),
                                        ("C", &[("1.0.0", &[])])]);
        let merged = existing.and(&new, &ps);
        assert_eq!(merged, Ok((expected, true)));
    }

    #[test]
    fn constraint_set_merge_unmodified() {
        let existing = constraint_set(&[("A", &[("1.0.0", &[])]), ("B", &[("1.0.0", &[])])]);
        let new = constraint_set(&[("B", &[("1.0.0", &[]), ("2.0.0", &[])]),
                                   ("S", &[("1.0.0", &[])])]);
        let ps = partial_sln(&[("S", ("1.0.0", &[]))]);
        let merged = existing.and(&new, &ps);
        assert_eq!(merged, Ok((existing, false)));
    }

    #[test]
    fn constraint_set_merge_modified_due_to_changed_constraint() {
        let existing = constraint_set(&[("A", &[("1.0.0", &[]), ("2.0.0", &[])])]);
        let new = constraint_set(&[("A", &[("1.0.0", &[])])]);
        let ps = partial_sln(&[]);
        let merged = existing.and(&new, &ps);
        assert_eq!(merged, Ok((new, true)));
    }

    #[test]
    fn constraint_set_merge_modified_due_to_added_package() {
        let existing = constraint_set(&[("A", &[("1.0.0", &[])])]);
        let new = constraint_set(&[("B", &[("1.0.0", &[])])]);
        let ps = partial_sln(&[]);
        let expected = constraint_set(&[("A", &[("1.0.0", &[])]), ("B", &[("1.0.0", &[])])]);
        let merged = existing.and(&new, &ps);
        assert_eq!(merged, Ok((expected, true)));
    }

    #[test]
    fn constraint_set_merge_partial_solution_conflict() {
        let existing = constraint_set(&[]);
        let ps = partial_sln(&[("S", ("1.0.0", &[("P1", "1.0.0")]))]);
        let new = constraint_set(&[("S", &[("2.0.0", &[("P2", "1.0.0")])])]);
        let expected_failure = Failure::conflict(Arc::new(pkg("S")),
                                                 constraint(&[("1.0.0", &[("P1", "1.0.0")])]),
                                                 constraint(&[("2.0.0", &[("P2", "1.0.0")])]));
        let merged = existing.and(&new, &ps);
        assert_eq!(merged, Err(expected_failure));
    }

    #[test]
    fn constraint_set_merge_existing_cset_conflict() {
        let existing = constraint_set(&[("A", &[("1.0.0", &[("P1", "1.0.0")])])]);
        let new = constraint_set(&[("A", &[("2.0.0", &[("P2", "1.0.0")])])]);
        let ps = partial_sln(&[]);
        let expected_failure = Failure::conflict(Arc::new(pkg("A")),
                                                 constraint(&[("1.0.0", &[("P1", "1.0.0")])]),
                                                 constraint(&[("2.0.0", &[("P2", "1.0.0")])]));
        let merged = existing.and(&new, &ps);
        assert_eq!(merged, Err(expected_failure));
    }
}
