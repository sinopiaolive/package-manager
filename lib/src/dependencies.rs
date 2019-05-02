use package::PackageName;
use constraint::VersionConstraint;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Dependency {
    pub package_name: PackageName,
    pub version_constraint: VersionConstraint,
}
