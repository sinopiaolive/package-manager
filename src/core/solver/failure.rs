use std::sync::Arc;
use manifest::PackageName;
use solver::constraints::Constraint;
use solver::path::Path;
use constraint::VersionConstraint;

pub enum Failure {
    Conflict(Conflict),
    PackageMissing(PackageMissing),
    UninhabitedConstraint(UninhabitedConstraint),
}

impl Failure {
    pub fn conflict(package: Arc<PackageName>,
                    existing: Constraint,
                    conflicting: Constraint)
                    -> Failure {
        Failure::Conflict(Conflict {
            package: package.clone(),
            existing: existing.clone(),
            conflicting: conflicting.clone(),
        })
    }

    pub fn package_missing(package: Arc<PackageName>, path: Path) -> Failure {
        Failure::PackageMissing(PackageMissing {
            package: package.clone(),
            path: path.clone(),
        })
    }

    pub fn uninhabited_constraint(package: Arc<PackageName>,
                                  constraint: Arc<VersionConstraint>,
                                  path: Path)
                                  -> Failure {
        Failure::UninhabitedConstraint(UninhabitedConstraint {
            package: package.clone(),
            constraint: constraint.clone(),
            path: path.clone(),
        })
    }
}

pub struct Conflict {
    pub package: Arc<PackageName>,
    pub existing: Constraint,
    pub conflicting: Constraint,
}

pub struct PackageMissing {
    package: Arc<PackageName>,
    path: Path,
}

pub struct UninhabitedConstraint {
    package: Arc<PackageName>,
    constraint: Arc<VersionConstraint>,
    path: Path,
}
