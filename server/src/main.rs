#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
#![allow(resolve_trait_on_defaulted_unit)]

extern crate rocket;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate rocket_contrib;
extern crate redis;
#[macro_use]
extern crate quick_error;
extern crate medallion;

mod error;
mod user;
mod store;
mod auth;

use std::default::Default;

use rocket::State;
use rocket::http::Status;
use rocket_contrib::JSON;

use error::{Res, Error};
use auth::{Authenticate, JWTToken};
use store::Store;
use user::make_secret;



#[derive(Deserialize)]
pub struct AuthPair {
    user: String,
    password: String,
}

#[post("/register", data = "<auth>")]
fn register(store: State<Store>, auth: JSON<AuthPair>) -> Res<String> {
    match store.exists(&auth.user)? {
        true => Err(Error::Status(Status::BadRequest)),
        false => {
            store.set_password(&auth.user, &auth.0.password)?;
            store.set_secret(&auth.user, &make_secret())?;
            Ok("OK".to_string())
        }
    }
}

#[post("/auth", data = "<auth>")]
fn auth(store: State<Store>, auth: JSON<AuthPair>) -> Res<String> {
    let user = store.get(&auth.user)?;
    match user.password == auth.password {
        false => Err(Error::Status(Status::Unauthorized)),
        true => {
            let header: medallion::Header<()> = Default::default();
            let payload = medallion::Payload {
                claims: Some(JWTToken { user: auth.user.clone() }),
                ..Default::default()
            };
            let token = medallion::Token::new(header, payload);
            Ok(token.sign(user.secret.as_bytes())?)
        }
    }
}

#[get("/test")]
fn test(store: State<Store>, auth: Authenticate) -> Res<String> {
    auth.validate(&store)?;
    Ok("Hello Joe".to_string())
}

fn main() {
    let store = Store::new().expect("couldn't connect to Redis server");
    rocket::ignite()
        .manage(store)
        .mount("/", routes![register, auth, test])
        .launch()
}
