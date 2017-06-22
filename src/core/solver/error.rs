use std::sync::Arc;
use constraint::VersionConstraint;
use manifest::{PackageName, DependencySet};
use registry::Registry;
use solver::path::Path;
use solver::failure;
use solver::failure::Failure;
pub use solver::failure::{PackageMissing, UninhabitedConstraint};
use solver::adapter::RegistryAdapter;
use solver::mappable::Mappable;

pub enum Error {
    Conflict(Conflict),
    PackageMissing(PackageMissing),
    UninhabitedConstraint(UninhabitedConstraint),
}

pub struct Conflict {
    pub package: Arc<PackageName>,
    pub existing: VersionConstraint,
    pub existing_path: Path,
    pub conflicting: VersionConstraint,
    pub conflicting_path: Path,
}

impl Error {
    pub fn from_failure(
        registry: &Registry,
        deps: &DependencySet,
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
    fn from(
        registry: &Registry,
        deps: &DependencySet,
        ra: &RegistryAdapter,
        conflict: failure::Conflict,
    ) -> Self {
        let vc_from_path = |path: &Path| {
            let depset = match path.head() {
                None => deps,
                Some(&(ref pkg, ref ver)) => {
                    &registry
                        .packages
                        .get(&pkg)
                        .expect("path package must exist in registry")
                        .releases
                        .get(&ver)
                        .expect("path version must exist in registry")
                        .manifest
                        .dependencies
                }
            };
            depset.get(&conflict.package).expect(
                "package must be listed in dependency set, according to path",
            )
        };

        let (existing_ver, existing_path) = conflict.existing.iter().next().expect(
            "constraints must not be empty",
        );
        let (conflicting_ver, conflicting_path) = conflict.conflicting.iter().next().expect(
            "constraints must not be empty",
        );
        // Get constraints justifying existing and conflicting version.
        let version_constraint_for_existing = vc_from_path(&existing_path);
        let version_constraint_for_conflicting = vc_from_path(&conflicting_path);

        let constraint_for_existing = ra.constraint_for(conflict.package.clone(), Arc::new(version_constraint_for_existing.clone()), Path::default()).expect("we should not have gotten a conflict if there is a PackageMissing or UninhabitedConstraint error");
        let constraint_for_conflicting = ra.constraint_for(conflict.package.clone(), Arc::new(version_constraint_for_conflicting.clone()), Path::default()).expect("we should not have gotten a conflict if there is a PackageMissing or UninhabitedConstraint error");


        let constraints_overlap = constraint_for_existing.as_map().keys().any(|ver| {
            !constraint_for_conflicting.get(&ver).is_none()
        });
        let (existing_vc, conflicting_vc) = if constraints_overlap {
            // The version constraints we found have some overlap, so it would
            // be confusing to report them to the user. Therefore, we use
            // artificial exact constraints.
            (
                VersionConstraint::Exact((**existing_ver).clone()),
                VersionConstraint::Exact((**conflicting_ver).clone()),
            )
        } else {
            (
                version_constraint_for_existing.clone(),
                version_constraint_for_conflicting.clone(),
            )
        };
        Conflict {
            package: conflict.package.clone(),
            existing: existing_vc,
            existing_path: existing_path.clone(),
            conflicting: conflicting_vc,
            conflicting_path: conflicting_path.clone(),
        }
    }
}
