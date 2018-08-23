#![allow(unused_features)]
#![feature(specialization, plugin, test)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate docopt;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate pm_lib;
extern crate rmp_serde;
extern crate serde_json;
extern crate toml;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate im;
extern crate data_encoding;
extern crate futures;
extern crate hyper;
extern crate pest;
extern crate rand;
extern crate reqwest;
extern crate url;
extern crate webbrowser;
#[macro_use]
extern crate pest_derive;
extern crate brotli;
extern crate console;
extern crate git2;
extern crate glob;
extern crate indicatif;
extern crate tar;
#[cfg(test)]
extern crate test;
#[cfg(test)]
#[macro_use]
extern crate matches;

mod config;
mod manifest;
mod path;
mod project;
mod registry;
#[allow(dead_code)]
// TODO please remove this when the solver is actually being used
#[macro_use]
mod solver;
mod files;
mod git;
mod io;
#[allow(dead_code)]
mod manifest_parser;
mod manifest_parser_error;

use docopt::Docopt;
use serde::de::Deserialize;
use std::process;

const USAGE: &'static str = "Your package manager.

Usage:
    pm <command> [<args>...]
    pm [options]

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
        $mac!(login);
        $mac!(test);
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
    docopt
        .deserialize()
        .map_err(|e| e.exit())
        .and_then(|opts| exec(opts))
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

fn run_shell_command(cmd: &str, args: &Vec<String>) -> Result {
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
                println!("ERROR: {}", e);
                process::exit(1)
            }
        }
    }
}
