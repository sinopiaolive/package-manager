#![allow(unused_macros)]

use version::Version;
use package::PackageName;
use constraint::VersionConstraint;



pub fn ver(s: &str) -> Version {
    Version::from_str(s).unwrap()
}

pub fn range(s: &str) -> VersionConstraint {
    VersionConstraint::from_str(s).unwrap()
}

pub fn pkg(s: &str) -> PackageName {
    let segments = s.split("/").count();
    let pkg = if segments == 1 {
        PackageName::from_str(&format!("test/{}", s))
    } else {
        PackageName::from_str(s)
    }.unwrap();
    PackageName {
        namespace: pkg.namespace,
        name: pkg.name,
    }
}

macro_rules! ver {
    ( $( $x:expr ),* ) => {{
        let mut version_parts = Vec::new();
        $(
            version_parts.push($x);
        )*;
        $crate::version::Version::new(version_parts, vec![], vec![])
    }};
}
