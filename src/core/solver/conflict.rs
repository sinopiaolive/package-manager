use std::sync::Arc;
use solver::path::Path;
use manifest::PackageName;
use constraint::VersionConstraint;
use list::List;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Constraint {
    pub path: Path,
    pub package: PackageName,
    pub constraint: VersionConstraint,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conflict {
    pub existing: Arc<List<Constraint>>,
    pub conflicting: Constraint,
}