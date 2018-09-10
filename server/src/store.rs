use std::env;
use std::str::FromStr;
use std::time::SystemTime;

use diesel;
use diesel::expression::dsl::now;
use diesel::pg::expression::extensions::IntervalDsl;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind::UniqueViolation;
use diesel::result::Error::DatabaseError;

use data_encoding::BASE64;

use error::{Error, Res};
use file::File;
use package::{Package, PackageOwner, Release};
use user::{User, UserRecord};

use schema::{files, login_sessions, package_owners, package_releases, packages, users};

#[allow(dead_code)]
#[derive(Queryable)]
pub struct LoginSession {
    token: String,
    callback: String,
    stamp: SystemTime,
}

#[derive(Insertable)]
#[table_name = "login_sessions"]
pub struct NewLoginSession {
    token: String,
    callback: String,
}

pub struct Store {
    db_url: String,
}

impl Store {
    pub fn new() -> Res<Store> {
        Ok(Store {
            db_url: env::var("DATABASE_URL")?,
        })
    }

    pub fn db(&self) -> Res<PgConnection> {
        Ok(PgConnection::establish(&self.db_url)?)
    }

    pub fn register_login(&self, token: &str, callback: &str) -> Res<()> {
        let db = self.db()?;
        if BASE64.decode(token.as_bytes()).is_err() {
            return Err(Error::InvalidLoginState(token.to_string()));
        }
        diesel::insert_into(login_sessions::table)
            .values(&NewLoginSession {
                token: token.to_string(),
                callback: callback.to_string(),
            }).execute(&db)?;
        Ok(())
    }

    pub fn validate_login(&self, token: &str) -> Res<String> {
        let db = self.db()?;
        if BASE64.decode(token.as_bytes()).is_err() {
            return Err(Error::InvalidLoginState(token.to_string()));
        }
        let results: Vec<LoginSession> = login_sessions::table
            .filter(login_sessions::token.eq(token))
            .filter(login_sessions::stamp.gt(now - 30.minutes()))
            .load(&db)?;
        match results.into_iter().next() {
            None => Err(Error::InvalidLoginState(token.to_string())),
            Some(session) => {
                diesel::delete(login_sessions::table.filter(login_sessions::token.eq(token)))
                    .execute(&db)?;
                Ok(session.callback)
            }
        }
    }

    pub fn update_user(&self, user: &UserRecord) -> Res<()> {
        let db = self.db()?;
        match self.get_user(&user.user()?) {
            Ok(_) => {
                diesel::update(users::table.filter(users::id.eq(&user.id)))
                    .set((
                        users::name.eq(&user.name),
                        users::email.eq(&user.email),
                        users::avatar.eq(&user.avatar),
                    )).execute(&db)?;
            }
            Err(_) => {
                diesel::insert_into(users::table)
                    .values(user)
                    .execute(&db)?;
            }
        }
        Ok(())
    }

    pub fn get_user(&self, user: &User) -> Res<UserRecord> {
        let db = self.db()?;
        let results: Vec<UserRecord> = users::table
            .filter(users::id.eq(user.to_string()))
            .load(&db)?;
        match results.into_iter().next() {
            None => Err(Error::UnknownUser(user.to_string())),
            Some(user) => Ok(user),
        }
    }

    pub fn get_package(&self, namespace: &str, name: &str) -> Res<Package> {
        let db = self.db()?;
        let results = packages::table
            .filter(
                packages::namespace
                    .eq(&namespace)
                    .and(packages::name.eq(&name)),
            ).load(&db)?;
        match results.into_iter().next() {
            None => Err(Error::UnknownPackage(namespace.to_owned(), name.to_owned())),
            Some(pkg) => Ok(pkg),
        }
    }

    pub fn insert_package(&self, namespace: &str, name: &str, owner: &User) -> Res<()> {
        let db = self.db()?;
        match diesel::insert_into(packages::table)
            .values(&Package {
                namespace: namespace.to_owned(),
                name: name.to_owned(),
                deleted: None,
                deleted_on: None,
            }).execute(&db)
        {
            Ok(_) => self.add_package_owner(namespace, name, owner),
            Err(_) => Ok(()),
        }
    }

    pub fn get_package_owners(&self, namespace: &str, name: &str) -> Res<Vec<User>> {
        let db = self.db()?;
        let results: Vec<PackageOwner> = package_owners::table
            .filter(
                package_owners::namespace
                    .eq(namespace)
                    .and(package_owners::name.eq(name)),
            ).load(&db)?;
        results.iter().map(|o| User::from_str(&o.user_id)).collect()
    }

    pub fn add_package_owner(&self, namespace: &str, name: &str, owner: &User) -> Res<()> {
        let db = self.db()?;
        diesel::insert_into(package_owners::table)
            .values(&PackageOwner {
                namespace: namespace.to_owned(),
                name: name.to_owned(),
                user_id: owner.to_string(),
                added_time: SystemTime::now(),
            }).execute(&db)?;
        Ok(())
    }

    pub fn remove_package_owner(&self, namespace: &str, name: &str, owner: &User) -> Res<()> {
        let db = self.db()?;
        diesel::delete(
            package_owners::table.filter(
                package_owners::namespace.eq(namespace).and(
                    package_owners::name
                        .eq(name)
                        .and(package_owners::user_id.eq(&owner.to_string())),
                ),
            ),
        ).execute(&db)?;
        Ok(())
    }

    pub fn get_releases(&self, namespace: &str, name: &str) -> Res<Vec<Release>> {
        let db = self.db()?;
        Ok(package_releases::table
            .filter(
                package_releases::namespace
                    .eq(namespace)
                    .and(package_releases::name.eq(name)),
            ).load(&db)?)
    }

    pub fn add_release(&self, release: &Release, data: &[u8]) -> Res<()> {
        let db = self.db()?;
        db.transaction(|| {
            diesel::insert_into(package_releases::table)
                .values(release)
                .execute(&db)
                .map_err(|err| match err {
                    DatabaseError(UniqueViolation, _) => Error::ReleaseAlreadyExists(
                        release.namespace.clone(),
                        release.name.clone(),
                        release.version.clone(),
                    ),
                    e => Error::from(e),
                })?;
            diesel::insert_into(files::table)
                .values(&File {
                    namespace: release.namespace.to_owned(),
                    name: release.name.to_owned(),
                    version: release.version.to_owned(),
                    data: data.to_owned(),
                }).execute(&db)?;
            Ok(())
        })
    }

    pub fn get_file(&self, namespace: &str, name: &str, version: &str) -> Res<File> {
        let db = self.db()?;
        let results: Vec<File> = files::table
            .filter(files::namespace.eq(namespace).and(files::name.eq(name)).and(files::version.eq(version)))
            .load(&db)?;
        match results.into_iter().next() {
            None => Err(Error::UnknownRelease(namespace.to_string(), name.to_string(), version.to_string())),
            Some(file) => Ok(file),
        }
    }
}
