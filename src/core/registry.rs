use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Registry {
    pub packages: HashMap<PackageName, Package>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Package {
    pub releases: HashMap<Version, Release>,
    pub owners: Vec<Username>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Release {
    pub dependencies: Vec<Dependency>,
    // no devDependencies here -- they only go in the manifest

    // TODO filesystem things
    pub artifact_url: String,

    pub description: String,
    pub author: String,
    pub license: String,
    pub license_file: String, // TODO filesystem things
    pub homepage: String,
    pub bugs: String,
    pub repository: Repository,
    pub keywords: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Dependency {
    pub name: PackageName,
    pub version_constraint: VersionConstraint,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum VersionConstraint {
    Exact(Version),
    Range { // exclusive
        minimum_version: Option<Version>,
        maximum_version: Option<Version>
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct PackageName {
    pub namespace: String,
    pub name: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct Version {
    pub fields: Vec<u64>,
    pub prerelease: Vec<VersionIdentifier>,
    pub build: Vec<VersionIdentifier>
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum VersionIdentifier {
    Numeric(u64),
    AlphaNumeric(String)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Repository {
    pub repository_type: String,
    pub url: String,
}

pub type Username = String;
