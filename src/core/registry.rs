use std::collections::HashMap;

pub struct Registry {
    packages: HashMap<PackageName, Package>,
}

pub struct Package {
    releases: HashMap<Version, Release>,
    owners: Vec<Username>,
}

pub struct Release {
    dependencies: Vec<Dependency>,
    // no devDependencies here -- they only go in the manifest

    // TODO filesystem things
    artifact_url: String,

    description: String,
    author: String,
    license: String,
    license_file: String, // TODO filesystem things
    homepage: String,
    bugs: String,
    repository: Repository,
    keywords: Vec<String>,
}

pub struct Dependency {
    name: PackageName,
    version_constraint: VersionConstraint,
}

pub enum VersionConstraint {
    Exact {
        version: Version
    },
    Range { // exclusive
        minimum_version: Option<Version>,
        maximum_version: Option<Version>
    }
}

pub struct PackageName {
    namespace: String,
    name: String
}

pub struct Version {
    fields: Vec<u64>,
    prerelease: Vec<VersionIdentifier>,
    build: Vec<VersionIdentifier>
}

pub enum VersionIdentifier {
    Numeric(u64),
    AlphaNumeric(String)
}

pub struct Repository {
    repository_type: String,
    url: String,
}

pub type Username = String;
