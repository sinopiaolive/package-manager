#![allow(dead_code)]

use failure;
use std::env;
use std::path::{Path, PathBuf};

pub struct ProjectPaths {
    pub root: PathBuf,
    pub manifest: PathBuf,
    pub lockfile: PathBuf, // might not exist
}

fn find_manifest(path: &Path) -> Option<PathBuf> {
    let manifest = path.join("deps");
    if manifest.exists() {
        Some(manifest)
    } else {
        path.parent().and_then(|p| find_manifest(p))
    }
}

pub fn find_manifest_path() -> Result<PathBuf, failure::Error> {
    let cwd = env::current_dir()?;
    find_manifest(&cwd).ok_or_else(|| format_err!("no project file found!"))
}

pub fn find_project_dir() -> Result<PathBuf, failure::Error> {
    let mut manifest_path = find_manifest_path()?;
    manifest_path.pop();
    Ok(manifest_path)
}

pub fn find_project_paths() -> Result<ProjectPaths, failure::Error> {
    let cwd = env::current_dir()?;
    find_project_paths_from(&cwd)
}

pub fn find_project_paths_from(root: &Path) -> Result<ProjectPaths, failure::Error> {
    let manifest = root.join("deps");
    if manifest.exists() {
        Ok(ProjectPaths {
            root: root.to_path_buf(),
            manifest,
            lockfile: root.join("deps.lock"),
        })
    } else {
        match root.parent() {
            None => bail!("File not found: deps"),
            Some(parent) => find_project_paths_from(parent)
        }
    }
}
