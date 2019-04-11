use version::Version;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NamedTextFile {
    pub name: String,
    pub contents: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Repository {
    pub type_: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Dependency {
    pub namespace: String,
    pub name: String,
    pub version_constraint: String,
}

/// Manifest structure used for publishing packages to the registry API.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    pub namespace: String,
    pub name: String,
    pub version: Version,

    pub description: String,
    pub authors: Vec<String>,
    pub keywords: Vec<String>,
    pub homepage_url: Option<String>,
    pub repository: Option<Repository>,
    pub bugs_url: Option<String>,
    pub license: Option<String>,
    pub license_file: Option<NamedTextFile>,
    pub manifest: Option<NamedTextFile>,
    pub readme: Option<NamedTextFile>,

    pub dependencies: Vec<Dependency>,
    pub tar_br: Vec<u8>,
}
