use redis::{cmd, Client};
use data_encoding::BASE64;

use error::{Res, Error};

pub struct Store {
    redis: Client,
}

impl Store {
    pub fn new() -> Res<Store> {
        Ok(Store { redis: Client::open("redis://localhost/")? })
    }

    pub fn register_login(&self, token: &str, callback: &str) -> Res<bool> {
        if BASE64.decode(token.as_bytes()).is_err() {
            return Err(Error::InvalidLoginState(token.to_string()))
        }
        Ok(cmd("SET").arg(format!("login:{}", token)).arg(callback).arg("EX").arg(1800).query(&self.redis)?)
    }

    pub fn validate_login(&self, token: &str) -> Res<String> {
        if BASE64.decode(token.as_bytes()).is_err() {
            return Err(Error::InvalidLoginState(token.to_string()))
        }
        let key = format!("login:{}", token);
        let r = cmd("GET").arg(key.clone()).query(&self.redis)?;
        cmd("DEL").arg(key).query(&self.redis)?;
        Ok(r)
    }
}
