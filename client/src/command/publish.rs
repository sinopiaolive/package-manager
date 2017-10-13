use std::io::Read;
use std::fs::File;
use std::path::PathBuf;

use tar;
use brotli;
use rmp_serde::encode;

use pm_lib::manifest::Manifest;

use error::Error;
use project::{read_manifest, find_project_dir};
use registry::post;

pub const USAGE: &'static str = "Publish a package to the registry.

Usage:
    pm publish [options]

Options:
    -h, --help     Display this message.
";

#[derive(Debug, Deserialize)]
pub struct Args {}

pub fn execute(_args: Args) -> Result<(), Error> {
    let manifest = read_manifest()?;

    let tar = build_archive(manifest.files.iter().map(|f| PathBuf::from(f)).collect())?;
    let artifact = compress(&mut tar.as_slice())?;

    let req = Manifest {
        namespace: manifest.name.namespace.clone(),
        name: manifest.name.name.clone(),
        version: manifest.version.clone(),
        description: manifest.description.clone(),
        license: manifest.license.clone(),
        readme: manifest.readme.clone(),
        keywords: manifest.keywords.clone(),
        manifest: String::new(),
        data: artifact,
    };

    let res = post("publish", map![], encode::to_vec_named(&req)?)?;

    println!("Server says: {}", res);

    Ok(())
}

fn build_archive(files: Vec<PathBuf>) -> Result<Vec<u8>, Error> {
    let project_path = find_project_dir()?;
    let mut tar = tar::Builder::new(Vec::new());
    for local_path in files {
        let mut path = project_path.clone();
        path.push(local_path.clone());
        let mut file = File::open(path)?;
        tar.append_file(local_path, &mut file)?;
    }
    tar.finish()?;
    Ok(tar.into_inner()?)
}

fn compress<R: Read>(reader: &mut R) -> Result<Vec<u8>, Error> {
    let mut out = vec![];
    brotli::BrotliCompress(reader, &mut out, 9, 22)?;
    Ok(out)
}
