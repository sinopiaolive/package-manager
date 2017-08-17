use std::io::{Read, Write};
use std::fs::{File, create_dir_all};
use toml;

use error::Error;
use path::config_path;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub auth: Auth,
}

impl Config {
    fn new() -> Config {
        Config {
            auth: Auth {
                token: None
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Auth {
    pub token: Option<String>,
}

fn read_config<R>(r: &mut R) -> Result<Config, Error>
where
    R: Read,
{
    let mut data = String::new();
    r.read_to_string(&mut data)?;
    Ok(toml::from_str(&data)?)
}

pub fn get_config() -> Result<Config, Error> {
    let mut path = config_path()?;
    path.push("config.toml");
    match File::open(path) {
        Err(_) => Ok(Config::new()),
        Ok(mut file) => read_config(&mut file)
    }
}

pub fn write_config(config: &Config) -> Result<(), Error> {
    let mut path = config_path()?;
    create_dir_all(&path)?;
    path.push("config.toml");
    let mut file = File::create(path)?;
    let data = toml::to_string(config)?;
    file.write(data.as_bytes())?;
    Ok(())
}
