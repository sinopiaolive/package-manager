use linked_hash_map::LinkedHashMap;
use std::string::String;
use version::Version;
use manifest::{PackageName, Manifest};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    pub packages: LinkedHashMap<PackageName, Package>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub releases: LinkedHashMap<Version, Release>,
    pub owners: Vec<Username>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
// we're using this for testing; can get rid of it later
#[serde(deny_unknown_fields)]
pub struct Release {
    pub artifact_url: String,
    pub manifest: Manifest,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Repository {
    pub repository_type: String,
    pub url: String,
}

pub type Username = String;

macro_rules! gen_registry {
    ( $( $name:ident => ( $( $release:expr => ( $( $dep:ident => $constraint:expr ),* ) ),* ) ),* ) => {{
        let mut packs = ::linked_hash_map::LinkedHashMap::new();
        $({
            let name = ::PackageName {
                namespace: Some("leftpad".to_string()), name: stringify!($name).to_string()
            };
            let mut releases = ::linked_hash_map::LinkedHashMap::new();
            $({
                let ver = ::Version::from_str($release).unwrap();
                let mut deps = ::linked_hash_map::LinkedHashMap::new();
                $({
                    let pkg = ::PackageName {
                        namespace: Some("leftpad".to_string()), name: stringify!($dep).to_string()
                    };
                    let constraint = ::VersionConstraint::from_str($constraint).unwrap();
                    deps.insert(pkg, constraint);
                })*;

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
                    dependencies: deps,
                };
                releases.insert(ver, ::Release {
                    manifest: manifest, artifact_url: "http://left-pad.com/left-pad.tar.gz".to_string()
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
