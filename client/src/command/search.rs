use std::cmp::{min, max};

use console::{Term, Style};

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
    match registry::get::<Vec<SearchResult>>(
        "search",
        ordmap!{"ns".to_string() => args.arg_namespace, "q".to_string() => args.arg_keyword.join(" ")},
    )? {
        Ok(ref results) if results.is_empty() => println!("No results found!"),
        Ok(ref results) => print_results(results),
        Err(ref msg) => println!("Registry response: {}", msg),
    };
    Ok(())
}

fn print_results(results: &Vec<SearchResult>) {
    let border = Style::new().dim();
    let header = Style::new().green();
    let package = Style::new().bold();

    let term = Term::stdout();
    let width = term.size().1 as usize;
    let max_avail = width - 2;
    let bar = border.apply_to("|");
    let w_pkg = max(
        7,
        min(24, results.iter().map(|r| r.name.len()).max().unwrap()),
    );
    let h_pkg = header.apply_to(format!("{:1$}", "Package", w_pkg));
    let w_ver = max(
        7,
        min(16, results.iter().map(|r| r.version.len()).max().unwrap()),
    );
    let h_ver = header.apply_to(format!("{:1$}", "Version", w_ver));
    let w_desc = max_avail - w_pkg - w_ver;
    let h_desc = header.apply_to(format!("{:1$}", "Description", w_desc));
    let sep = border.apply_to(format!(
        "{}|{}|{}",
        "-".repeat(w_pkg),
        "-".repeat(w_ver),
        "-".repeat(w_desc)
    ));
    println!("{}", sep);
    println!("{}{}{}{}{}", h_pkg, bar, h_ver, bar, h_desc);
    println!("{}", sep);
    for result in results {
        println!(
            "{:4$}{3}{:5$}{3}{}",
            package.apply_to(&result.name),
            result.version,
            result.description,
            bar,
            w_pkg,
            w_ver
        )
    }
    println!("{}", sep);
}
