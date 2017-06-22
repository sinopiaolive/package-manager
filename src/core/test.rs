#![allow(unused_macros)]

use std::collections::HashMap;
use linked_hash_map::LinkedHashMap;
use std::hash::Hash;

use version::Version;
use manifest::PackageName;
use constraint::VersionConstraint;
use registry::Registry;


pub fn unlink<A, B>(linked: &LinkedHashMap<A, B>) -> HashMap<A, B>
where
    A: Eq + Hash + Clone,
    B: Clone,
{
    let mut map = HashMap::new();
    for (key, value) in linked.iter() {
        map.insert(key.clone(), value.clone());
    }
    map
}

pub fn ver(s: &str) -> Version {
    Version::from_str(s).unwrap()
}

pub fn range(s: &str) -> VersionConstraint {
    VersionConstraint::from_str(s).unwrap()
}

pub fn pkg(s: &str) -> PackageName {
    let pkg = PackageName::from_str(s).unwrap();
    PackageName {
        namespace: Some(pkg.namespace.unwrap_or("test".to_string())),
        name: pkg.name,
    }
}

macro_rules! solution(
    { $($dep:ident => $version:expr),+ } => {
        {
            let mut m = ::immutable_map::map::TreeMap::new();
            $(
                let version = ::Version::from_str($version).unwrap();
                m = m.insert(::std::sync::Arc::new(::test::pkg(stringify!($dep))),
                             ::std::sync::Arc::new(version));
            )+
            ::solver::Solution::wrap(m)
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
            let constraint = ::VersionConstraint::from_str($constraint).unwrap();
            deps.insert(::test::pkg(stringify!($dep)), constraint);
        })*;
        deps
    }};
}

macro_rules! gen_registry {
    ( $( $name:ident => ( $( $release:expr => $deps:expr ),+ ) ),+ ) => {{
        let mut packs = ::std::collections::HashMap::new();
        $({
            let name = ::test::pkg(stringify!($name));
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

pub fn sample_registry() -> Registry {
    gen_registry!(
        left_pad => (
            "1.0.0" => deps!(
                right_pad => "^1.0.0"
            ),
            "2.0.0" => deps!(
                right_pad => "^2.0.0"
            )
        ),
        lol_pad => (
            "1.0.0" => deps!(
                right_pad => "^2.0.0"
            )
        ),
        right_pad => (
            "1.0.0" => deps!(
                up_pad => "^1.0.0"
            ),
            "1.0.1" => deps!(
                up_pad => "^1.0.0"
            ),
            "2.0.0" => deps!(
                up_pad => "^2.0.0"
            ),
            "2.0.1" => deps!(
                up_pad => "^2.0.0",
                coleft_copad => "^2.0.0"
            )
        ),
        up_pad => (
            "1.0.0" => deps!(),
            "2.0.0" => deps!(),
            "2.1.0" => deps!(
                coleft_copad => "^1.0.0"
            )
        ),
        coleft_copad => (
            "1.0.0" => deps!(),
            "1.0.1" => deps!(),
            "1.1.0" => deps!(),
            "2.0.0" => deps!()
        ),
        down_pad => (
            "1.0.0" => deps!(),
            "1.2.0" => deps!()
        )
    )
}
