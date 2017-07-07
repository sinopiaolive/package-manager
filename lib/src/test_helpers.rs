#![allow(unused_macros)]

use version::Version;
use manifest::PackageName;
use constraint::VersionConstraint;
use index::Index;


pub fn ver(s: &str) -> Version {
    Version::from_str(s).unwrap()
}

pub fn range(s: &str) -> VersionConstraint {
    VersionConstraint::from_str(s).unwrap()
}

pub fn pkg(s: &str) -> PackageName {
    let pkg = PackageName::from_str(s).unwrap();
    PackageName {
        namespace: Some(pkg.namespace.unwrap_or("test".to_string())),
        name: pkg.name,
    }
}

macro_rules! solution(
    { $($dep:ident => $version:expr),+ } => {
        {
            let mut m = ::im::map::Map::new();
            $(
                let version = $crate::version::Version::from_str($version).unwrap();
                m = m.insert(::std::sync::Arc::new($crate::test_helpers::pkg(stringify!($dep))),
                             ::std::sync::Arc::new(version));
            )+
            $crate::solver::Solution::wrap(m)
        }
     };
);

macro_rules! ver {
    ( $( $x:expr ),* ) => {{
        let mut version_parts = Vec::new();
        $(
            version_parts.push($x);
        )*;
        $crate::version::Version::new(version_parts, vec![], vec![])
    }};
}

macro_rules! deps {
    () => { ::std::collections::HashMap::new() };

    ( $( $dep:ident => $constraint:expr ),* ) => {{
        let mut deps = ::std::collections::HashMap::new();
        $({
            let constraint = $crate::constraint::VersionConstraint::from_str($constraint).unwrap();
            deps.insert(::test_helpers::pkg(stringify!($dep)), constraint);
        })*;
        deps
    }};
}

macro_rules! gen_registry {
    ( $( $name:ident => ( $( $release:expr => $deps:expr ),+ ) ),+ ) => {{
        let mut packs = ::std::collections::HashMap::new();
        $({
            let name = $crate::test_helpers::pkg(stringify!($name));
            let mut releases = ::std::collections::HashMap::new();
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
