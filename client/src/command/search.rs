use std::cmp::{min, max};

use term_size;
use colored::Colorize;

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

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct SearchResult {
    name: String,
    version: String,
    publisher: String,
    description: String,
}

pub fn execute(args: Args) -> Result<(), Error> {
    match registry::get_json::<Vec<SearchResult>>(
        "search",
        map!{"ns".to_string() => args.arg_namespace, "q".to_string() => args.arg_keyword.join(" ")},
    ) {
        Ok(ref results) if results.is_empty() => println!("No results found!"),
        Ok(ref results) => print_results(results),
        Err(ref msg) => println!("Registry response: {}", msg),
    };
    Ok(())
}

fn print_results(results: &Vec<SearchResult>) {
    let width = match term_size::dimensions() {
        Some((width, _)) => width,
        None => 80,
    };
    let max_avail = width - 2;
    let bar = "|".dimmed();
    let w_pkg = max(7, min(24, results.iter().map(|r| r.name.len()).max().unwrap()));
    let h_pkg = format!("{:1$}", "Package", w_pkg).green();
    let w_ver = max(7, min(16, results.iter().map(|r| r.version.len()).max().unwrap()));
    let h_ver = format!("{:1$}", "Version", w_ver).green();
    let w_desc = max_avail - w_pkg - w_ver;
    let h_desc = format!("{:1$}", "Description", w_desc).green();
    let sep = format!("{}|{}|{}", "-".repeat(w_pkg), "-".repeat(w_ver), "-".repeat(w_desc)).dimmed();
    println!("{}", sep);
    println!("{}{}{}{}{}", h_pkg, bar, h_ver, bar, h_desc);
    println!("{}", sep);
    for result in results {
        println!("{:4$}{3}{:5$}{3}{}", result.name.bold(), result.version, result.description, bar, w_pkg, w_ver)
    }
    println!("{}", sep);
}
