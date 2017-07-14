#![allow(dead_code)]

use std::path::PathBuf;
use std::env::home_dir;

use error::Error;

pub fn config_path() -> Result<PathBuf, Error> {
    let mut p = home_dir().ok_or(Error::from("unable to find user home directory!"))?;
    p.push(".package-manager");
    Ok(p)
}
