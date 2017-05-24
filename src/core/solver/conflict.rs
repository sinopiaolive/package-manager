use solver::path::Path;
use manifest::PackageName;
use constraint::VersionConstraint;
use list::List;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedConstraint {
    pub path: Path,
    pub package: PackageName,
    pub constraint: VersionConstraint,
}

impl NamedConstraint {
    pub fn new(package: &PackageName, constraint: &VersionConstraint) -> NamedConstraint {
        NamedConstraint {
            path: list![],
            package: package.clone(),
            constraint: constraint.clone()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conflict {
    pub existing: List<NamedConstraint>,
    pub conflicting: NamedConstraint,
}
