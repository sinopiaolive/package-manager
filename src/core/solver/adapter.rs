use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::vec::Vec;
use constraint::VersionConstraint;
use manifest::{DependencySet, PackageName};
use version::Version;
use registry::Registry;
use solver::failure::Failure;
use solver::constraints::{Constraint, ConstraintSet};
use solver::path::Path;
use solver::mappable::Mappable;

pub struct RegistryAdapter<'r> {
    registry: &'r Registry,
    cache: RefCell<HashMap<(Arc<PackageName>, Arc<VersionConstraint>), Option<Vec<Arc<Version>>>>>,
}

impl<'r> RegistryAdapter<'r> {
    pub fn new(registry: &Registry) -> RegistryAdapter {
        RegistryAdapter {
            registry: registry,
            cache: RefCell::new(HashMap::new()),
        }
    }

    /// Return a vector of all versions of `package` matching `constraint`, or
    /// `None` if the `package` was not found in the registry. The vector can be
    /// empty if no versions match.
    pub fn versions_for(&self,
                        package: Arc<PackageName>,
                        constraint: Arc<VersionConstraint>)
                        -> Option<Vec<Arc<Version>>> {
        let key = (package.clone(), constraint.clone());
        let mut cache = self.cache.borrow_mut();
        if let Some(value) = cache.get(&key) {
            return value.clone();
        }
        let value = match self.registry.packages.get(&package) {
            None => None,
            Some(pkg) => {
                Some(pkg.releases
                         .keys()
                         .filter(|v| constraint.contains(v))
                         .map(|v| Arc::new(v.clone()))
                         .collect())
            }
        };
        cache.insert(key, value.clone());
        value
    }

    /// Return a constraint containing all versions of `package` matching
    /// `constraint`. Can fail with PackageMissing or UninhabitedConstraint.
    pub fn constraint_for(&self,
                          package: Arc<PackageName>,
                          version_constraint: Arc<VersionConstraint>,
                          path: Path)
                          -> Result<Constraint, Failure> {
        match self.versions_for(package.clone(), version_constraint.clone()) {
            None => Err(Failure::package_missing(package.clone(), path.clone())),
            Some(versions) => {
                if versions.is_empty() {
                    Err(Failure::uninhabited_constraint(package.clone(),
                                                        version_constraint.clone(),
                                                        path.clone()))
                } else {
                    let mut constraint = Constraint::new();
                    for version in versions {
                        constraint = constraint.insert(version, path.clone());
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
    pub fn constraint_set_for(&self,
                              package: Arc<PackageName>,
                              version: Arc<Version>,
                              path: Path)
                              -> Result<ConstraintSet, Failure> {
        let new_path = path.cons((package.clone(), version.clone()));
        let release = self.registry
            .packages
            .get(&package)
            .unwrap()
            .releases
            .get(&version)
            .unwrap();
        let dependency_set = &release.manifest.dependencies;
        let mut constraint_set = ConstraintSet::new();
        for (dep_package, version_constraint) in dependency_set {
            let dep_package_arc = Arc::new(dep_package.clone());
            let version_constraint_arc = Arc::new(version_constraint.clone());
            let constraint = self.constraint_for(dep_package_arc.clone(),
                                                 version_constraint_arc.clone(),
                                                 new_path.clone())?;
            constraint_set = constraint_set.insert(dep_package_arc, constraint);
        }
        Ok(constraint_set)
    }

    pub fn constraint_set_from(&self, deps: &DependencySet) -> Result<ConstraintSet, Failure> {
        let mut constraint_set = ConstraintSet::new();
        for (package, version_constraint) in deps {
            let package_arc = Arc::new(package.clone());
            let version_constraint_arc = Arc::new(version_constraint.clone());
            let constraint =
                self.constraint_for(package_arc.clone(), version_constraint_arc.clone(), list![])?;
            constraint_set = constraint_set.insert(package_arc.clone(), constraint);
        }
        Ok(constraint_set)
    }
}
