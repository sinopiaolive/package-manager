use std::io::Read;

use linefeed::reader::{Reader, ReadResult};
use rpassword::prompt_password_stdout;
use reqwest;

use error::Error;
use auth::AuthPair;

pub const USAGE: &'static str = "Login.

Usage:
    pm login [options] [<username>]

Options:
    -h, --help     Display this message.
";

#[derive(Debug, Deserialize)]
pub struct Args {
    arg_username: Option<String>
}



pub fn execute(args: Args) -> Result<(), Error> {
    let mut reader = Reader::new("pm")?;
    let user = match args.arg_username {
        Some(user) => user.clone(),
        None => {
            reader.set_prompt("Username: ");
            match reader.read_line() {
                Ok(ReadResult::Input(user)) => user,
                Err(e) => return Err(Error::from(e)),
                _ => return Err(Error::from("read error"))
            }
        }
    };
    let password = prompt_password_stdout("Password: ")?;
    let auth = AuthPair {user, password};

    let http = reqwest::Client::new()?;
    let mut res = http.post("http://localhost:8000/auth")?.json(&auth)?.send()?;

    println!("{} responded with code {}:", res.url(), res.status());

    if !res.status().is_success() {
        return Err(Error::from(format!("remote server said {}", res.status())))
    }

    let mut token = String::new();
    res.read_to_string(&mut token)?;

    println!("Your token is {}", token);

    Ok(())
}
