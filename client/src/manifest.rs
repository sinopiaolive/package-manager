use pm_lib::manifest::{PackageName, DependencySet};
use pm_lib::version::Version;

// The Manifest struct represents a parsed manifest file.

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Manifest {
    // Once we support groups, this will be a different type.
    pub dependencies: DependencySet,

    pub metadata: Option<Metadata>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Metadata {
    pub name: PackageName,
    pub version: Version,

    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub homepage: Option<String>,
    pub bugs: Option<String>,
    pub repository: Option<String>,

    pub license: Option<String>,
    pub license_files: Vec<String>,

    // To do: files, authors
}
