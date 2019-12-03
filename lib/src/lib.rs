#![warn(clippy::all)]
#![allow(dead_code, unused_features)]

extern crate im_rc as im;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate nom;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;

#[macro_use]
pub mod test_helpers;
pub mod constraint;
pub mod dependencies;
pub mod index;
pub mod package;
pub mod publication_request;
#[macro_use]
pub mod solver;
pub mod version;
