#![allow(dead_code)]

#[macro_use] extern crate nom;
extern crate semver_parser;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate toml;
extern crate regex;
#[macro_use] extern crate quick_error;
extern crate linked_hash_map;
extern crate license_exprs;

#[macro_use] mod test;
mod lazy;
mod registry;
pub use registry::*;
mod version;
pub use version::*;
mod constraint;
pub use constraint::*;
pub mod manifest;
pub use manifest::*;
pub mod error;
mod solver;
pub use solver::*;
mod lockfile;
pub use lockfile::*;

macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::linked_hash_map::LinkedHashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
     };
);

pub fn test() {
    let test_registry_toml = r#"
[packages."rust/foo"]
owners = ["bodil"]

[packages."rust/foo".releases."1.0.0"]
artifact_url = "https://.../foo.tar"
[packages."rust/foo".releases."1.0.0".manifest]
dependencies = { "rust/bar" = "^1.2.0" }

[packages."rust/bar"]
owners = ["jo"]

[packages."rust/bar".releases."1.2.0"]
artifact_url = "https://.../bar.tar"
[packages."rust/bar".releases."1.2.0".manifest]
dependencies = { "rust/baz" = ">= 0.5.0" }

[packages."rust/baz"]
owners = ["jo"]

[packages."rust/baz".releases."0.5.0"]
artifact_url = "https://.../baz.tar"
    "#;
    let test_registry: registry::Registry = toml::from_str(test_registry_toml).unwrap();
    println!("test_registry as JSON: {}", serde_json::to_string(&test_registry).unwrap());
}
