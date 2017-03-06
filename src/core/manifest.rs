use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::Error;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use constraint::VersionConstraint;
use version::Version;
use std::fmt::Display;
use std::env;
use std::fs::File;
use std::io::Read;
use toml;
use super::error;

fn is_false(a: &bool) -> bool {
    !*a
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub name: PackageName,
    pub description: String,
    pub author: String,
    pub license: Option<String>,
    pub license_file: Option<String>,
    pub homepage: Option<String>,
    pub bugs: Option<String>,
    pub repository: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")] pub keywords: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")] pub files: Vec<String>,
    #[serde(default, skip_serializing_if = "is_false")] pub private: bool,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")] pub dependencies: DependencySet,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")] pub dev_dependencies: DependencySet,
}

impl Manifest {
    pub fn to_string(&self) -> String {
        toml::ser::to_string(self).unwrap()
    }
}

pub type DependencySet = HashMap<PackageName, VersionConstraint>;

pub type VersionSet = HashMap<PackageName, Version>;

#[derive(Debug, PartialEq, Eq, Hash, Default)]
pub struct PackageName {
    pub namespace: Option<String>,
    pub name: String,
}

impl Display for PackageName {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        match &self.namespace {
            &Some(ref namespace) => write!(f, "{}/{}", namespace, self.name),
            &None => write!(f, "{}", self.name)
        }
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
        match v.len() {
            1 => Ok(PackageName {
                namespace: None,
                name: v[0].to_string()
            }),
            2 => Ok(PackageName {
                namespace: Some(v[0].to_string()),
                name: v[1].to_string()
            }),
            _ => Err(D::Error::custom(format!("Wrong number of components (1 or 2 allowed): {:?}", s)))
        }
    }
}

fn find_manifest(path: &Path) -> Option<PathBuf> {
    let manifest = path.join("manifest.toml");
    if manifest.exists() {
        Some(manifest)
    } else {
        path.parent().and_then(|p| find_manifest(p))
    }
}

pub fn find_manifest_path() -> Result<PathBuf, error::Error> {
    let cwd = env::current_dir()?;
    find_manifest(&cwd).ok_or(error::Error::Message("no project file found!"))
}

pub fn find_project_dir() -> Result<PathBuf, error::Error> {
    let mut manifest_path = find_manifest_path()?;
    manifest_path.pop();
    Ok(manifest_path)
}

pub fn read_manifest() -> Result<Manifest, error::Error> {
    let manifest_path = find_manifest_path()?;
    let data = File::open(manifest_path).and_then(|mut f| {
        let mut s = String::new();
        f.read_to_string(&mut s).map(|_| s)
    })?;
    Ok(toml::from_str(&data)?)
}
