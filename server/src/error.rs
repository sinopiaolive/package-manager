use std::env;
use redis;
use serde_json;
use reqwest;
use data_encoding;
use url;
use rocket::http::Status;
use rocket::response::{Response, Responder};
use rocket::request::Request;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Io(err: ::std::io::Error) {
            cause(err)
            description(err.description())
            from()
        }
        Redis(err: redis::RedisError) {
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
