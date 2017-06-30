use std::sync::Arc;
use constraint::VersionConstraint;
use manifest::{PackageName};
use index::{Index, Dependencies};
use solver::path::Path;
use solver::failure;
use solver::failure::Failure;
pub use solver::failure::{PackageMissing, UninhabitedConstraint};
use solver::adapter::RegistryAdapter;
use solver::mappable::Mappable;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Conflict(Conflict),
    PackageMissing(PackageMissing),
    UninhabitedConstraint(UninhabitedConstraint),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Conflict {
    pub package: Arc<PackageName>,
    pub existing: VersionConstraint,
    pub existing_path: Path,
    pub conflicting: VersionConstraint,
    pub conflicting_path: Path,
}

impl Error {
    pub fn from_failure(
        registry: &Index,
        deps: &Dependencies,
        ra: &RegistryAdapter,
        failure: Failure,
    ) -> Self {
        match failure {
            Failure::Conflict(f) => Error::Conflict(Conflict::from(&registry, &deps, &ra, f)),
            Failure::PackageMissing(f) => Error::PackageMissing(f),
            Failure::UninhabitedConstraint(f) => Error::UninhabitedConstraint(f),
        }
    }
}

impl Conflict {
    /// This function turns a `solver::failure::Conflict` (internal to the
    /// solver) into an `error::Conflict`. While the `failure::Conflict` has a
    /// `Constraint` (a set of versions), the `error::Conflict` has a
    /// `VersionConstraint` (a range specified in a manifest).
    ///
    /// The Constraints are justified by paths, so we look up matching
    /// VersionConstraints in the registry or the deps.
    ///
    /// Most of the work this function performs is due to the following edge
    /// case:
    ///
    /// The Constraints might have been narrowed by the solver, so the
    /// VersionConstraints might be wider (weaker) than the corresponding
    /// Constraints. For example, say the existing constraint is [1.0] and the
    /// conflicting constraint is [1.1], and we find VersionConstraints ^1.0 and
    /// ^1.1. Clearly ^1.0 should match [1.0, 1.1], so the solver must have
    /// somehow discovered that 1.1 is impossible and narrowed it to [1.0].
    /// If we naively reported those VersionConstraints to the user, it would
    /// look like this:
    ///
    /// ```text
    /// Conflict in package X:
    /// A 1.0 -> X ^1.0
    /// B 1.0 -> X ^1.1
    /// ```
    ///
    /// This will leave the user confused, because ^1.0 and ^1.1 should not
    /// conflict. With our simple Conflict structure that will produce simple
    /// error messages, we cannot always give an exhaustive explanation of why a
    /// conflict occurred, because it may involve many different paths. So we do
    /// the next-best thing and substitute an artificial exact VersionConstraint
    /// 1.0, producing the following error message:
    ///
    /// ```text
    /// Conflict in package X:
    /// A 1.0 -> X 1.0
    /// B 1.0 -> X ^1.1
    /// ```
    ///
    /// Technically, A 1 depends on X ^1.0 and not X 1.0, but this appears to be
    /// the least-confusing error we can produce in this case.
    fn from(
        registry: &Index,
        deps: &Dependencies,
        ra: &RegistryAdapter,
        conflict: failure::Conflict,
    ) -> Self {
        let vc_from_path = |path: &Path| {
            // Recall that paths are stored right-to-left, in the opposite order
            // of how we print them. Thus we need the head.
            let depset = match path.head().map(|v| (*v).clone()) {
                None => deps,
                Some((ref pkg, ref ver)) => {
                    &registry
                        .get(&pkg)
                        .expect("path package must exist in registry")
                        .get(&ver)
                        .expect("path version must exist in registry")
                }
            };
            depset
                .get(&conflict.package)
                .expect(
                    "package must be listed in dependency set, according to path",
                )
                .clone()
        };

        let (existing_ver, existing_path) = conflict.existing.iter().next().expect(
            "constraints must not be empty",
        );
        let (conflicting_ver, conflicting_path) = conflict.conflicting.iter().next().expect(
            "constraints must not be empty",
        );

        // Get version constraints justifying existing and conflicting version
        // from the registry or deps ("original existing", "original
        // conflicting"):
        let oe = vc_from_path(&existing_path);
        let oc = vc_from_path(&conflicting_path);
        // Make exact version constraints in case the original ones overlap
        // ("narrow existing", "narrow conflicting"):
        let ne = VersionConstraint::Exact((*existing_ver).clone());
        let nc = VersionConstraint::Exact((*conflicting_ver).clone());

        let disjoint = |vc1: &VersionConstraint, vc2: &VersionConstraint| -> bool {
            // Turn version constraints into constraints
            let c1 = ra.constraint_for(conflict.package.clone(), Arc::new(vc1.clone()), Path::default())
                .expect("we should not have gotten a conflict if there is a PackageMissing or UninhabitedConstraint error");
            let c2 = ra.constraint_for(conflict.package.clone(), Arc::new(vc2.clone()), Path::default())
                .expect("we should not have gotten a conflict if there is a PackageMissing or UninhabitedConstraint error");

            c1.as_map().keys().all(|ver| !c2.contains_key(&ver))
        };

        // Use the original version constraints unless they overlap.
        let (existing_vc, conflicting_vc) = if disjoint(&oe, &oc) {
            (oe, oc)
        } else if disjoint(&oe, &nc) {
            (oe, nc)
        } else if disjoint(&ne, &oc) {
            (ne, oc)
        } else {
            (ne, nc)
        };

        Conflict {
            package: conflict.package.clone(),
            existing: existing_vc,
            existing_path: (*existing_path).clone(),
            conflicting: conflicting_vc,
            conflicting_path: (*conflicting_path).clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test_helpers::{pkg, range};
    use solver::test_helpers::{constraint, path};

    #[test]
    fn test_conflict_from_solver_conflict() {
        let registry =
            gen_registry!(
                A => (
                    "1" => deps!(
                        X => ">= 1 < 3"
                    ),
                    "2" => deps!(
                        X => ">= 2 < 3"
                    ),
                    "3" => deps!(
                        X => ">= 2 < 4"
                    )
                ),
                B => (
                    "1" => deps!(
                        A => ">= 1 < 4",
                        C => ">= 1 < 4"
                    )
                ),
                C => (
                    "1" => deps!(
                        X => ">= 1 < 4"
                    )
                ),
                X => (
                    "1" => deps!(),
                    "2" => deps!(),
                    "3" => deps!()
                )
            );
        let ra = RegistryAdapter::new(&registry);
        let deps =
            deps!(
                X => ">= 3 < 4"
            );

        // Disjoint
        let sc1 = failure::Conflict {
            package: Arc::new(pkg("X")),
            existing: constraint(
                &[
                    ("1", &[("A", "1"), ("B", "1")]),
                    ("2", &[("A", "2"), ("B", "1")]),
                ],
            ),
            conflicting: constraint(&[("3", &[])]),
        };
        assert_eq!(
            Conflict::from(&registry, &deps, &ra, sc1),
            Conflict {
                package: Arc::new(pkg("X")),
                existing: range(">= 2 < 3"), // from A 2
                existing_path: path(&[("A", "2"), ("B", "1")]),
                conflicting: range(">= 3 < 4"), // from deps
                conflicting_path: path(&[]),
            }
        );

        // Overlapping
        let sc2 = failure::Conflict {
            package: Arc::new(pkg("X")),
            existing: constraint(
                &[
                    ("1", &[("A", "1"), ("B", "1")]),
                    ("2", &[("C", "1"), ("B", "1")]),
                ],
            ),
            conflicting: constraint(&[("3", &[])]),
        };
        assert_eq!(
            Conflict::from(&registry, &deps, &ra, sc2),
            Conflict {
                package: Arc::new(pkg("X")),
                existing: range("2"), // C 1's range was replaced
                existing_path: path(&[("C", "1"), ("B", "1")]),
                conflicting: range(">= 3 < 4"), // from deps
                conflicting_path: path(&[]),
            }
        );

        // Overlapping -- same but with existing and conflicting swapped
        let sc3 = failure::Conflict {
            package: Arc::new(pkg("X")),
            conflicting: constraint(
                &[
                    ("1", &[("A", "1"), ("B", "1")]),
                    ("2", &[("C", "1"), ("B", "1")]),
                ],
            ),
            existing: constraint(&[("3", &[])]),
        };
        assert_eq!(
            Conflict::from(&registry, &deps, &ra, sc3),
            Conflict {
                package: Arc::new(pkg("X")),
                conflicting: range("2"), // C 1's range was replaced
                conflicting_path: path(&[("C", "1"), ("B", "1")]),
                existing: range(">= 3 < 4"), // from deps
                existing_path: path(&[]),
            }
        );

    }
}
