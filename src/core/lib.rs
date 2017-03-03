#![allow(dead_code)]

#[macro_use]
extern crate nom;
extern crate semver_parser;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;
extern crate regex;

pub mod registry;
#[macro_use] pub mod version;
pub mod constraint;

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
    let test_registry_toml = r#"
[packages."rust/foo"]
owners = ["bodil"]

[packages."rust/foo".releases."1.0.0"]
dependencies = [
  { name = "rust/bar", version_constraint = "^1.2.0" }
]
artifact_url = "https://.../foo.tar"

[packages."rust/bar"]
owners = ["jo"]

[packages."rust/bar".releases."1.2.0"]
dependencies = [
  { name = "rust/baz", version_constraint = ">= 0.5.0" }
]
artifact_url = "https://.../bar.tar"

[packages."rust/baz"]
owners = ["jo"]

[packages."rust/baz".releases."0.5.0"]
dependencies = [
]
artifact_url = "https://.../baz.tar"
    "#;
    let test_registry: registry::Registry = toml::from_str(test_registry_toml).unwrap();
    println!("test_registry as JSON: {}", serde_json::to_string(&test_registry).unwrap());
}
