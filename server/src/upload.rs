use std::io::Read;

use rmp_serde::decode;
use tar;
use brotli;

use pm_lib::manifest::Manifest;

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

                description: manifest.description.clone(),
                authors: manifest.authors.clone(),
                keywords: manifest.keywords.clone(),
                homepage_url: manifest.homepage_url.clone(),
                repository_type: manifest.repository.as_ref().map(|r| r.type_.clone()),
                repository_url: manifest.repository.as_ref().map(|r| r.url.clone()),
                bugs_url: manifest.bugs_url.clone(),

                license: manifest.license.clone(),
                license_file_name: manifest.license_file.as_ref().map(|named_text_file| named_text_file.name.clone()),
                license_file_contents: manifest.license_file.as_ref().map(|named_text_file| named_text_file.contents.clone()),

                manifest_file_name: manifest.manifest.as_ref().map(|named_text_file| named_text_file.name.clone()),
                manifest_file_contents: manifest.manifest.as_ref().map(|named_text_file| named_text_file.contents.clone()),

                readme_name: manifest.readme.as_ref().map(|named_text_file| named_text_file.name.clone()),
                readme_contents: manifest.readme.as_ref().map(|named_text_file| named_text_file.contents.clone()),

                publisher: user.id.clone(),
            };
            store.add_release(&release, manifest.tar_br.as_slice())?;
            Ok(())
        }
    }
}
