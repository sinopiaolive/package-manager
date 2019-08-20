use resolve::resolve;
use project::find_project_paths;

pub const USAGE: &str = "Install dependencies.

Usage:
    pm install [options]

Options:
    -h, --help     Display this message.
";

#[derive(Debug, Deserialize)]
pub struct Args {
}

pub fn execute(_args: Args) -> Result<(), failure::Error> {
    let project_paths = find_project_paths()?;
    let solution = resolve(&project_paths)?;

    // TODO
    // if !lockfile || lockfile.not_up_to_date() {
    //     resolve()
    //     write_lock_file()
    // }
    // install_to_disk()

    // TODO: read manifest without parsing files section (split up?)
    Ok(())
}
