use crate::constraint::VersionConstraint;
use crate::package::PackageName;
use crate::solver::constraints::Constraint;
use crate::solver::path::Path;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq)]
pub enum Failure {
    Conflict(Conflict),
    PackageMissing(PackageMissing),
    UninhabitedConstraint(UninhabitedConstraint),
}

impl Failure {
    pub fn conflict(
        package: Arc<PackageName>,
        existing: Constraint,
        conflicting: Constraint,
    ) -> Failure {
        Failure::Conflict(Conflict {
            package,
            existing,
            conflicting,
        })
    }

    pub fn package_missing(package: Arc<PackageName>, path: Path) -> Failure {
        Failure::PackageMissing(PackageMissing { package, path })
    }

    pub fn uninhabited_constraint(
        package: Arc<PackageName>,
        constraint: Arc<VersionConstraint>,
        path: Path,
    ) -> Failure {
        Failure::UninhabitedConstraint(UninhabitedConstraint {
            package,
            constraint,
            path,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Conflict {
    pub package: Arc<PackageName>,
    pub existing: Constraint,
    pub conflicting: Constraint,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PackageMissing {
    pub package: Arc<PackageName>,
    pub path: Path,
}

#[derive(Debug, PartialEq, Eq)]
pub struct UninhabitedConstraint {
    pub package: Arc<PackageName>,
    pub constraint: Arc<VersionConstraint>,
    pub path: Path,
}
