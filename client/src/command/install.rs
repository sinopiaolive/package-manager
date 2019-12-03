use std::fs;

use lockfile::Lockfile;
use manifest::Manifest;
use pm_lib::solver::{solve, Solution};
use project::find_project_paths;
use resolve::fetch_index;

use pm_lib::index;

pub const USAGE: &str = "Install dependencies.

Usage:
    pm install [options]

Options:
    -h, --help     Display this message.
";

#[derive(Debug, Deserialize)]
pub struct Args {}

pub fn execute(_args: Args) -> Result<(), failure::Error> {
    let project_paths = find_project_paths()?;
    let manifest = Manifest::from_file(&project_paths)?;
    let mut maybe_solution: Option<Solution> = None;
    let mut maybe_new_lockfile: Option<Lockfile> = None;
    if let Some(lockfile) = Lockfile::from_file(&project_paths)? {
        maybe_solution = lockfile.to_solution_if_up_to_date(&manifest.dependencies)?;
    }
    if maybe_solution.is_none() {
        let index = fetch_index()?;
        let dependencies = index::dependencies_from_slice(&manifest.dependencies);
        maybe_solution = Some(solve(&index, &dependencies)?);
        if let Some(ref solution) = maybe_solution {
            maybe_new_lockfile = Some(Lockfile::from_solution(solution, &index)?);
        }
        // TODO use existing lockfile (if any) in resolution
    }
    let solution = maybe_solution.expect("resolved");
    println!("{:?}", solution);
    if let Some(new_lockfile) = maybe_new_lockfile {
        let lockfile_string = format!("{}", new_lockfile);
        fs::write(&project_paths.lockfile, &lockfile_string)?;
        println!("Updating lockfile:\n{}", &lockfile_string);
    } else {
        println!("Lockfile is up to date.");
    }

    // install_to_disk()

    // TODO: read manifest without parsing files section (split up?)
    Ok(())
}
