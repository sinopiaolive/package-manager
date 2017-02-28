#![allow(dead_code)]

#[macro_use] extern crate nom;
extern crate semver_parser;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate toml;
extern crate regex;

pub mod registry;
pub mod version;

use std::collections::HashMap;

macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
     };
);

pub fn test() {
    let r = registry::Repository {
        repository_type: "git".to_string(),
        url: "https://...".to_string() };
    println!("r = {:?}", r);
    println!("json = {}", serde_json::to_string(&r).unwrap());
    println!("\ntoml:\n{}", toml::to_string(&r).unwrap());
    let deserialized: registry::Repository = serde_json::from_str("{\"repository_type\":\"git\",\"url\":\"https://...\"}").unwrap();
    println!("deserialized = {:?}", deserialized);

    // TODO: implement VersionConstraint serialization
    println!("version_constraint:\n{}",
        toml::to_string(&registry::VersionConstraint::Exact(registry::Version {
            fields: vec![1, 0, 0],
            prerelease: vec![],
            build: vec![],
        })).unwrap());

    // let reg = registry::Registry {
    //     packages: map! {
    //         registry::PackageName { namespace: "rust".to_string(), name: "foo".to_string() }
    //           => registry::Package { owners: vec![], releases: HashMap::new() }
    //     }
    // };
    // println!("reg: {}", toml::to_string(&reg).unwrap());

    let test_registry_toml = r#"
[packages."rust/foo"]
owners = ["bodil"]

[packages."rust/foo".releases."1.0.0"]
dependencies = [
  { name = "rust/bar", version_constraint = "" }
]
    "#;
    let test_registry: registry::Registry = toml::from_str(test_registry_toml).unwrap();
    println!("test_registry: {:?}", test_registry)
}
