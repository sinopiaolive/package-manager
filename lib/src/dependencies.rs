use package::PackageName;
use constraint::VersionConstraint;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub package_name: PackageName,
    pub version_constraint: VersionConstraint,
}
