use version::Version;
use manifest::PackageName;
use constraint::VersionConstraint;
use std::collections::HashMap;
use linked_hash_map::LinkedHashMap;
use hamt_rs::HamtMap;
use std::hash::Hash;

pub fn to_mut<A, B>(hamt: &HamtMap<A, B>) -> HashMap<A, B>
    where A: Eq + Hash + Clone + Sync + Send, B: Clone + Sync + Send
{
    let mut map = HashMap::new();
    for (key, value) in hamt.iter() {
        map.insert(key.clone(), value.clone());
    }
    map
}

pub fn unlink<A, B>(linked: &LinkedHashMap<A, B>) -> HashMap<A, B>
    where A: Eq + Hash + Clone, B: Clone
{
    let mut map = HashMap::new();
    for (key, value) in linked.iter() {
        map.insert(key.clone(), value.clone());
    }
    map
}

pub fn to_im<A, B>(linked: &LinkedHashMap<A, B>) -> HamtMap<A, B>
    where A: Eq + Hash + Clone + Sync + Send, B: Clone + Sync + Send
{
    let mut hamt = HamtMap::new();
    for (key, value) in linked.iter() {
        hamt = hamt.plus(key.clone(), value.clone());
    }
    hamt
}

pub fn ver(s: &str) -> Version {
    Version::from_str(s).unwrap()
}

pub fn range(s: &str) -> VersionConstraint {
    VersionConstraint::from_str(s).unwrap()
}

pub fn pkg(s: &str) -> PackageName {
    PackageName::from_str(s).unwrap()
}

macro_rules! dict {
    () => { ::hamt_rs::HamtMap::new() };

    ( $( $key:expr => $value:expr ),* ) => {{
        let mut map = ::hamt_rs::HamtMap::new();
        $({
            map = map.plus($key, $value);
        })*;
        map
    }};
}

macro_rules! solution(
    { $($dep:ident => $version:expr),+ } => {
        {
            let mut m = ::hamt_rs::HamtMap::new();
            $(
                let pkg = ::PackageName {
                    namespace: Some("leftpad".to_string()), name: stringify!($dep).to_string()
                };
                let version = ::Version::from_str($version).unwrap();
                m = m.plus(pkg, version);
            )+
            ::solver::Solution::Solution(m)
        }
     };
);

macro_rules! ver {
    ( $( $x:expr ),* ) => {{
        let mut version_parts = Vec::new();
        $(
            version_parts.push($x);
        )*;
        ::Version::new(version_parts, vec![], vec![])
    }};
}

macro_rules! deps {
    () => { ::linked_hash_map::LinkedHashMap::new() };

    ( $( $dep:ident => $constraint:expr ),* ) => {{
        let mut deps = ::linked_hash_map::LinkedHashMap::new();
        $({
            let pkg = ::PackageName {
                namespace: Some("leftpad".to_string()), name: stringify!($dep).to_string()
            };
            let constraint = ::VersionConstraint::from_str($constraint).unwrap();
            deps.insert(pkg, constraint);
        })*;
        deps
    }};
}

macro_rules! gen_registry {
    ( $( $name:ident => ( $( $release:expr => $deps:expr ),+ ) ),+ ) => {{
        let mut packs = ::std::collections::HashMap::new();
        $({
            let name = ::PackageName {
                namespace: Some("leftpad".to_string()), name: stringify!($name).to_string()
            };
            let mut releases = ::std::collections::HashMap::new();
            $({
                let ver = ::Version::from_str($release).unwrap();

                let manifest = ::Manifest {
                    name: name.clone(),
                    description: "A generalised sinister spatiomorphism.".to_string(),
                    author: "IEEE Text Alignment Working Group".to_string(),
                    license: None,
                    license_file: None,
                    homepage: None,
                    bugs: None,
                    repository: None,
                    keywords: vec![],
                    files: vec![],
                    private: false,
                    dev_dependencies: ::linked_hash_map::LinkedHashMap::new(),
                    dependencies: $deps,
                };
                releases.insert(ver, ::Release {
                    manifest: manifest,
                    artifact_url: "http://left-pad.com/left-pad.tar.gz".to_string()
                });
            })*;
            let pack = ::Package {
                owners: vec!["Left Pad Working Group".to_string()],
                releases: releases
            };
            packs.insert(name, pack);
        })*;
        ::Registry { packages: packs }
    }}
}
