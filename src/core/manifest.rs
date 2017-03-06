use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::Error;
use std::path::{Path, PathBuf};
use constraint::VersionConstraint;
use version::Version;
use std::fmt::Display;
use std::env;
use std::fs::File;
use std::io::Read;
use std::iter::FromIterator;
use std::clone::Clone;
use toml;
use linked_hash_map::LinkedHashMap;
use super::error;

fn is_false(a: &bool) -> bool {
    !*a
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
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
    #[serde(default, skip_serializing_if = "LinkedHashMap::is_empty")] pub dependencies: DependencySet,
    #[serde(default, skip_serializing_if = "LinkedHashMap::is_empty")] pub dev_dependencies: DependencySet,
}

impl Manifest {
    pub fn to_string(&self) -> Result<String, error::Error> {
        Ok(toml::ser::to_string(self)?)
    }
}

pub type DependencySet = LinkedHashMap<PackageName, VersionConstraint>;

pub type VersionSet = LinkedHashMap<PackageName, Version>;

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone)]
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

fn normalise_dep(path: &String, dep: &PackageName) -> PackageName {
    PackageName {
        namespace: Some(dep.namespace.clone().unwrap_or((*path).clone())),
        name: dep.name.clone()
    }
}

fn denormalise_dep(path: &String, dep: &PackageName) -> PackageName {
    match dep.namespace {
        Some(ref ns) => PackageName {
            namespace: if ns == path { None } else { Some((*ns).clone()) },
            name: dep.name.clone()
        },
        None => dep.clone()
    }
}

fn normalise_deps(path: &String, deps: &DependencySet) -> DependencySet {
    DependencySet::from_iter(deps.into_iter().map(|(k, v)| (normalise_dep(path, k), (*v).clone())))
}

fn denormalise_deps(path: &String, deps: &DependencySet) -> DependencySet {
    DependencySet::from_iter(deps.into_iter().map(|(k, v)| (denormalise_dep(path, k), (*v).clone())))
}

pub fn normalise_manifest(manifest: &Manifest) -> Result<Manifest, error::Error> {
    let path = manifest.name.clone().namespace.ok_or(error::Error::Message("Package name must contain a namespace!"))?;
    let deps = normalise_deps(&path, &manifest.dependencies);
    let dev_deps = normalise_deps(&path, &manifest.dev_dependencies);
    let mut m = (*manifest).clone();
    m.dependencies = deps;
    m.dev_dependencies = dev_deps;
    Ok(m)
}

pub fn denormalise_manifest(manifest: &Manifest) -> Result<Manifest, error::Error> {
    let path = manifest.name.clone().namespace.ok_or(error::Error::Message("Package name must contain a namespace!"))?;
    let deps = denormalise_deps(&path, &manifest.dependencies);
    let dev_deps = denormalise_deps(&path, &manifest.dev_dependencies);
    let mut m = (*manifest).clone();
    m.dependencies = deps;
    m.dev_dependencies = dev_deps;
    Ok(m)
}

pub fn serialise_manifest(manifest: &Manifest) -> Result<String, error::Error> {
    denormalise_manifest(manifest)?.to_string()
}

pub fn deserialise_manifest(data: &String) -> Result<Manifest, error::Error> {
    Ok(normalise_manifest(&toml::from_str(data)?)?)
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
    deserialise_manifest(&data)
}
