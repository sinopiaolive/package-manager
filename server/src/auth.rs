use std::fmt;

use data_encoding::BASE64URL;
use serde_json;

use rocket::request::FromFormValue;
use rocket::http::RawStr;

use error::{Res, Error};
use user::{UserRecord, OrgRecord, User};

use github::Github;
use gitlab::Gitlab;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum AuthSource {
    Github,
    Gitlab
}

impl AuthSource {
    pub fn from_str(name: &str) -> Res<Self> {
        match name {
            "github" => Ok(AuthSource::Github),
            "gitlab" => Ok(AuthSource::Gitlab),
            _ => Err(Error::NoSuchAuthSource(name.to_string()))
        }
    }

    pub fn provider(&self) -> Res<Box<AuthProvider>> {
        Ok(match self {
            &AuthSource::Github => Box::new(Github::new()?),
            &AuthSource::Gitlab => Box::new(Gitlab::new()?),
        })
    }
}

impl fmt::Display for AuthSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str(match self {
            &AuthSource::Github => "github",
            &AuthSource::Gitlab => "gitlab",
        })
    }
}

impl<'v> FromFormValue<'v> for AuthSource {
    type Error = Error;

    fn from_form_value(val: &'v RawStr) -> Res<Self> {
        AuthSource::from_str(&val.url_decode()?)
    }
}



pub trait AuthProvider {
    fn user(&self, token: &str) -> Res<UserRecord>;
    fn orgs(&self, token: &str) -> Res<Box<Iterator<Item = OrgRecord>>>;
}



#[derive(Serialize, Deserialize, Clone, FromForm)]
pub struct AuthToken {
    pub user: User,
    pub token: String
}

impl AuthToken {
    pub fn new(user: &User, token: &str) -> AuthToken {
        AuthToken {
            user: user.clone(),
            token: token.to_string()
        }
    }

    pub fn decode(data: &[u8]) -> Res<AuthToken> {
        Ok(serde_json::from_slice(&BASE64URL.decode(data)?)?)
    }

    pub fn encode(&self) -> Res<String> {
        Ok(BASE64URL.encode(serde_json::to_string(self)?.as_bytes()))
    }
}
