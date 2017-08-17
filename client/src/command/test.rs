use std::io::Read;
use reqwest;
use reqwest::header::Authorization;
use error::Error;
use config::get_config;

pub const USAGE: &'static str = "Test page.

Usage:
    pm test [options]

Options:
    -h, --help     Display this message.
";

#[derive(Debug, Deserialize)]
pub struct Args {}



pub fn execute(_args: Args) -> Result<(), Error> {
    let config = get_config()?;
    let token = config.auth.token.ok_or(Error::Message(From::from("Please log in first.")))?;

    let http = reqwest::Client::new()?;
    let mut res = http.get("http://localhost:8000/test")?
        .header(Authorization(format!("Bearer {}", token)))
        .send()?;

    if res.status().is_success() {
        println!("You are logged in with a valid auth token.");
    } else {
        let mut data = String::new();
        res.read_to_string(&mut data)?;
        println!("{} says: {} {}", res.url(), res.status(), data);
    }

    Ok(())
}
