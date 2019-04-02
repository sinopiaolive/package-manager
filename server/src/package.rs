use std::time::SystemTime;

use schema::{packages, package_owners, package_releases, release_dependencies};
use user::UserRecord;

#[derive(Identifiable, Queryable, Insertable, AsChangeset, Associations, Debug)]
#[table_name = "packages"]
#[primary_key(namespace, name)]
pub struct Package {
    pub namespace: String,
    pub name: String,
    pub deleted: Option<String>,
    pub deleted_on: Option<SystemTime>
}

#[derive(Identifiable, Queryable, Insertable, AsChangeset, Associations, Debug)]
#[table_name = "package_owners"]
#[primary_key(namespace, name, user_id)]
#[belongs_to(UserRecord, foreign_key = "user_id")]
pub struct PackageOwner {
    pub namespace: String,
    pub name: String,
    pub user_id: String,
    pub added_time: SystemTime
}

#[derive(Insertable, AsChangeset, Identifiable, Queryable, Associations, Debug)]
#[belongs_to(UserRecord, foreign_key = "publisher")]
#[table_name = "package_releases"]
#[primary_key(namespace, name, version)]
pub struct Release {
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub publish_time: SystemTime,
    pub artifact_url: String,
    pub description: String,
    pub license: Option<String>,
    pub license_file: Option<String>,
    pub keywords: Vec<String>,
    pub manifest: String,
    pub readme_filename: Option<String>,
    pub readme: Option<String>,
    pub deprecated: bool,
    pub deprecated_by: Option<String>,
    pub deprecated_on: Option<SystemTime>,
    pub deleted: Option<String>,
    pub deleted_on: Option<SystemTime>
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
    pub dependency_version_constraint: String
}
