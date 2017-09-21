use error::Error;
use registry;

pub const USAGE: &'static str = "Search for packages by keyword.

Usage:
    pm search [options] <namespace> <keyword>...

Options:
    -h, --help     Display this message.
";

#[derive(Debug, Deserialize)]
pub struct Args {
    arg_namespace: String,
    arg_keyword: Vec<String>,
}



pub fn execute(args: Args) -> Result<(), Error> {
    match registry::get(
        "search",
        map!{"ns".to_string() => args.arg_namespace, "q".to_string() => args.arg_keyword.join(" ")},
    ) {
        Ok(result) => println!("Registry says: {}", result),
        Err(msg) => println!("Registry response: {}", msg),
    };
    Ok(())
}
