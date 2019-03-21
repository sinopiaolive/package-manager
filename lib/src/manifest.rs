use version::Version;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum License {
    SPDX(String),
    File(String),
    SPDXAndFile(String, String)
}

/// Manifest structure used for publishing packages to the registry API.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    pub namespace: String,
    pub name: String,
    pub version: Version,
    pub description: String,
    pub license: License,
    pub keywords: Vec<String>,
    pub manifest: String,
    pub readme: Option<(String, String)>,
    pub tar_br: Vec<u8>,
}
