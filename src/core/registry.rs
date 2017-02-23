use std::iter::Map;

pub struct Registry {
    packages: Map<PackageName, Package>,
}

pub struct Package {
    //name: String
    releases: Map<Version, Release>,
    owners: Vec<Username>,
}

pub struct Release {
    //version: Version,
    dependencies: Vec<Dependency>,
    // no devDependencies here -- they only go in the manifest

    // TODO filesystem things
    artifactURL: String,

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
    versionConstraint: VersionConstraint,
}

pub enum VersionConstraint {
    Exact {
        version: Version
    },
    Range { // exclusive
        minimumVersion: Option<Version>,
        maximumVersion: Option<Version>
    }
}

pub struct PackageName {
    namespace: String,
    name: String
}

pub struct Version {
    // TODO validate
    fields: Vec<u64>,
    prerelease: Vec<VersionIdentifier>,
    build: Vec<VersionIdentifier>
}

pub enum VersionIdentifier {
    Numeric(u64),
    AlphaNumeric(String)
}

pub struct Repository {
    repoType: String,
    url: String,
}

pub type Username = String;
