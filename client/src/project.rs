#![allow(dead_code)]

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use error::Error;
use manifest::Manifest;

fn find_manifest(path: &Path) -> Option<PathBuf> {
    let manifest = path.join("manifest");
    if manifest.exists() {
        Some(manifest)
    } else {
        path.parent().and_then(|p| find_manifest(p))
    }
}

pub fn find_manifest_path() -> Result<PathBuf, Error> {
    let cwd = env::current_dir()?;
    find_manifest(&cwd).ok_or(Error::from("no project file found!"))
}

pub fn find_project_dir() -> Result<PathBuf, Error> {
    let mut manifest_path = find_manifest_path()?;
    manifest_path.pop();
    Ok(manifest_path)
}

pub fn read_manifest() -> Result<Manifest, Error> {
    let manifest_path = find_manifest_path()?;
    let root = manifest_path.parent().unwrap_or(Path::new(&"."));
    let data = File::open(manifest_path.clone()).and_then(|mut f| {
        let mut s = String::new();
        f.read_to_string(&mut s).map(|_| s)
    })?;
    Ok(Manifest::from_str(data, root)?)
}
