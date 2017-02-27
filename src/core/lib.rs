#![allow(dead_code)]

#[macro_use]
extern crate serde_derive;

extern crate serde_json;

pub fn test() {
    let r = registry::Repository {
        repository_type: "git".to_string(),
        url: "https://...".to_string() };
    println!("r = {:?}", r);
    println!("json = {}", serde_json::to_string(&r).unwrap());
    let deserialized: registry::Repository = serde_json::from_str("{\"repository_type\":\"git\",\"url\":\"https://...\"}").unwrap();
    println!("deserialized = {:?}", deserialized);
}

pub mod registry;
