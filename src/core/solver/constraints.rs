use manifest::PackageName;
use solver::path::Path;
use immutable_map::map::TreeMap as Map;
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

    pub fn merge(&self,
                 other: &Constraint,
                 package: Arc<PackageName>)
                 -> Result<Constraint, Failure> {
        let mut out = Constraint::new();
        for (version, self_path) in self.iter() {
            if let Some(ref other_path) = other.get(version) {
                let shortest_path = if self_path.length() <= other_path.length() {
                    self_path
                } else {
                    other_path
                };
                out = out.insert(version.clone(), shortest_path.clone());
            }
        }

        if out.is_empty() {
            Err(Failure::conflict(package.clone(), self.clone(), other.clone()))
        } else {
            Ok(out)
        }
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ConstraintSet(pub Map<Arc<PackageName>, Constraint>);

impl ConstraintSet {
    pub fn new() -> ConstraintSet {
        ConstraintSet(Map::new())
    }

    pub fn pop(&self) -> Option<(ConstraintSet, (Arc<PackageName>, Constraint))> {
        match self.0.delete_min() {
            None => None,
            Some((new_set, (k, v))) => Some((ConstraintSet(new_set), (k.clone(), v.clone()))),
        }
    }

    pub fn merge(&self,
                 new: &ConstraintSet,
                 solution: &PartialSolution)
                 -> Result<ConstraintSet, Failure> {
        let mut out = self.clone();
        for (package, new_constraint) in new.iter() {
            if contained_in(package.clone(), new_constraint, solution)? {
                continue;
            }
            out = match out.get(package) {
                None => out.insert(package.clone(), new_constraint.clone()),
                Some(ref existing_constraint) => {
                    out.insert(package.clone(),
                               existing_constraint.merge(&new_constraint, package.clone())?)
                }
            }
        }
        Ok(out)
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
        Some(&JustifiedVersion { ref version, ref path })
            if !constraint.contains_key(&version.clone()) => {
            let exact_constraint = Constraint::new().insert(version.clone(), path.clone());
            Err(Failure::conflict(package.clone(), exact_constraint, constraint.clone()))
        }
        _ => Ok(true),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test::*;

    fn path(l: &[(&str, &str)]) -> Path {
        l.iter().map(|&(p, v)| (Arc::new(pkg(p)), Arc::new(ver(v)))).collect()
    }

    fn constraint(l: &[(&str, &[(&str, &str)])]) -> Constraint {
        Constraint(l.iter().map(|&(v, pa)| (Arc::new(ver(v)), path(pa))).collect())
    }

    fn constraint_set(l: &[(&str, &[(&str, &[(&str, &str)])])]) -> ConstraintSet {
        ConstraintSet(l.iter().map(|&(p, c)| (Arc::new(pkg(p)), constraint(c))).collect())
    }

    fn jver(l: (&str, &[(&str, &str)])) -> JustifiedVersion {
        let (v, pa) = l;
        JustifiedVersion {
            version: Arc::new(ver(v)),
            path: path(pa),
        }
    }

    fn partial_sln(l: &[(&str, (&str, &[(&str, &str)]))]) -> PartialSolution {
        PartialSolution(l.iter().map(|&(p, jv)| (Arc::new(pkg(p)), jver(jv))).collect())
    }

    #[test]
    fn constraint_merge() {
        let c1 = constraint(&[("1.0.0", &[("A", "1.0.0")]),
                              ("1.0.1", &[("A", "2.0.0"), ("B", "2.0.0")])]);
        let c2 = constraint(&[("1.0.1", &[("C", "1.0.0")]), ("1.0.2", &[("C", "2.0.0")])]);
        let expected = constraint(&[("1.0.1", &[("C", "1.0.0")])]);
        let merged = c1.merge(&c2, Arc::new(pkg("X")));
        assert_eq!(merged, Ok(expected));
    }

    #[test]
    fn constraint_merge_conflict() {
        let c1 = constraint(&[("1.0.0", &[("A", "1.0.0")])]);
        let c2 = constraint(&[("2.0.0", &[("B", "1.0.0")])]);
        let expected_failure = Failure::conflict(Arc::new(pkg("X")), c1.clone(), c2.clone());
        let merged = c1.merge(&c2, Arc::new(pkg("X")));
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
        let merged = existing.merge(&new, &ps);
        assert_eq!(merged, Ok(expected));
    }

    #[test]
    fn constraint_set_merge_partial_solution_conflict() {
        let existing = constraint_set(&[]);
        let ps = partial_sln(&[("S", ("1.0.0", &[("P1", "1.0.0")]))]);
        let new = constraint_set(&[("S", &[("2.0.0", &[("P2", "1.0.0")])])]);
        let expected_failure = Failure::conflict(
            Arc::new(pkg("S")),
            constraint(&[("1.0.0", &[("P1", "1.0.0")])]),
            constraint(&[("2.0.0", &[("P2", "1.0.0")])]));
        let merged = existing.merge(&new, &ps);
        assert_eq!(merged, Err(expected_failure));
    }

    #[test]
    fn constraint_set_merge_existing_cset_conflict() {
        let existing = constraint_set(&[("A", &[("1.0.0", &[("P1", "1.0.0")])])]);
        let new = constraint_set(&[("A", &[("2.0.0", &[("P2", "1.0.0")])])]);
        let ps = partial_sln(&[]);
        let expected_failure = Failure::conflict(
            Arc::new(pkg("A")),
            constraint(&[("1.0.0", &[("P1", "1.0.0")])]),
            constraint(&[("2.0.0", &[("P2", "1.0.0")])]));
        let merged = existing.merge(&new, &ps);
        assert_eq!(merged, Err(expected_failure));
    }
}
