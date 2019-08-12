use std::sync::Arc;
use solver::{Path, Constraint, ConstraintSet, JustifiedVersion, PartialSolution};
use pm_lib::test_helpers::{pkg, ver};
use pm_lib::index::Index;

macro_rules! solution(
    { $($dep:ident => $version:expr),+ } => {
        {
            let mut m = ::std::collections::BTreeMap::new();
            $(
                let version = ::pm_lib::version::Version::from_str($version).unwrap();
                m.insert(::pm_lib::test_helpers::pkg(stringify!($dep)),
                             version);
            )+
            $crate::solver::Solution(m)
        }
     };
);

macro_rules! deps {
    () => { ::std::collections::BTreeMap::new() };

    ( $( $dep:ident => $constraint:expr ),* ) => {{
        let mut deps = ::std::collections::BTreeMap::new();
        $({
            let constraint = ::pm_lib::constraint::VersionConstraint::from_str($constraint).unwrap();
            deps.insert(::pm_lib::test_helpers::pkg(stringify!($dep)), constraint);
        })*;
        deps
    }};
}

macro_rules! gen_registry {
    ( $( $name:ident => ( $( $release:expr => $deps:expr ),+ ) ),+ ) => {{
        let mut packs = ::pm_lib::index::Index::new();
        $({
            let name = ::pm_lib::test_helpers::pkg(stringify!($name));
            let mut releases = ::pm_lib::index::Package::new();
            $({
                let ver = ::pm_lib::version::Version::from_str($release).unwrap();

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
