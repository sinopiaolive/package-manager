extern crate docopt;
extern crate rustc_serialize;

use docopt::Docopt;
use std::process::{self};

const USAGE: &'static str = "Your package manager.

Usage:
    pm <command> [<args>...]
    pm [options]

Options:
    -h, --help     Display this message.
    -v, --version  Print version info.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_command: String,
    arg_args: Vec<String>,
}

fn run_command(cmd: &str, args: &Vec<String>) {
    let prefixed_cmd = format!("pm-{}", cmd);
    let sh = process::Command::new(&prefixed_cmd)
             .args(args)
             .output();
    println!("exec({:?}, {:?}) -> {:?}", prefixed_cmd, args, sh);
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .map(|d| d.options_first(true))
        .map(|d| d.help(true))
        .map(|d| d.version(Some("0.999999-rc623-beta2".to_string())))
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());
    if args.arg_command.is_empty() {
        println!("{:?}", args);
        print!("{}", USAGE);
        process::exit(1)
    } else {
        run_command(&args.arg_command, &args.arg_args);
    }
}
