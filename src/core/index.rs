use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use rmp_serde::{self, encode};
use serde_json;

use error::Error;
use version::Version;
use constraint::VersionConstraint;
use manifest::PackageName;
use path;



pub type Index = HashMap<PackageName, Package>;
pub type Package = HashMap<Version, Dependencies>;
pub type Dependencies = HashMap<PackageName, VersionConstraint>;

pub fn read_index(path: &Path) -> Result<Arc<Index>, Error> {
    let mut f = File::open(path)?;
    let mut s = Vec::new();
    f.read_to_end(&mut s)?;
    Ok(Arc::new(rmp_serde::from_slice(&s)?))
}

pub fn read_default() -> Result<Arc<Index>, Error> {
    read_index(&path::config_path()?.as_path().join("index.rmp"))
}

pub fn write_to<W>(i: &Index, wr: &mut W) -> Result<(), Error>
where
    W: Write,
{
    Ok(encode::write(wr, i)?)
}

pub fn write_index(i: &Index, path: &Path) -> Result<(), Error> {
    fs::create_dir_all(path)?;
    let mut f = File::create(path)?;
    write_to(i, &mut f)
}

pub fn read_json<P>(path: P) -> Result<Arc<Index>, Error>
where
    P: AsRef<Path>,
{
    let mut f = File::open(path)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    serde_json::from_str(&s)
        .map_err(|e| Error::Custom(format!("{}", e)))
        .map(|v| Arc::new(v))
}
