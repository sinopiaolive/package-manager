#![allow(dead_code)]

use dirs::home_dir;
use std::path::PathBuf;

use failure;

pub fn config_path() -> Result<PathBuf, failure::Error> {
    let mut p = home_dir().ok_or_else(|| format_err!("unable to find user home directory!"))?;
    p.push(".package-manager");
    Ok(p)
}
