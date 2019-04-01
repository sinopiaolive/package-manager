use std::io::Read;
use std::time::SystemTime;

use rmp_serde::decode;
use tar;
use brotli;

use pm_lib::manifest::{License, Manifest};

use store::Store;
use user::User;
use error::{Res, Error};
use package::Release;

fn validate_manifest(_manifest: &Manifest) -> Res<()> {
    // TODO pls to validate manifest here
    Ok(())
}

fn validate_archive<R: Read>(mut reader: R) -> Res<()> {
    for entry in tar::Archive::new(brotli::Decompressor::new(&mut reader, 4096))
        .entries()
        .map_err(|_| Error::InvalidArtifact("not a Brotli compressed TAR archive"))?
    {
        entry.map_err(|_| {
            Error::InvalidArtifact("malformed TAR archive")
        })?;
    }
    Ok(())
}

pub fn process_upload<R: Read>(store: &Store, user: &User, reader: R) -> Res<()> {
    let manifest: Manifest = decode::from_read(reader)?;
    store.insert_package(
        &manifest.namespace,
        &manifest.name,
        user,
    )?;
    let owners = store.get_package_owners(
        &manifest.namespace,
        &manifest.name,
    )?;
    let filename = format!("{}-{}.tar.br", manifest.name, manifest.version);
    let url = format!("/files/{}/{}", manifest.namespace, filename);
    match owners.iter().find(|o| *o == user) {
        None => Err(Error::AccessDenied(
            manifest.namespace.clone(),
            manifest.name.clone(),
            user.clone(),
        )),
        Some(_) => {
            validate_manifest(&manifest)?;
            validate_archive(&mut manifest.tar_br.as_slice())?;
            let release = Release {
                namespace: manifest.namespace.clone(),
                name: manifest.name.clone(),
                version: manifest.version.to_string(),
                publisher: user.to_string(),
                publish_time: SystemTime::now(),
                artifact_url: url.clone(),
                description: manifest.description.to_string(),
                license: match manifest.license {
                    License::SPDX(ref tag) => Some(tag.clone()),
                    License::SPDXAndFile(ref tag, _) => Some(tag.clone()),
                    _ => None,
                },
                license_file: match manifest.license {
                    License::File(ref file) => Some(file.clone()),
                    License::SPDXAndFile(_, ref file) => Some(file.clone()),
                    _ => None,
                },
                keywords: manifest.keywords.clone(),
                manifest: manifest.manifest.clone(),
                readme_filename: match manifest.readme {
                    Some((ref filename, _)) => Some(filename.clone()),
                    None => None,
                },
                readme: match manifest.readme {
                    Some((_, ref content)) => Some(content.clone()),
                    None => None,
                },
                deprecated: false,
                deprecated_by: None,
                deprecated_on: None,
                deleted: None,
                deleted_on: None,
            };
            store.add_release(&release, manifest.tar_br.as_slice())?;
            Ok(())
        }
    }
}
