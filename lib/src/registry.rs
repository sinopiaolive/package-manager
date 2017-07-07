use std::collections::HashMap;
use std::string::String;
use version::Version;
use manifest::{PackageName, Manifest};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    pub packages: HashMap<PackageName, Package>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub releases: HashMap<Version, Release>,
    pub owners: Vec<Username>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
// we're using this for testing; can get rid of it later
#[serde(deny_unknown_fields)]
pub struct Release {
    pub artifact_url: String,
    pub manifest: Manifest,
}

pub type Username = String;
