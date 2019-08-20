use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

use brotli;
use indicatif::{ProgressBar, ProgressStyle};
use rmp_serde::encode;
use tar;

use pm_lib::publication_request::PublicationRequest;

use failure;
use io::ProgressIO;
use project::{find_project_paths, ProjectPaths};
use manifest::Manifest;
use registry::post;

pub const USAGE: &str = "Publish a package to the registry.

Usage:
    pm publish [options]

Options:
    -v, --verbose  List files being added to the release.
    -q, --quiet    Don't print any descriptive messages.
    --dry-run      Run through the procedure, but don't actually publish.
    -h, --help     Display this message.
";

#[derive(Debug, Deserialize)]
pub struct Args {
    flag_verbose: bool,
    flag_dry_run: bool,
    flag_quiet: bool,
}

fn make_progress(msg: &str, len: usize, quiet: bool) -> ProgressBar {
    let bar = if quiet {
        ProgressBar::hidden()
    } else {
        ProgressBar::new(len as u64)
    };
    bar.set_style(
        ProgressStyle::default_bar()
            .template("{msg} {bar:40} {percent:>3}% {bytes:>8}/{total_bytes} {eta:>6} left"),
    );
    bar.set_message(msg);
    bar
}

pub fn execute(args: Args) -> Result<(), failure::Error> {
    let project_paths = find_project_paths()?;
    let manifest = Manifest::from_file(&project_paths)?;

    if !args.flag_quiet {
        println!("Building release {}-{}...", manifest.name, manifest.version);
    }

    let tar = build_archive(
        manifest.files.iter().map(PathBuf::from).collect(),
        &project_paths,
        &args,
    )?;

    let mut tar_br = vec![];
    let compress_progress = make_progress("Compressing:", tar.len(), args.flag_quiet);

    let mut brotli_encoder_params = brotli::enc::BrotliEncoderInitParams();
    brotli_encoder_params.quality = 9;
    brotli_encoder_params.lgwin = 22; // log2 of window size
    brotli::BrotliCompress(
        &mut ProgressIO::reader_from(tar, |c, _| compress_progress.set_position(c as u64)),
        &mut tar_br,
        &brotli_encoder_params,
    )?;
    compress_progress.finish_and_clear();

    let req = PublicationRequest {
        namespace: manifest.name.namespace.clone(),
        name: manifest.name.name.clone(),
        version: manifest.version.clone(),

        description: manifest.description.clone(),
        authors: manifest.authors.clone(),
        keywords: manifest.keywords.clone(),
        homepage_url: manifest.homepage.clone(),
        repository: None, // TODO
        bugs_url: None,   // TODO
        license: manifest.license.clone(),
        license_file: None, // TODO
        manifest: None,     // TODO
        readme: None,       // TODO

        dependencies: manifest.dependencies.clone(),

        tar_br,
    };

    let payload = encode::to_vec_named(&req)?;
    let upload_progress = Arc::new(make_progress("Uploading:", payload.len(), args.flag_quiet));
    let up = upload_progress.clone(); // lifetime management shenanigans
    let body = ProgressIO::reader_from(payload, move |c, _| up.set_position(c as u64));

    if !args.flag_dry_run {
        post::<(), _>("publish", ordmap![], body)??;
    };
    upload_progress.finish_and_clear();

    if !args.flag_quiet {
        if args.flag_dry_run {
            println!("Dry run successful")
        } else {
            println!(
                "Package {} version {} has been published",
                manifest.name, manifest.version
            );
        }
    }

    Ok(())
}

fn build_archive(
    files: Vec<PathBuf>,
    project_paths: &ProjectPaths,
    args: &Args,
) -> Result<Vec<u8>, failure::Error> {
    let mut tar = tar::Builder::new(Vec::new());
    for local_path in files {
        if args.flag_verbose {
            let repr = local_path
                .to_str()
                .unwrap_or_else(|| panic!("non-representable file name: {:?}", local_path));
            if args.flag_quiet {
                println!("{}", repr)
            } else {
                println!("    {}", repr)
            }
        }
        let mut path = project_paths.root.clone();
        path.push(local_path.clone());
        let mut file = File::open(path)?;
        tar.append_file(local_path, &mut file)?;
    }
    tar.finish()?;
    Ok(tar.into_inner()?)
}
