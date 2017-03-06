extern crate docopt;
extern crate rustc_serialize;
extern crate package_manager;

use docopt::Docopt;
use std::process;
use rustc_serialize::Decodable;
use std::env;
use package_manager::error::Error;
use package_manager::manifest::find_project_dir;

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

type Result = std::result::Result<(), Error>;



macro_rules! each_subcommand {
    ($mac:ident) => {
        $mac!(test);
        // add more like this here
    }
}

macro_rules! declare_mod {
    ($name:ident) => ( pub mod $name; )
}
each_subcommand!(declare_mod);



fn run_builtin_command<Flags: Decodable>(exec: fn(Flags) -> Result, usage: &str) -> Result {
    let docopt = Docopt::new(usage).unwrap().help(true);
    docopt.decode().map_err(|e| e.exit()).and_then(|opts| exec(opts))
}

fn attempt_builtin_command(cmd: &str) -> Option<Result> {
    macro_rules! cmd {
        ($name:ident) => (if cmd == stringify!($name).replace("_", "-") {
            return Some(run_builtin_command($name::execute, $name::USAGE))
        })
    }
    each_subcommand!(cmd);
    None
}

fn run_shell_command(cmd: &str, args: &Vec<String>) -> Result {
    let prefixed_cmd = format!("pm-{}", cmd);
    let sh = process::Command::new(&prefixed_cmd)
        .args(args)
        .output();
    println!("exec({:?}, {:?}) -> {:?}", prefixed_cmd, args, sh);
    Ok(()) // FIXME: report subprocess result properly
}

fn change_to_project_dir() -> Result {
    let path = find_project_dir()?;
    Ok(env::set_current_dir(path)?)
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
        match change_to_project_dir() {
            Ok(_) => (),
            Err(e) => {
                println!("ERROR: {}", e);
                process::exit(1)
            }
        }
        match attempt_builtin_command(&args.arg_command)
            .or_else(|| Some(run_shell_command(&args.arg_command, &args.arg_args)))
            .unwrap() {
            Ok(_) => process::exit(0),
            Err(e) => {
                println!("{}", e);
                process::exit(1)
            }
        }
    }
}
