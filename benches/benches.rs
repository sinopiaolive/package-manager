#![feature(test)]

extern crate package_manager;
extern crate test;

use test::Bencher;
use std::path::Path;

use package_manager::index::read_index;

#[bench]
fn read_index_test(_b: &mut Bencher) {
    read_index(Path::new("test/cargo.rmp")).unwrap();
}
