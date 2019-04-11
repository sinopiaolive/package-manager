use std::io::Read;

use rmp_serde::decode;
use tar;
use brotli;

use pm_lib::publication_request::PublicationRequest;

use store::Store;
use user::User;
use error::{Res, Error};
use package;

fn validate_archive<R: Read>(mut reader: R) -> Res<()> {
    // TODO validate file names, content length
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
    let pr: PublicationRequest = decode::from_read(reader)?;
    store.insert_package(
        &pr.namespace,
        &pr.name,
        user,
    )?;
    let owners = store.get_package_owners(
        &pr.namespace,
        &pr.name,
    )?;
    match owners.iter().find(|o| *o == user) {
        None => Err(Error::AccessDenied(
            pr.namespace.clone(),
            pr.name.clone(),
            user.clone(),
        )),
        Some(_) => {
            // TODO validate metadata
            validate_archive(&mut pr.tar_br.as_slice())?;
            let release = package::Release {
                namespace: pr.namespace.clone(),
                name: pr.name.clone(),
                version: pr.version.to_string(),

                description: pr.description.clone(),
                authors: pr.authors.clone(),
                keywords: pr.keywords.clone(),
                homepage_url: pr.homepage_url.clone(),
                repository_type: pr.repository.as_ref().map(|r| r.type_.clone()),
                repository_url: pr.repository.as_ref().map(|r| r.url.clone()),
                bugs_url: pr.bugs_url.clone(),

                license: pr.license.clone(),
                license_file_name: pr.license_file.as_ref().map(|named_text_file| named_text_file.name.clone()),
                license_file_contents: pr.license_file.as_ref().map(|named_text_file| named_text_file.contents.clone()),

                manifest_file_name: pr.manifest.as_ref().map(|named_text_file| named_text_file.name.clone()),
                manifest_file_contents: pr.manifest.as_ref().map(|named_text_file| named_text_file.contents.clone()),

                readme_name: pr.readme.as_ref().map(|named_text_file| named_text_file.name.clone()),
                readme_contents: pr.readme.as_ref().map(|named_text_file| named_text_file.contents.clone()),

                publisher: user.id.clone(),
            };
            let dependencies = pr.dependencies.iter().enumerate().map(|(index, dep)|
                package::Dependency {
                    namespace: pr.namespace.clone(),
                    name: pr.name.clone(),
                    version: pr.version.to_string(),

                    ordering: index as i32,

                    dependency_namespace: dep.namespace.clone(),
                    dependency_name: dep.name.clone(),
                    dependency_version_constraint: dep.version_constraint.to_string(),
                }
            ).collect::<Vec<package::Dependency>>();
            store.add_release(&release, &dependencies, pr.tar_br.as_slice())?;
            Ok(())
        }
    }
}
