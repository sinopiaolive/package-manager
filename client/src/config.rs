use failure;
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use toml;

use crate::path::config_path;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub auth: Auth,
}

impl Config {
    fn new() -> Config {
        Config {
            auth: Auth { token: None },
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Auth {
    pub token: Option<String>,
}

fn read_config<R>(r: &mut R) -> Result<Config, failure::Error>
where
    R: Read,
{
    let mut data = String::new();
    r.read_to_string(&mut data)?;
    Ok(toml::from_str(&data)?)
}

pub fn get_config() -> Result<Config, failure::Error> {
    let mut path = config_path()?;
    path.push("config.toml");
    match File::open(path) {
        Err(_) => Ok(Config::new()),
        Ok(mut file) => read_config(&mut file),
    }
}

pub fn write_config(config: &Config) -> Result<(), failure::Error> {
    let mut path = config_path()?;
    create_dir_all(&path)?;
    path.push("config.toml");
    let mut file = File::create(path)?;
    let data = toml::to_string(config)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}
