use lockfile::Lockfile;
use manifest::Manifest;
use project::find_project_paths;
use resolve::resolve;
use solver::Solution;

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
    if let Some(lockfile) = Lockfile::from_file(&project_paths)? {
        maybe_solution = lockfile.to_solution_if_up_to_date(&manifest.dependencies)?;
    }
    if maybe_solution.is_none() {
        maybe_solution = Some(resolve(&manifest)?);
        // TODO use existing lockfile (if any) in resolution
        // TODO update lockfile if not up-to-date
    }
    let solution = maybe_solution.expect("resolved");
    println!("{:?}", solution);

    // install_to_disk()

    // TODO: read manifest without parsing files section (split up?)
    Ok(())
}
