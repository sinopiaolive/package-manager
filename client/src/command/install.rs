use resolve::resolve;

pub const USAGE: &str = "Install dependencies.

Usage:
    pm install [options]

Options:
    -h, --help     Display this message.
";

#[derive(Debug, Deserialize)]
pub struct Args {
}

pub fn execute(_args: Args) -> Result<(), failure::Error> {
    resolve()?;
    Ok(())
}
