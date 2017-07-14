use redis::{cmd, Client};

use error::Res;
use user::User;

pub struct Store {
    redis: Client,
}

impl Store {
    pub fn new() -> Res<Store> {
        Ok(Store { redis: Client::open("redis://localhost/")? })
    }

    pub fn exists(&self, key: &str) -> Res<bool> {
        let r = cmd("EXISTS").arg(format!("{}:secret", key)).query(
            &self.redis,
        )?;
        Ok(r)
    }

    pub fn get(&self, key: &str) -> Res<User> {
        let password = cmd("GET").arg(format!("{}:password", key)).query(
            &self.redis,
        )?;
        let secret = cmd("GET").arg(format!("{}:secret", key)).query(
            &self.redis,
        )?;
        Ok(User { password, secret })
    }

    pub fn set_password(&self, key: &str, password: &str) -> Res<()> {
        cmd("SET")
            .arg(format!("{}:password", key))
            .arg(password)
            .query(&self.redis)?;
        Ok(())
    }

    pub fn set_secret(&self, key: &str, secret: &str) -> Res<()> {
        cmd("SET")
            .arg(format!("{}:secret", key))
            .arg(secret)
            .query(&self.redis)?;
        Ok(())
    }
}
