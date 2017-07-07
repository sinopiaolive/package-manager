#![allow(unused_macros)]

use version::Version;
use manifest::PackageName;
use constraint::VersionConstraint;



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

macro_rules! ver {
    ( $( $x:expr ),* ) => {{
        let mut version_parts = Vec::new();
        $(
            version_parts.push($x);
        )*;
        $crate::version::Version::new(version_parts, vec![], vec![])
    }};
}
