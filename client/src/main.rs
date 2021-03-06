#![warn(clippy::all)]

extern crate dirs;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate im_rc as im;
#[macro_use]
extern crate pest_derive;
#[cfg(test)]
#[macro_use]
extern crate matches;

mod config;
mod files;
mod git;
mod io;
mod lockfile;
mod manifest;
mod manifest_parser;
mod manifest_parser_error;
mod path;
mod project;
mod registry;
mod resolve;

use docopt::Docopt;
use serde::de::Deserialize;
use std::env;
use std::process;

static REGISTRY_URL: &str = "http://localhost:8000";

const USAGE: &str = "Your package manager.

Usage:
    pm <command> [<args>...]
    pm [options]

Subcommands:
    install
    search
    login
    publish

Options:
    -h, --help     Display this message.
    -v, --version  Print version info.
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_command: String,
    arg_args: Vec<String>,
}

type Result = std::result::Result<(), failure::Error>;

macro_rules! each_subcommand {
    ($mac:ident) => {
        $mac!(install);
        $mac!(login);
        $mac!(search);
        $mac!(publish);
    };
}

mod command;

fn run_builtin_command<'de, Flags: Deserialize<'de>>(
    exec: fn(Flags) -> Result,
    usage: &str,
) -> Result {
    let docopt = Docopt::new(usage).unwrap().help(true);
    docopt.deserialize().map_err(|e| e.exit()).and_then(exec)
}

fn attempt_builtin_command(cmd: &str) -> Option<Result> {
    macro_rules! cmd {
        ($name:ident) => {
            if cmd == stringify!($name).replace("_", "-") {
                return Some(run_builtin_command(
                    command::$name::execute,
                    command::$name::USAGE,
                ));
            }
        };
    }
    each_subcommand!(cmd);
    None
}

fn run_shell_command(cmd: &str, args: &[String]) -> Result {
    let prefixed_cmd = format!("pm-{}", cmd);
    let sh = process::Command::new(&prefixed_cmd).args(args).output();
    println!("exec({:?}, {:?}) -> {:?}", prefixed_cmd, args, sh);
    Ok(()) // FIXME: report subprocess result properly
}

fn main() {
    // manifest::test_reader();
    // git::test_git();
    // process::exit(0);

    let args: Args = Docopt::new(USAGE)
        .map(|d| d.options_first(true))
        .map(|d| d.help(true))
        .map(|d| d.version(Some("0.999999-rc623-beta2".to_string())))
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    if args.arg_command.is_empty() {
        println!("{:?}", args);
        print!("{}", USAGE);
        process::exit(1)
    } else {
        match attempt_builtin_command(&args.arg_command)
            .or_else(|| Some(run_shell_command(&args.arg_command, &args.arg_args)))
            .unwrap()
        {
            Ok(_) => process::exit(0),
            Err(e) => {
                if env::var_os("RUST_BACKTRACE").is_none()
                    && env::var_os("RUST_FAILURE_BACKTRACE").is_none()
                {
                    println!("{}", e);
                    println!();
                    println!(
                        "Run with `RUST_BACKTRACE=1` environment variable to display a backtrace."
                    );
                } else {
                    println!("{:?}", e);
                    println!();
                }
                println!("Exiting due to error.");
                process::exit(1)
            }
        }
    }
}
