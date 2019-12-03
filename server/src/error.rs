use std::env;
use std::io::Cursor;
use serde_json;
use rmp_serde;
use reqwest;
use data_encoding;
use url;
use rocket::http::{Status, ContentType};
use rocket::response::{Response, Responder};
use rocket::request::Request;
use diesel;

use crate::user::User;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Io(err: ::std::io::Error) {
            cause(err)
            description(err.description())
            from()
        }
        Utf8(err: ::std::str::Utf8Error) {
            cause(err)
            description(err.description())
            from()
        }
        Decode(err: data_encoding::DecodeError) {
            cause(err)
            description(err.description())
            from()
        }
        JSON(err: serde_json::Error) {
            cause(err)
            description(err.description())
            from()
        }
        MessagePack(err: rmp_serde::decode::Error) {
            cause(err)
            description(err.description())
            from()
        }
        HTTP(err: reqwest::Error) {
            cause(err)
            description(err.description())
            from()
        }
        Url(err: url::ParseError) {
            cause(err)
            description(err.description())
            from()
        }
        EnvVar(err: env::VarError) {
            cause(err)
            description(err.description())
            from()
        }
        DieselConnection(err: diesel::ConnectionError) {
            cause(err)
            description(err.description())
            from()
        }
        DieselResult(err: diesel::result::Error) {
            cause(err)
            description(err.description())
            from()
        }
        Status(code: Status) {}
        NoSuchAuthSource(name: String) {
            description(name)
        }
        InvalidUserID(name: String) {
            description(name)
        }
        InvalidLoginState(name: String) {
            description(name)
        }
        UserHasNoEmail(name: String) {
            description(name)
        }
        UnknownUser(name: String) {
            description(name)
        }
        UnknownPackage(namespace: String, name: String) {
            display("No such package: {}/{}", namespace, name)
        }
        UnknownRelease(namespace: String, name: String, version: String) {
            display("No such package version: {}/{}-{}", namespace, name, version)
        }
        AccessDenied(namespace: String, name: String, user: User) {
            display("User {} is not an owner of {}/{}", user, name, namespace)
        }
        InvalidManifest(reason: &'static str) {
            display("Invalid manifest: {}", reason)
        }
        InvalidArtifact(reason: &'static str) {
            display("Invalid upload artifact: {}", reason)
        }
        ReleaseAlreadyExists(namespace: String, name: String, version: String) {
            display("This release already exists: {}/{}-{}", namespace, name, version)
        }
    }
}

#[derive(Serialize)]
struct ServerError {
    message: String,
}

impl<'a> Responder<'a> for Error {
    fn respond_to(self, _: &Request) -> Result<Response<'a>, Status> {
        match self {
            Error::Status(code) => Err(code),
            // TODO real logging?
            _ => {
                println!("error: {:?}", self);
                let data = serde_json::to_vec(&ServerError { message: format!("{}", self) })
                    .unwrap_or_else(|_|
                        "{message:\"an error occurred but I couldn't serialise it for you\"}"
                            .as_bytes()
                            .to_owned(),
                    );
                Response::build()
                    .status(Status::InternalServerError)
                    .header(ContentType::JSON)
                    .sized_body(Cursor::new(data))
                    .ok()
            }
        }
    }
}

pub type Res<A> = Result<A, Error>;
