extern crate docopt;
extern crate rustc_serialize;
extern crate package_manager;

use std::result;
use std::env;

pub const USAGE: &'static str = "Test page.

Usage:
    pm test [options]

Options:
    -h, --help     Display this message.
    --bdd          Use the Official BDD Style.
";

#[derive(Debug, RustcDecodable)]
pub struct Args {
    flag_bdd: bool,
}



pub fn execute(args: Args) -> result::Result<(), String> {
    if args.flag_bdd {
        println!("As the test command, when I am called, then I am the test command.")
    } else {
        println!("This is the test command.")
    }
    println!("Also, my working directory is {:?}", env::current_dir().unwrap().display());

    Ok(())
}
