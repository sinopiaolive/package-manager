use std::collections::HashMap;
use std::string::String;
use std::iter::Iterator;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use version::Version;
use constraint::VersionConstraint;
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    pub packages: HashMap<PackageName, Package>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub releases: HashMap<Version, Release>,
    pub owners: Vec<Username>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct Dependency {
    pub name: PackageName,
    pub version_constraint: VersionConstraint,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct PackageName {
    pub namespace: String,
    pub name: String,
}

impl Display for PackageName {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "{}/{}", self.namespace, self.name)
    }
}

impl Serialize for PackageName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(&*self.to_string())
    }
}

impl Deserialize for PackageName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        let s = String::deserialize(deserializer)?;
        let v: Vec<&str> = s.split('/').collect();
        Ok(PackageName {
            namespace: v[0].to_string(),
            name: v[1].to_string(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Repository {
    pub repository_type: String,
    pub url: String,
}

pub type Username = String;
