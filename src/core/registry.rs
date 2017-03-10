use linked_hash_map::LinkedHashMap;
use std::string::String;
use version::Version;
use manifest::{PackageName, Manifest};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    pub packages: LinkedHashMap<PackageName, Package>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub releases: LinkedHashMap<Version, Release>,
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Repository {
    pub repository_type: String,
    pub url: String,
}

pub type Username = String;
