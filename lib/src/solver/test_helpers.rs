use crate::index::Index;
use crate::test_helpers::{pkg, ver};
use solver::{Constraint, ConstraintSet, JustifiedVersion, PartialSolution, Path};
use std::sync::Arc;

#[macro_export]
macro_rules! solution(
    { $($dep:ident => $version:expr),+ } => {
        {
            let mut m = ::std::collections::BTreeMap::new();
            $(
                let version = $crate::version::Version::from_str($version).unwrap();
                m.insert($crate::test_helpers::pkg(stringify!($dep)),
                             version);
            )+
            m
        }
     };
);

#[macro_export]
macro_rules! deps {
    () => { ::std::collections::BTreeMap::new() };

    ( $( $dep:ident => $constraint:expr ),* ) => {{
        let mut deps = ::std::collections::BTreeMap::new();
        $({
            let constraint = $crate::constraint::VersionConstraint::from_str($constraint).unwrap();
            deps.insert($crate::test_helpers::pkg(stringify!($dep)), constraint);
        })*;
        deps
    }};
}

#[macro_export]
macro_rules! gen_registry {
    ( $( $name:ident => ( $( $release:expr => $deps:expr ),+ ) ),+ ) => {{
        let mut packs = $crate::index::Index::new();
        $({
            let name = $crate::test_helpers::pkg(stringify!($name));
            let mut releases = $crate::index::Package::new();
            $({
                let ver = $crate::version::Version::from_str($release).unwrap();

                releases.insert(ver, $deps);
            })*;
            packs.insert(name, releases);
        })*;
        packs
    }}
}

pub fn sample_registry() -> Index {
    gen_registry!(
        left_pad => (
            "1.0.0" => deps!(
                right_pad => "^1.0.0"
            ),
            "2.0.0" => deps!(
                right_pad => "^2.0.0"
            )
        ),
        lol_pad => (
            "1.0.0" => deps!(
                right_pad => "^2.0.0"
            )
        ),
        right_pad => (
            "1.0.0" => deps!(
                up_pad => "^1.0.0"
            ),
            "1.0.1" => deps!(
                up_pad => "^1.0.0"
            ),
            "2.0.0" => deps!(
                up_pad => "^2.0.0"
            ),
            "2.0.1" => deps!(
                up_pad => "^2.0.0",
                coleft_copad => "^2.0.0"
            )
        ),
        up_pad => (
            "1.0.0" => deps!(),
            "2.0.0" => deps!(),
            "2.1.0" => deps!(
                coleft_copad => "^1.0.0"
            )
        ),
        coleft_copad => (
            "1.0.0" => deps!(),
            "1.0.1" => deps!(),
            "1.1.0" => deps!(),
            "2.0.0" => deps!()
        ),
        down_pad => (
            "1.0.0" => deps!(),
            "1.2.0" => deps!()
        )
    )
}

pub fn path(l: &[(&str, &str)]) -> Path {
    Path::from_vec(
        l.iter()
            .map(|&(p, v)| (Arc::new(pkg(p)), Arc::new(ver(v))))
            .collect(),
    )
}

pub fn constraint(l: &[(&str, &[(&str, &str)])]) -> Constraint {
    Constraint(l.iter().map(|&(v, pa)| (ver(v), path(pa))).collect())
}

pub fn constraint_set(l: &[(&str, &[(&str, &[(&str, &str)])])]) -> ConstraintSet {
    ConstraintSet(l.iter().map(|&(p, c)| (pkg(p), constraint(c))).collect())
}

pub fn jver(l: (&str, &[(&str, &str)])) -> JustifiedVersion {
    let (v, pa) = l;
    JustifiedVersion {
        version: Arc::new(ver(v)),
        path: path(pa),
    }
}

pub fn partial_sln(l: &[(&str, (&str, &[(&str, &str)]))]) -> PartialSolution {
    PartialSolution(l.iter().map(|&(p, jv)| (pkg(p), jver(jv))).collect())
}
