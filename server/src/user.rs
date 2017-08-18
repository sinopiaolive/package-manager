use std::fmt;
use std::str::FromStr;

use rocket::request::FromFormValue;
use rocket::http::RawStr;

use error::{Res, Error};
use auth::AuthSource;
use schema::users;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct User {
    pub provider: AuthSource,
    pub id: String,
}

impl User {
    pub fn to_string(&self) -> String {
        format!("{}", self)
    }
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

impl FromStr for User {
    type Err = Error;
    fn from_str(s: &str) -> Res<User> {
        let mut it = s.split(':');
        let source = it.next().ok_or(Error::InvalidUserID(s.to_string()))?;
        let id = it.next().ok_or(Error::InvalidUserID(s.to_string()))?;
        if it.next() != None {
            return Err(Error::InvalidUserID(s.to_string()));
        }
        Ok(User {
            provider: AuthSource::from_str(source)?,
            id: id.to_string(),
        })
    }
}

impl<'v> FromFormValue<'v> for User {
    type Error = Error;

    fn from_form_value(val: &'v RawStr) -> Res<Self> {
        Self::from_str(&val.url_decode()?)
    }
}

#[derive(Insertable, Queryable, Debug)]
#[table_name = "users"]
pub struct UserRecord {
    pub id: String,
    pub name: String,
    pub email: String,
    pub avatar: Option<String>,
}

impl UserRecord {
    pub fn new(user: &User, name: &str, email: &str, avatar: &str) -> Self {
        UserRecord {
            id: user.to_string(),
            name: name.to_string(),
            email: email.to_string(),
            avatar: if avatar.len() > 0 {
                Some(avatar.to_string())
            } else {
                None
            },
        }
    }

    pub fn user(&self) -> Res<User> {
        User::from_str(&self.id)
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

#[derive(Clone, Debug)]
pub struct OrgRecord {
    pub id: Org,
    pub name: String,
}

impl OrgRecord {
    pub fn org(&self) -> Org {
        self.id.clone()
    }
}
