use failure;
use im::OrdMap as Map;
use reqwest::Body;
use reqwest::{self, Method};
use serde::Deserialize;
use std::fmt;
use std::io::Read;
use url::form_urlencoded::Serializer;

use config::get_config;

#[derive(Fail, Deserialize, Debug)]
pub struct RegistryError {
    message: String,
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message)
    }
}

pub type Response<A> = Result<A, RegistryError>;

fn read_auth() -> Result<String, failure::Error> {
    let config = get_config()?;
    config
        .auth
        .token
        .ok_or_else(|| format_err!("Please log in first using `pm login`."))
}

fn request<A, R>(
    method: Method,
    url: &str,
    args: Map<String, String>,
    body: Option<R>,
    auth: bool,
) -> Result<Response<A>, failure::Error>
where
    for<'de> A: Deserialize<'de>,
    R: Read + Send + 'static,
{
    let mut ser = Serializer::new(String::new());
    for (k, v) in args {
        ser.append_pair(&*k, &*v);
    }
    let args_str = ser.finish();

    let http = reqwest::Client::new();
    let mut req = http.request(
        method,
        &format!("http://localhost:8000/{}?{}", url, args_str),
    );
    if auth {
        req = req.header("Authorization", format!("Bearer {}", read_auth()?));
    }
    if let Some(data) = body {
        req = req.body(Body::new(data));
    }
    let mut res = req.send()?;

    if res.status().is_success() {
        Ok(Ok(::serde_json::from_reader(res)?))
    } else {
        Ok(Err(RegistryError { message: res.text()? }))
    }
}

pub fn get<A>(url: &str, args: Map<String, String>) -> Result<Response<A>, failure::Error>
where
    for<'de> A: Deserialize<'de>,
{
    request::<A, &'static [u8]>(Method::GET, url, args, None, false)
}

pub fn post<A, R>(
    url: &str,
    args: Map<String, String>,
    data: R,
) -> Result<Response<A>, failure::Error>
where
    for<'de> A: Deserialize<'de>,
    R: Read + Send + 'static,
{
    request(Method::POST, url, args, Some(data), true)
}
