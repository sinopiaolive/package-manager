use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

use brotli;
use console::Style;
use indicatif::{ProgressBar, ProgressStyle};
use rmp_serde::encode;
use tar;

use pm_lib::manifest::Manifest;

use failure;
use io::ProgressIO;
use project::{find_project_dir, read_manifest};
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
    let manifest = read_manifest()?;

    if !args.flag_quiet {
        println!("Building release {}-{}...", manifest.name, manifest.version);
    }

    let tar = build_archive(manifest.files.iter().map(PathBuf::from).collect(), &args)?;

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

    let req = Manifest {
        namespace: manifest.name.namespace.clone(),
        name: manifest.name.name.clone(),
        version: manifest.version.clone(),
        description: manifest.description.clone(),
        license: manifest.license.clone(),
        readme: manifest.readme.clone(),
        keywords: manifest.keywords.clone(),
        manifest: String::new(),
        tar_br,
    };

    let payload = encode::to_vec_named(&req)?;
    let upload_progress = Arc::new(make_progress("Uploading:", payload.len(), args.flag_quiet));
    let up = upload_progress.clone(); // lifetime management shenanigans
    let body = ProgressIO::reader_from(payload, move |c, _| up.set_position(c as u64));

    let res = if !args.flag_dry_run {
        post::<(), _>("publish", ordmap![], body)?
    } else {
        Ok(())
    };
    upload_progress.finish_and_clear();

    if !args.flag_quiet {
        if args.flag_dry_run {
            println!("Seems to work!")
        } else {
            match res {
                Ok(_) => println!(
                    "Package {} version {} has been published!",
                    manifest.name, manifest.version
                ),
                Err(msg) => println!("{}: {}", Style::new().red().bold().apply_to("ERROR"), msg),
            }
        }
    }

    Ok(())
}

fn build_archive(files: Vec<PathBuf>, args: &Args) -> Result<Vec<u8>, failure::Error> {
    let project_path = find_project_dir()?;
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
        let mut path = project_path.clone();
        path.push(local_path.clone());
        let mut file = File::open(path)?;
        tar.append_file(local_path, &mut file)?;
    }
    tar.finish()?;
    Ok(tar.into_inner()?)
}
