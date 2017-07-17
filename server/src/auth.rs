use medallion::Token;
use rocket::request::{Request, FromRequest};
use rocket::Outcome;
use rocket::http::Status;

use error::{Res, Error};
use store::Store;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct JWTToken {
    pub user: String,
}

impl Default for JWTToken {
    fn default() -> JWTToken {
        JWTToken { user: String::default() }
    }
}

fn parse_auth_header<'a>(header: &'a str) -> Option<&'a str> {
    let start = "Bearer ";
    if header.starts_with(start) {
        Some(&header[start.len()..])
    } else {
        None
    }
}

pub struct Authenticate(Token<(), JWTToken>);

impl<'a, 'r> FromRequest<'a, 'r> for Authenticate {
    type Error = Error;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, (Status, Self::Error), ()> {
        match request.headers().get_one("Authorization").and_then(
            parse_auth_header,
        ) {
            None => Outcome::Failure((Status::Unauthorized, Error::Status(Status::Unauthorized))),
            Some(token) => match Token::parse(token) {
                Ok(token) => Outcome::Success(Authenticate(token)),
                Err(err) => Outcome::Failure((Status::Unauthorized, Error::JWT(err)))
            }
        }
    }
}

impl Authenticate {
    fn token(&self) -> &Token<(), JWTToken> {
        &self.0
    }

    fn claims(&self) -> Res<JWTToken> {
        match self.token().payload.claims {
            None => Err(Error::Status(Status::Unauthorized)),
            Some(ref claims) => Ok(claims.clone())
        }
    }

    pub fn validate(&self, store: &Store) -> Res<JWTToken> {
        let claims = self.claims()?;
        match store.get(&claims.user) {
            Err(_) => Err(Error::Status(Status::Unauthorized)),
            Ok(user) => match self.token().verify(user.secret.as_bytes()) {
                Ok(true) => Ok(claims),
                _ => Err(Error::Status(Status::Unauthorized))
            }
        }
    }
}
