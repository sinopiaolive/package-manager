use std::collections::HashMap;
use std::string::String;
use std::iter::Iterator;
use std::fmt::Display;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::Error;
use version::version;
use nom::IResult;

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

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum VersionConstraint {
    Exact(Version),
    Range { // exclusive
        minimum_version: Option<Version>,
        maximum_version: Option<Version>
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct PackageName {
    pub namespace: String,
    pub name: String
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
        Ok(PackageName { namespace: v[0].to_string(), name: v[1].to_string() })
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Version {
    pub fields: Vec<u64>,
    pub prerelease: Vec<VersionIdentifier>,
    pub build: Vec<VersionIdentifier> // TODO Vec?
}

impl Version {
    pub fn new(v: Vec<u64>, p: Vec<VersionIdentifier>, b: Vec<VersionIdentifier>) -> Version {
        Version { fields: v, prerelease: p, build: b }
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut s = String::new();
        s.push_str(&*self.fields.iter().map(|f| f.to_string()).collect::<Vec<_>>().join("."));
        if self.prerelease.len() > 0 {
            s.push_str("-");
            s.push_str(&*self.prerelease.iter().map(|f| f.to_string()).collect::<Vec<_>>().join("."));
        }
        if self.build.len() > 0 {
            s.push_str("+");
            s.push_str(&*self.build.iter().map(|f| f.to_string()).collect::<Vec<_>>().join("."));
        }
        serializer.serialize_str(&*s)
    }
}

impl Deserialize for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        let s = String::deserialize(deserializer)?;
        match version(s.as_bytes()) {
            IResult::Done(r, v) => {
                if r == &b""[..] {
                    Ok(v)
                } else {
                    Err(D::Error::custom(format!("{:?} is not a valid version descriptor", s)))
                }
            }
            _ => {
                Err(D::Error::custom(format!("{:?} is not a valid version descriptor", s)))
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub enum VersionIdentifier {
    Numeric(u64),
    Alphanumeric(String)
}

impl Display for VersionIdentifier {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        match *self {
            VersionIdentifier::Numeric(ref n) => write!(f, "{}", n),
            VersionIdentifier::Alphanumeric(ref s) => write!(f, "{}", s)
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Repository {
    pub repository_type: String,
    pub url: String,
}

pub type Username = String;
