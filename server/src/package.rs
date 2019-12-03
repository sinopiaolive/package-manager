use std::time::SystemTime;

use crate::schema::{package_owners, package_releases, packages, release_dependencies};
use crate::user::UserRecord;

#[derive(Identifiable, Queryable, Insertable, AsChangeset, Debug)]
#[table_name = "packages"]
#[primary_key(namespace, name)]
pub struct Package {
    pub namespace: String,
    pub name: String,
    pub deleted: Option<String>,
    pub deleted_on: Option<SystemTime>,
}

#[derive(Identifiable, Queryable, Insertable, AsChangeset, Associations, Debug)]
#[table_name = "package_owners"]
#[primary_key(namespace, name, user_id)]
#[belongs_to(UserRecord, foreign_key = "user_id")]
pub struct PackageOwner {
    pub namespace: String,
    pub name: String,
    pub user_id: String,
    // need ordering
    pub added_time: SystemTime,
}

#[derive(Insertable, Identifiable, Associations, Debug)]
#[belongs_to(UserRecord, foreign_key = "publisher")]
#[table_name = "package_releases"]
#[primary_key(namespace, name, version)]
pub struct Release {
    pub namespace: String,
    pub name: String,
    pub version: String,

    pub description: String,
    pub authors: Vec<String>,
    pub keywords: Vec<String>,
    pub homepage_url: Option<String>,
    pub repository_type: Option<String>,
    pub repository_url: Option<String>,
    pub bugs_url: Option<String>,

    pub license: Option<String>,
    pub license_file_name: Option<String>,
    pub license_file_contents: Option<String>,

    pub manifest_file_name: Option<String>,
    pub manifest_file_contents: Option<String>,

    pub readme_name: Option<String>,
    pub readme_contents: Option<String>,

    pub publisher: String,
}

#[derive(Insertable, AsChangeset, Identifiable, Queryable, Associations, Debug)]
// We'd like to have a #[belongs_to] attribute to link this to Release, but
// belongs_to doesn't support composite keys yet.
#[table_name = "release_dependencies"]
#[primary_key(namespace, name, version, ordering)]
pub struct Dependency {
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub ordering: i32,
    pub dependency_namespace: String,
    pub dependency_name: String,
    pub dependency_version_constraint: String,
}
