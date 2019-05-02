#![warn(clippy::all)]
#![allow(dead_code, unused_features)]
#![feature(test)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate rmp_serde;
extern crate serde_json;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate nom;
#[cfg(test)]
extern crate test;

#[macro_use]
pub mod test_helpers;
pub mod version;
pub mod constraint;
pub mod package;
pub mod dependencies;
pub mod index;
pub mod publication_request;
