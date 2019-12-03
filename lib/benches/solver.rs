#![feature(test)]

#[macro_use]
extern crate pm_lib;
extern crate test;

use test::Bencher;

use pm_lib::index::read_index;
use pm_lib::solver::test_helpers::path;
use pm_lib::solver::{Conflict, Error};
use pm_lib::test_helpers::{pkg, range};
use std::sync::Arc;

use pm_lib::solver::*;

#[bench]
fn resolve_something_real(b: &mut Bencher) {
    let reg = read_index(::std::path::Path::new("../test/cargo.rmp")).unwrap();

    let problem = deps! {
        tokio_proto => "<1",
        hyper => "^0.11",
        url => "^1"
    };

    b.iter(|| {
        assert_eq!(
            solve(&reg, &problem),
            Ok(solution! {
                base64 => "0.6.0",
                byteorder => "1.1.0",
                bytes => "0.4.4",
                cfg_if => "0.1.2",
                futures => "0.1.14",
                futures_cpupool => "0.1.5",
                httparse => "1.2.3",
                hyper => "0.11.1",
                idna => "0.1.4",
                iovec => "0.1.0",
                kernel32_sys => "0.2.2",
                language_tags => "0.2.2",
                lazycell => "0.4.0",
                libc => "0.2.26",
                log => "0.3.8",
                matches => "0.1.6",
                mime => "0.3.2",
                mio => "0.6.9",
                miow => "0.2.1",
                net2 => "0.2.29",
                num_cpus => "1.6.2",
                percent_encoding => "1.0.0",
                rand => "0.3.15",
                redox_syscall => "0.1.26",
                safemem => "0.2.0",
                scoped_tls => "0.1.0",
                slab => "0.3.0",
                smallvec => "0.2.1",
                take => "0.1.0",
                time => "0.1.38",
                tokio_core => "0.1.8",
                tokio_io => "0.1.2",
                tokio_proto => "0.1.1",
                tokio_service => "0.1.0",
                unicase => "2.0.0",
                unicode_bidi => "0.3.4",
                unicode_normalization => "0.1.5",
                url => "1.5.1",
                winapi => "0.2.8",
                ws2_32_sys => "0.2.1"
            })
        );
    });
}

#[bench]
fn deep_conflict(b: &mut Bencher) {
    let reg = read_index(::std::path::Path::new("../test/cargo.rmp")).unwrap();

    let problem = deps! {
        rocket => "^0.2.8",
        hyper_rustls => "^0.8"
    };

    b.iter(|| {
        assert_eq!(
            solve(&reg, &problem),
            Err(Error::Conflict(Box::new(Conflict {
                package: Arc::new(pkg("hyper")),
                existing: range("^0.11"),
                existing_path: path(&[("hyper_rustls", "0.8.0")]),
                conflicting: range("^0.10.4"),
                conflicting_path: path(&[("rocket", "0.2.9")]),
            })))
        );
    });
}
