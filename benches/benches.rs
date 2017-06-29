#![feature(test)]

extern crate package_manager;
extern crate test;

use test::Bencher;
use std::path::Path;

use package_manager::index::read_index;

#[bench]
fn read_index_bench(_b: &mut Bencher) {
    // There doesn't seem to be a way to run a function once and still get
    // timings as with b.iter().
    read_index(Path::new("test/cargo.rmp")).unwrap();
}
