use std::path::PathBuf;
use pm_lib::manifest::{PackageName, DependencySet};
use pm_lib::version::Version;

// The Manifest struct represents a parsed manifest file.

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Release {
    pub name: PackageName,
    pub version: Version,

    pub dependencies: DependencySet,

    pub authors: Vec<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub bugs: Option<String>,
    // We should infer the repository.
    //pub repository: Option<String>,
    pub keywords: Vec<String>,

    pub license: Option<String>,
    pub license_files: Vec<PathBuf>,

    pub readme_contents: String,
    pub files: Vec<PathBuf>,
}
