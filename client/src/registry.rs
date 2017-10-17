use std::fmt;
use std::io::Read;
use reqwest::{self, Method};
use reqwest::header::Authorization;
use reqwest::Body;
use im::Map;
use url::form_urlencoded::Serializer;
use serde::Deserialize;

use config::get_config;
use error::Error;

#[derive(Deserialize)]
pub struct RegistryError {
    message: String,
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message)
    }
}

pub type Response<A> = Result<A, RegistryError>;

fn read_auth() -> Result<String, Error> {
    let config = get_config()?;
    config.auth.token.ok_or(Error::Message(
        From::from("Please log in first using `pm login`."),
    ))
}

fn request<A, R>(
    method: Method,
    url: &str,
    args: Map<String, String>,
    body: Option<R>,
    auth: bool,
) -> Result<Response<A>, Error>
where
    for<'de> A: Deserialize<'de>,
    R: Read + Send + 'static,
{
    let mut ser = Serializer::new(String::new());
    for (k, v) in args {
        ser.append_pair(&*k, &*v);
    }
    let args_str = ser.finish();

    let http = reqwest::Client::new()?;
    let mut req = http.request(
        method,
        &format!("http://localhost:8000/{}?{}", url, args_str),
    )?;
    if auth {
        req.header(Authorization(format!("Bearer {}", read_auth()?)));
    }
    if let Some(data) = body {
        req.body(Body::new(data));
    }
    let res = req.send()?;

    if res.status().is_success() {
        Ok(Ok(::serde_json::from_reader(res)?))
    } else {
        Ok(Err(::serde_json::from_reader(res)?))
    }
}

pub fn get<A>(url: &str, args: Map<String, String>) -> Result<Response<A>, Error>
where
    for<'de> A: Deserialize<'de>,
{
    request::<A, &'static [u8]>(Method::Get, url, args, None, false)
}

pub fn get_auth<A>(url: &str, args: Map<String, String>) -> Result<Response<A>, Error>
where
    for<'de> A: Deserialize<'de>,
{
    request::<A, &'static [u8]>(Method::Get, url, args, None, true)
}

pub fn post<A, R>(url: &str, args: Map<String, String>, data: R) -> Result<Response<A>, Error>
where
    for<'de> A: Deserialize<'de>,
    R: Read + Send + 'static,
{
    request(Method::Post, url, args, Some(data), true)
}
