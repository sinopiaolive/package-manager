use std::io::Read;
use reqwest;
use reqwest::header::Authorization;
use reqwest::Body;
use im::Map;
use url::form_urlencoded::Serializer;
use serde::Deserialize;

use config::get_config;
use error::Error;

pub fn get(url: &str, args: Map<String, String>) -> Result<String, Error> {
    let mut ser = Serializer::new(String::new());
    for (k, v) in args {
        ser.append_pair(&*k, &*v);
    }
    let args_str = ser.finish();

    let http = reqwest::Client::new()?;
    let mut res = http.get(&format!("http://localhost:8000/{}?{}", url, args_str))?
        .send()?;

    let mut data = String::new();
    res.read_to_string(&mut data)?;

    if res.status().is_success() {
        Ok(data)
    } else {
        Err(Error::Server(format!("{} {}", res.status(), data)))
    }
}

pub fn get_json<A>(url: &str, args: Map<String, String>) -> Result<A, Error>
where
    for<'de> A: Deserialize<'de>,
{
    Ok(::serde_json::from_reader(get(url, args)?.as_bytes())?)
}

pub fn get_auth(url: &str, args: Map<String, String>) -> Result<String, Error> {
    let config = get_config()?;
    let token = config.auth.token.ok_or(Error::Message(From::from(
        "Please log in first using `pm login`.",
    )))?;

    let mut ser = Serializer::new(String::new());
    for (k, v) in args {
        ser.append_pair(&*k, &*v);
    }
    let args_str = ser.finish();

    let http = reqwest::Client::new()?;
    let mut res = http.get(&format!("http://localhost:8000/{}?{}", url, args_str))?
        .header(Authorization(format!("Bearer {}", token)))
        .send()?;

    let mut data = String::new();
    res.read_to_string(&mut data)?;

    if res.status().is_success() {
        Ok(data)
    } else {
        Err(Error::Server(format!("{} {}", res.status(), data)))
    }
}

pub fn post<R>(url: &str, args: Map<String, String>, data: R) -> Result<String, Error>
where
    Body: From<R>,
{
    let config = get_config()?;
    let token = config.auth.token.ok_or(Error::Message(From::from(
        "Please log in first using `pm login`.",
    )))?;

    let mut ser = Serializer::new(String::new());
    for (k, v) in args {
        ser.append_pair(&*k, &*v);
    }
    let args_str = ser.finish();

    let http = reqwest::Client::new()?;
    let mut res = http.post(&format!("http://localhost:8000/{}?{}", url, args_str))?
        .header(Authorization(format!("Bearer {}", token)))
        .body(Body::from(data))
        .send()?;

    let mut data = String::new();
    res.read_to_string(&mut data)?;

    if res.status().is_success() {
        Ok(data)
    } else {
        Err(Error::Server(format!("{} {}", res.status(), data)))
    }
}
