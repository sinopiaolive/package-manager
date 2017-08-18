use std::env;
use std::time::SystemTime;

use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::expression::dsl::now;
use diesel::pg::expression::extensions::MicroIntervalDsl;

use data_encoding::BASE64;

use error::{Res, Error};
use user::{User, UserRecord};

use schema::{users, login_sessions};

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
        Ok(Store { db_url: env::var("DATABASE_URL")? })
    }

    pub fn db(&self) -> Res<PgConnection> {
        Ok(PgConnection::establish(&self.db_url)?)
    }

    pub fn register_login(&self, token: &str, callback: &str) -> Res<()> {
        let db = self.db()?;
        if BASE64.decode(token.as_bytes()).is_err() {
            return Err(Error::InvalidLoginState(token.to_string()));
        }
        diesel::insert(&NewLoginSession {
            token: token.to_string(),
            callback: callback.to_string(),
        }).into(login_sessions::table)
            .execute(&db)?;
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
                diesel::delete(login_sessions::table.filter(
                    login_sessions::token.eq(token),
                )).execute(&db)?;
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
                    ))
                    .execute(&db)?;
            }
            Err(_) => {
                diesel::insert(user).into(users::table).execute(&db)?;
            }
        }
        Ok(())
    }

    pub fn get_user(&self, user: &User) -> Res<UserRecord> {
        let db = self.db()?;
        let results: Vec<UserRecord> = users::table.filter(users::id.eq(user.to_string())).load(
            &db,
        )?;
        match results.into_iter().next() {
            None => Err(Error::UnknownUser(user.to_string())),
            Some(user) => Ok(user),
        }
    }
}
