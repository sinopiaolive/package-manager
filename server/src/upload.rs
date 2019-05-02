use std::io::Read;
use std::time::SystemTime;

use rmp_serde::decode;
use tar;
use brotli;
use diesel::prelude::*;
use diesel::result::Error::DatabaseError;
use diesel::result::DatabaseErrorKind;
use pm_lib::publication_request::PublicationRequest;

use file::File;
use store::Store;
use schema::{files, packages, package_owners, package_releases, release_dependencies};
use user::User;
use error::{Res, Error};
use package;
use package::{Package, PackageOwner};

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
    let db = store.db();
    db.build_transaction().serializable().run(|| {
        let pr: PublicationRequest = decode::from_read(reader)?;
        if store.get_package(&pr.namespace, &pr.name)?.is_some() {
            let owners = store.get_package_owners(
                &pr.namespace,
                &pr.name,
            )?;
            if !owners.iter().any(|o| o == user) {
                return Err(Error::AccessDenied(
                    pr.namespace.clone(),
                    pr.name.clone(),
                    user.clone(),
                ));
            }
        } else {
            // Add new package. This logic implicitly relies on uniqueness
            // constraints and transaction semantics. If any of these break, it
            // might allow people to add themselves as owners to other people's
            // packages by racing the initial release of the package. We should
            // make it more robust.
            diesel::insert_into(packages::table)
                .values(&Package {
                    namespace: pr.namespace.clone(),
                    name: pr.name.clone(),
                    deleted: None,
                    deleted_on: None,
                })
                .execute(db)?;
            diesel::insert_into(package_owners::table)
                .values(&PackageOwner {
                    namespace: pr.namespace.clone(),
                    name: pr.name.clone(),
                    user_id: user.to_string(),
                    added_time: SystemTime::now(),
                })
                .execute(db)?;
        }

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

            publisher: format!("{}", user).to_string(),
        };
        let dependencies = pr.dependencies.iter().enumerate().map(|(index, dep)|
            package::Dependency {
                namespace: pr.namespace.clone(),
                name: pr.name.clone(),
                version: pr.version.to_string(),

                ordering: index as i32,

                dependency_namespace: dep.package_name.namespace.clone(),
                dependency_name: dep.package_name.name.clone(),
                dependency_version_constraint: dep.version_constraint.to_string(),
            }
        ).collect::<Vec<package::Dependency>>();

        diesel::insert_into(package_releases::table)
            .values(&release)
            .execute(db)
            .map_err(|err| match err {
                DatabaseError(DatabaseErrorKind::UniqueViolation, _) => Error::ReleaseAlreadyExists(
                    release.namespace.clone(),
                    release.name.clone(),
                    release.version.clone(),
                ),
                e => Error::from(e),
            })?;
        diesel::insert_into(release_dependencies::table)
            .values(&dependencies)
            .execute(db)?;
        diesel::insert_into(files::table)
            .values(&File {
                namespace: release.namespace.to_owned(),
                name: release.name.to_owned(),
                version: release.version.to_owned(),
                data: pr.tar_br.clone(),
            })
            .execute(db)?;
        Ok(())
    })
}
