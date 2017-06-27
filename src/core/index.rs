use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use rmp_serde::decode;

use std::io::prelude::*;

use version::Version;
use constraint::VersionConstraint;
use manifest::PackageName;



#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Index {
    pub packages: HashMap<PackageName, Package>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub releases: HashMap<Version, Dependencies>,
}

pub type Dependencies = HashMap<PackageName, VersionConstraint>;

impl Index {
    pub fn from_read<R>(rd: R) -> Result<Index, decode::Error> where R: Read {
        decode::from_read(rd)
    }

    // pub fn read(path: &Path) -> Arc<Index> {

    // }
}
