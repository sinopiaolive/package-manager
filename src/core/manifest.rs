use serde::{Serialize, Serializer, Deserialize, Deserializer};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use constraint::VersionConstraint;
use version::Version;
use std::fmt::Display;
use std::env;
use std::fs::File;
use std::io::Read;
use toml;
use super::error::Error;

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)] // we're using this for testing; can get rid of it later
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub name: PackageName,
    pub description: String,
    pub author: String,
    pub license: String,
    pub license_file: String,
    pub homepage: Option<String>,
    pub bugs: Option<String>,
    pub repository: Option<String>,
    pub keywords: Vec<String>,
    pub files: Option<Vec<String>>,
    pub private: bool,
    pub dependencies: DependencySet,
    pub dev_dependencies: DependencySet,
}

pub type DependencySet = HashMap<PackageName, VersionConstraint>;

pub type VersionSet = HashMap<PackageName, Version>;

#[derive(Debug, PartialEq, Eq, Hash, Default)]
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

fn find_manifest(path: &Path) -> Option<PathBuf> {
    let manifest = path.join("manifest.toml");
    if manifest.exists() {
        Some(manifest)
    } else {
        path.parent().and_then(|p| find_manifest(p))
    }
}

pub fn find_manifest_path() -> Result<PathBuf, Error> {
    let cwd = env::current_dir()?;
    find_manifest(&cwd).ok_or(Error::Message("no project file found!"))
}

pub fn find_project_dir() -> Result<PathBuf, Error> {
    let mut manifest_path = find_manifest_path()?;
    manifest_path.pop();
    Ok(manifest_path)
}

pub fn read_manifest() -> Result<Manifest, Error> {
    let manifest_path = find_manifest_path()?;
    let data = File::open(manifest_path).and_then(|mut f| {
        let mut s = String::new();
        f.read_to_string(&mut s).map(|_| s)
    })?;
    Ok(toml::from_str(&data)?)
}
