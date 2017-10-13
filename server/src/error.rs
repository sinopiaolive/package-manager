use std::env;
use serde_json;
use rmp_serde;
use reqwest;
use data_encoding;
use url;
use rocket::http::Status;
use rocket::response::{Response, Responder};
use rocket::request::Request;
use diesel;

use user::User;

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
        UnknownFile(namespace: String, name: String) {
            display("No such file: {}/{}", namespace, name)
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
    }
}

impl<'a> Responder<'a> for Error {
    fn respond_to(self, _: &Request) -> Result<Response<'a>, Status> {
        match self {
            Error::Status(code) => Err(code),
            // TODO real logging?
            _ => {
                println!("error: {:?}", &self);
                Err(Status::InternalServerError)
            }
        }
    }
}

pub type Res<A> = Result<A, Error>;
