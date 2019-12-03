use crate::constraint::VersionConstraint;
use crate::index::{Dependencies, Index};
use crate::package::PackageName;
use crate::version::Version;
use crate::solver::constraints::{Constraint, ConstraintSet};
use crate::solver::failure::Failure;
use crate::solver::mappable::Mappable;
use crate::solver::path::Path;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::vec::Vec;

pub struct RegistryAdapter<'r> {
    registry: &'r Index,
    cache: RefCell<HashMap<(PackageName, VersionConstraint), Option<Vec<Version>>>>,
}

impl<'r> RegistryAdapter<'r> {
    pub fn new(registry: &Index) -> RegistryAdapter {
        RegistryAdapter {
            registry,
            cache: RefCell::new(HashMap::new()),
        }
    }

    /// Return a vector of all versions of `package` matching `constraint`, or
    /// `None` if the `package` was not found in the registry. The vector can be
    /// empty if no versions match.
    pub fn versions_for(
        &self,
        package: &PackageName,
        constraint: &VersionConstraint,
    ) -> Option<Vec<Version>> {
        let key = (package.clone(), constraint.clone());
        let mut cache = self.cache.borrow_mut();
        if let Some(value) = cache.get(&key) {
            return value.clone();
        }
        let value = match self.registry.get(&package) {
            None => None,
            Some(pkg) => Some(
                pkg.keys()
                    .filter(|v| constraint.contains(v))
                    .cloned()
                    .collect(),
            ),
        };
        cache.insert(key.clone(), value);
        cache.get(&key).unwrap().clone()
    }

    /// Return a constraint containing all versions of `package` matching
    /// `constraint`. Can fail with PackageMissing or UninhabitedConstraint.
    pub fn constraint_for(
        &self,
        package: &PackageName,
        version_constraint: &VersionConstraint,
        path: &Path,
    ) -> Result<Constraint, Failure> {
        match self.versions_for(package, version_constraint) {
            None => Err(Failure::package_missing(
                Arc::new(package.clone()),
                path.clone(),
            )),
            Some(versions) => {
                if versions.is_empty() {
                    Err(Failure::uninhabited_constraint(
                        Arc::new(package.clone()),
                        Arc::new(version_constraint.clone()),
                        path.clone(),
                    ))
                } else {
                    let mut constraint = Constraint::new();
                    for version in versions {
                        constraint = constraint.insert(Arc::new(version.clone()), path.clone());
                    }
                    Ok(constraint)
                }
            }
        }
    }

    /// Return a constraint set representing all the dependencies of the release
    /// identified by `package` and `version`.
    ///
    /// `path` must not include `(package, version)`. It will be added by this
    /// function instead.
    pub fn constraint_set_for(
        &self,
        package: &PackageName,
        version: &Version,
        path: &Path,
    ) -> Result<ConstraintSet, Failure> {
        let new_path = path.push((Arc::new(package.clone()), Arc::new(version.clone())));
        let release = self
            .registry
            .get(&package)
            .unwrap_or_else(|| panic!("package not found: {}", package))
            .get(&version)
            .unwrap_or_else(|| panic!("release not found: {} {}", package, version));
        let mut constraint_set = ConstraintSet::new();
        for (dep_package, version_constraint) in release {
            let constraint = self.constraint_for(dep_package, version_constraint, &new_path)?;
            constraint_set = constraint_set.insert(Arc::new(dep_package.clone()), constraint);
        }
        Ok(constraint_set)
    }

    pub fn constraint_set_from(&self, deps: &Dependencies) -> Result<ConstraintSet, Failure> {
        let mut constraint_set = ConstraintSet::new();
        for (package, version_constraint) in deps {
            let constraint = self.constraint_for(package, version_constraint, &Path::new())?;
            constraint_set = constraint_set.insert(Arc::new(package.clone()), constraint);
        }
        Ok(constraint_set)
    }
}
