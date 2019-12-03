use crate::package::PackageName;
use crate::version::Version;
use im::OrdMap as Map;
use solver::failure::Failure;
use solver::mappable::Mappable;
use solver::path::Path;
use solver::solution::{JustifiedVersion, PartialSolution};
use std::fmt;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Constraint(pub Map<Arc<Version>, Path>);

impl PartialEq for Constraint {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for Constraint {}

impl Default for Constraint {
    fn default() -> Self {
        Self::new()
    }
}

impl Constraint {
    pub fn new() -> Constraint {
        Constraint(Map::new())
    }

    pub fn and(
        &self,
        other: &Constraint,
        package: Arc<PackageName>,
    ) -> Result<(Constraint, bool), Failure> {
        let mut out = Constraint::new();
        let mut modified = false;
        for (version, self_path) in self.iter() {
            if let Some(ref other_path) = other.get(version) {
                // The version is included in both constraints, so we include it
                // in the intersection. It is correct to pick either self_path
                // or other_path to justify this version. To help us get good
                // error messages, we pick the shortest path, or if they're
                // equal in length, the path from the narrower constraint. (This
                // is the best we can do without looking up the original
                // VersionConstraints on the registry.)
                let path = if self_path.len() < other_path.len()
                    || (self_path.len() == other_path.len() && self.len() <= other.len())
                {
                    self_path
                } else {
                    modified = true; // we changed a path
                    other_path
                };
                out = out.insert(version.clone(), path.clone());
            } else {
                modified = true; // we dropped a version from the set
            }
        }

        if out.is_empty() {
            Err(Failure::conflict(package, self.clone(), other.clone()))
        } else {
            Ok((out, modified))
        }
    }

    pub fn or(&self, other: &Constraint) -> Constraint {
        let mut out = self.clone();
        for (version, other_path) in other.iter() {
            out = match self.get(version) {
                Some(ref self_path) if other_path.len() < self_path.len() => out,
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

pub struct BreadthFirstIter {
    paths: Vec<Path>,
    depth: usize,
    vec_pos: usize,
}

impl BreadthFirstIter {
    pub fn new(left: &Constraint, right: &Constraint) -> BreadthFirstIter {
        let mut vec = Vec::new();
        vec.extend(left.0.values().map(|v| (*v).clone()));
        vec.extend(right.0.values().map(|v| (*v).clone()));
        BreadthFirstIter {
            vec_pos: 0,
            depth: 0,
            paths: vec,
        }
    }
}

impl Iterator for BreadthFirstIter {
    type Item = (Arc<PackageName>, Arc<Version>);

    fn next(&mut self) -> Option<Self::Item> {
        let started = self.vec_pos;
        loop {
            let old_vec_pos = self.vec_pos;
            let old_depth = self.depth;
            self.vec_pos = (self.vec_pos + 1) % self.paths.len();
            if self.vec_pos == 0 {
                self.depth += 1;
            }
            let path = &self.paths[old_vec_pos];
            if old_depth < path.len() {
                return Some(path[old_depth].clone());
            }
            if self.vec_pos == started {
                return None;
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ConstraintSet(pub Map<Arc<PackageName>, Constraint>);

impl Default for ConstraintSet {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstraintSet {
    pub fn new() -> ConstraintSet {
        ConstraintSet(Map::new())
    }

    pub fn pop(
        &self,
        cheap_conflict: &Option<Failure>,
    ) -> Option<(ConstraintSet, Arc<PackageName>, Constraint)> {
        let path_iter: Box<dyn Iterator<Item = (Arc<PackageName>, Arc<Version>)>> =
            match cheap_conflict {
                Some(Failure::Conflict(ref conflict)) => Box::new(
                    BreadthFirstIter::new(&conflict.existing, &conflict.conflicting).chain(
                        ::std::iter::once((
                            conflict.package.clone(),
                            Arc::new(Version::new(vec![], vec![], vec![])),
                        )),
                    ),
                ),
                Some(Failure::PackageMissing(ref pkg_missing)) => {
                    Box::new(pkg_missing.path.iter().rev().cloned())
                }
                Some(Failure::UninhabitedConstraint(ref pkg_missing)) => {
                    Box::new(pkg_missing.path.iter().rev().cloned())
                }
                None => Box::new(::std::iter::empty()),
            };
        for (ref package, _) in path_iter {
            if let Some((constraint, cdr)) = self.uncons(package) {
                return Some((cdr, package.clone(), constraint));
            }
        }
        // Fall back to popping alphabetically.
        match self.0.without_min_with_key() {
            (None, _) => None,
            (Some((package, constraint)), cdr) => Some((ConstraintSet(cdr), package, constraint)),
        }
    }

    pub fn and(
        &self,
        new: &ConstraintSet,
        solution: &PartialSolution,
    ) -> Result<(ConstraintSet, bool), Failure> {
        let mut out = self.clone();
        let mut modified = false;
        for (package, new_constraint) in new.iter() {
            if contained_in(package, &new_constraint, solution)? {
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
            };
        }
        Ok((out, modified))
    }

    pub fn or(&self, other: &ConstraintSet) -> ConstraintSet {
        let mut out = ConstraintSet::new();
        for (package, self_constraint) in self.iter() {
            if let Some(other_constraint) = other.get(package) {
                out = out.insert(package.clone(), self_constraint.or(&other_constraint))
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
        writeln!(f, "ConstraintSet(")?;
        for (package, constraint) in self.iter() {
            writeln!(f, "    {:?}: {:?}", package, constraint)?;
        }
        write!(f, ")")
    }
}

fn contained_in(
    package: &Arc<PackageName>,
    constraint: &Constraint,
    solution: &PartialSolution,
) -> Result<bool, Failure> {
    match solution.get(package).map(|v| (*v).clone()) {
        None => Ok(false),
        Some(JustifiedVersion {
            ref version,
            ref path,
        }) if !constraint.contains_key(&version.clone()) => {
            let exact_constraint = Constraint::new().insert(version.clone(), path.clone());
            Err(Failure::conflict(
                package.clone(),
                exact_constraint,
                constraint.clone(),
            ))
        }
        _ => Ok(true),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_helpers::{pkg, range};
    use solver::test_helpers::{constraint, constraint_set, partial_sln, path};

    #[test]
    fn constraint_merge() {
        let c1 = constraint(&[("1", &[("A", "1")]), ("1.0.1", &[("B", "2"), ("A", "2")])]);
        let c2 = constraint(&[("1.0.1", &[("C", "1")]), ("1.0.2", &[("C", "2")])]);
        let expected = constraint(&[("1.0.1", &[("C", "1")])]);
        let merged = c1.and(&c2, Arc::new(pkg("X")));
        assert_eq!(merged, Ok((expected, true)));
    }

    #[test]
    fn constraint_merge_unmodified() {
        // c2 can have additional versions, and different paths (that are not
        // shorter)
        let c1 = constraint(&[("1", &[("A", "1")])]);
        let c2 = constraint(&[("1", &[("C", "1")]), ("2", &[])]);
        let merged = c1.and(&c2, Arc::new(pkg("X")));
        assert_eq!(merged, Ok((c1, false)));
    }

    #[test]
    fn constraint_merge_modified_due_to_shorter_path() {
        let c1 = constraint(&[("1", &[("A", "1")])]);
        let c2 = constraint(&[("1", &[])]);
        let expected = constraint(&[("1", &[])]);
        let merged = c1.and(&c2, Arc::new(pkg("X")));
        assert_eq!(merged, Ok((expected, true)));
    }

    #[test]
    fn constraint_merge_modified_due_to_version() {
        let c1 = constraint(&[("1", &[]), ("2", &[])]);
        let c2 = constraint(&[("1", &[])]);
        let expected = constraint(&[("1", &[])]);
        let merged = c1.and(&c2, Arc::new(pkg("X")));
        assert_eq!(merged, Ok((expected, true)));
    }

    #[test]
    fn constraint_merge_conflict() {
        let c1 = constraint(&[("1", &[("A", "1")])]);
        let c2 = constraint(&[("2", &[("B", "1")])]);
        let expected_failure = Failure::conflict(Arc::new(pkg("X")), c1.clone(), c2.clone());
        let merged = c1.and(&c2, Arc::new(pkg("X")));
        assert_eq!(merged, Err(expected_failure));
    }

    #[test]
    fn constraint_set_merge() {
        let existing = constraint_set(&[("A", &[("1", &[])]), ("B", &[("1", &[]), ("2", &[])])]);
        let new = constraint_set(&[
            ("B", &[("2", &[]), ("3", &[])]),
            ("C", &[("1", &[])]),
            ("S", &[("1", &[])]),
        ]);
        let ps = partial_sln(&[("S", ("1", &[]))]);
        let expected = constraint_set(&[
            ("A", &[("1", &[])]),
            ("B", &[("2", &[])]),
            ("C", &[("1", &[])]),
        ]);
        let merged = existing.and(&new, &ps);
        assert_eq!(merged, Ok((expected, true)));
    }

    #[test]
    fn constraint_set_merge_unmodified() {
        let existing = constraint_set(&[("A", &[("1", &[])]), ("B", &[("1", &[])])]);
        let new = constraint_set(&[("B", &[("1", &[]), ("2", &[])]), ("S", &[("1", &[])])]);
        let ps = partial_sln(&[("S", ("1", &[]))]);
        let merged = existing.and(&new, &ps);
        assert_eq!(merged, Ok((existing, false)));
    }

    #[test]
    fn constraint_set_merge_modified_due_to_changed_constraint() {
        let existing = constraint_set(&[("A", &[("1", &[]), ("2", &[])])]);
        let new = constraint_set(&[("A", &[("1", &[])])]);
        let ps = partial_sln(&[]);
        let merged = existing.and(&new, &ps);
        assert_eq!(merged, Ok((new, true)));
    }

    #[test]
    fn constraint_set_merge_modified_due_to_added_package() {
        let existing = constraint_set(&[("A", &[("1", &[])])]);
        let new = constraint_set(&[("B", &[("1", &[])])]);
        let ps = partial_sln(&[]);
        let expected = constraint_set(&[("A", &[("1", &[])]), ("B", &[("1", &[])])]);
        let merged = existing.and(&new, &ps);
        assert_eq!(merged, Ok((expected, true)));
    }

    #[test]
    fn constraint_set_merge_partial_solution_conflict() {
        let existing = constraint_set(&[]);
        let ps = partial_sln(&[("S", ("1", &[("P1", "1")]))]);
        let new = constraint_set(&[("S", &[("2", &[("P2", "1")])])]);
        let expected_failure = Failure::conflict(
            Arc::new(pkg("S")),
            constraint(&[("1", &[("P1", "1")])]),
            constraint(&[("2", &[("P2", "1")])]),
        );
        let merged = existing.and(&new, &ps);
        assert_eq!(merged, Err(expected_failure));
    }

    #[test]
    fn constraint_set_merge_existing_cset_conflict() {
        let existing = constraint_set(&[("A", &[("1", &[("P1", "1")])])]);
        let new = constraint_set(&[("A", &[("2", &[("P2", "1")])])]);
        let ps = partial_sln(&[]);
        let expected_failure = Failure::conflict(
            Arc::new(pkg("A")),
            constraint(&[("1", &[("P1", "1")])]),
            constraint(&[("2", &[("P2", "1")])]),
        );
        let merged = existing.and(&new, &ps);
        assert_eq!(merged, Err(expected_failure));
    }

    #[test]
    fn pop_basic_interesting() {
        let cset = constraint_set(&[("B", &[("1", &[])]), ("A", &[("1", &[])])]);

        let cdr1 = constraint_set(&[("B", &[("1", &[])])]);
        let constraint1 = constraint(&[("1", &[])]);
        assert_eq!(
            cset.pop(&None),
            Some((cdr1, Arc::new(pkg("A")), constraint1))
        );

        let cdr2 = constraint_set(&[("A", &[("1", &[])])]);
        let constraint2 = constraint(&[("1", &[])]);
        assert_eq!(
            cset.pop(&Some(Failure::uninhabited_constraint(
                Arc::new(pkg("null")),
                Arc::new(range("^5")),
                path(&[("B", "1")]),
            ))),
            Some((cdr2, Arc::new(pkg("B")), constraint2))
        );
    }

    #[test]
    fn pop_finds_all_paths() {
        let cset = constraint_set(&[("A", &[("1", &[])]), ("B", &[("1", &[])])]);

        assert_eq!(
            cset.pop(&Some(Failure::conflict(
                Arc::new(pkg("B")),
                constraint(&[("1", &[])]),
                constraint(&[("2", &[("A", "1")])]),
            )))
            .unwrap()
            .1,
            Arc::new(pkg("A"))
        );
    }

    #[test]
    fn pop_interesting() {
        let cset = constraint_set(&[
            ("C", &[("1", &[])]),
            ("B", &[("1", &[])]),
            ("A", &[("1", &[])]),
        ]);

        let null_constraint = constraint(&[("1", &[("null", "1")])]);
        assert_eq!(
            cset.pop(&Some(Failure::conflict(
                Arc::new(pkg("B")),
                null_constraint.clone(),
                null_constraint.clone(),
            )))
            .unwrap()
            .1,
            Arc::new(pkg("B"))
        );

        assert_eq!(
            cset.pop(&Some(Failure::conflict(
                Arc::new(pkg("B")),
                constraint(&[("1", &[("null", "1"), ("A", "1"), ("C", "1")])],),
                null_constraint,
            )))
            .unwrap()
            .1,
            Arc::new(pkg("A"))
        );

        assert_eq!(
            cset.pop(&Some(Failure::conflict(
                Arc::new(pkg("B")),
                constraint(&[("1", &[("null", "1"), ("A", "1")])]),
                constraint(&[("1", &[("C", "1"), ("null", "1")])]),
            )))
            .unwrap()
            .1,
            Arc::new(pkg("C"))
        );
    }
}
