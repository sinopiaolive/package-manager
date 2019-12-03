use std::str::FromStr;
use std::time::SystemTime;

use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::Error::NotFound;

use data_encoding::BASE64;

use crate::error::{Error, Res};
use crate::package::{Package, PackageOwner};
use crate::user::{User, UserRecord};

use crate::schema::{files, login_sessions, package_owners, packages, users};

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

// This struct is a wrapper around a thread-pooled connection to the Postgres
// server. It would be called "RegistryDbConn" if we went by the sample code in
// the Rocket guides. It must only have a single data field, but we can freely
// add our own methods, which we do in the impl below.
#[database("registry")]
pub struct Store(rocket_contrib::databases::diesel::PgConnection);

impl Store {
    pub fn db(&self) -> &PgConnection {
        &self.0
    }

    pub fn register_login(&self, token: &str, callback: &str) -> Res<()> {
        let db = self.db();
        if BASE64.decode(token.as_bytes()).is_err() {
            return Err(Error::InvalidLoginState(token.to_string()));
        }
        diesel::insert_into(login_sessions::table)
            .values(&NewLoginSession {
                token: token.to_string(),
                callback: callback.to_string(),
            })
            .execute(db)?;
        Ok(())
    }

    pub fn validate_login(&self, token: &str) -> Res<String> {
        let db = self.db();
        if BASE64.decode(token.as_bytes()).is_err() {
            return Err(Error::InvalidLoginState(token.to_string()));
        }
        let session: LoginSession =
            diesel::delete(login_sessions::table.filter(login_sessions::token.eq(token)))
                .get_result(db)
                .map_err(|err| match err {
                    NotFound => Error::InvalidLoginState(token.to_string()),
                    e => Error::from(e),
                })?;
        Ok(session.callback)
    }

    pub fn update_user(&self, user: &UserRecord) -> Res<()> {
        let db = self.db();
        assert!(
            user.id.contains(':'),
            "user_record.id must be namespaced to prevent collisions between authentication providers"
        );
        diesel::insert_into(users::table)
            .values(user)
            .on_conflict(users::id)
            .do_update()
            .set(user)
            .execute(db)?;
        Ok(())
    }

    pub fn get_user(&self, user: &User) -> Res<UserRecord> {
        let db = self.db();
        let user = users::table
            .filter(users::id.eq(user.to_string()))
            .get_result(db)
            .map_err(|err| match err {
                NotFound => Error::UnknownUser(user.to_string()),
                e => Error::from(e),
            })?;
        Ok(user)
    }

    pub fn get_package(&self, namespace: &str, name: &str) -> Res<Option<Package>> {
        let db = self.db();
        let pkg = packages::table
            .filter(
                packages::namespace
                    .eq(&namespace)
                    .and(packages::name.eq(&name)),
            )
            .get_result(db)
            .optional()?;
        Ok(pkg)
    }

    pub fn get_package_owners(&self, namespace: &str, name: &str) -> Res<Vec<User>> {
        let db = self.db();
        let results: Vec<PackageOwner> = package_owners::table
            .filter(
                package_owners::namespace
                    .eq(namespace)
                    .and(package_owners::name.eq(name)),
            )
            .load(db)?;
        results.iter().map(|o| User::from_str(&o.user_id)).collect()
    }

    pub fn remove_package_owner(&self, namespace: &str, name: &str, owner: &User) -> Res<()> {
        let db = self.db();
        diesel::delete(
            package_owners::table.filter(
                package_owners::namespace.eq(namespace).and(
                    package_owners::name
                        .eq(name)
                        .and(package_owners::user_id.eq(&owner.to_string())),
                ),
            ),
        )
        .execute(db)?;
        Ok(())
    }

    pub fn get_tar_br(&self, namespace: &str, name: &str, version: &str) -> Res<Vec<u8>> {
        let db = self.db();
        let tar_br = files::table
            .select(files::data)
            .filter(
                files::namespace
                    .eq(namespace)
                    .and(files::name.eq(name))
                    .and(files::version.eq(version)),
            )
            .order(files::id.desc())
            .limit(1)
            .get_result::<Vec<u8>>(db)
            .map_err(|err| match err {
                NotFound => Error::UnknownRelease(
                    namespace.to_string(),
                    name.to_string(),
                    version.to_string(),
                ),
                e => Error::from(e),
            })?;
        Ok(tar_br)
    }
}
