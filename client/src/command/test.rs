use error::Error;
use registry;

pub const USAGE: &'static str = "Test page.

Usage:
    pm test [options]

Options:
    -h, --help     Display this message.
";

#[derive(Debug, Deserialize)]
pub struct Args {}



pub fn execute(_args: Args) -> Result<(), Error> {
    match registry::get_auth::<String>("test", ordmap!{})? {
        Ok(_) => println!("You are logged in with a valid auth token."),
        Err(msg) => println!("Registry response: {}", msg),
    };
    Ok(())
}
