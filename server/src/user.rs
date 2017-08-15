use std::fmt;

use rocket::request::FromFormValue;
use rocket::http::RawStr;

use error::{Res, Error};
use auth::AuthSource;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct User {
    pub provider: AuthSource,
    pub id: String,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}:{}", self.provider, self.id)
    }
}

impl fmt::Debug for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(self, f)
    }
}

impl<'v> FromFormValue<'v> for User {
    type Error = Error;

    fn from_form_value(val: &'v RawStr) -> Res<Self> {
        let data = val.url_decode()?;
        let mut it = data.split(':');
        let source = it.next().ok_or(Error::InvalidUserID(data.clone()))?;
        let id = it.next().ok_or(Error::InvalidUserID(data.clone()))?;
        if it.next() != None {
            return Err(Error::InvalidUserID(data.clone()));
        }
        Ok(User {
            provider: AuthSource::from_str(source)?,
            id: id.to_string()
        })
    }
}



#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Org {
    pub provider: AuthSource,
    pub id: String,
}

impl fmt::Display for Org {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}:{}", self.provider, self.id)
    }
}

impl fmt::Debug for Org {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(self, f)
    }
}
