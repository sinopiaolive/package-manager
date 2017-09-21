use diesel;
use diesel::prelude::*;
use diesel::types::Text;
use diesel::pg::types::sql_types::Array;

use store::Store;
use error::Error;

sql_function!(package_search, package_search_t, (ns: Text, q: Array<Text>) -> Text);

pub fn search_db(db: &PgConnection, ns: &str, query: Vec<String>) -> Result<Vec<String>, Error> {
    Ok(diesel::select(package_search(ns, query)).load(db)?)
}

pub fn search(store: &Store, ns: &str, query: Vec<String>) -> Result<Vec<String>, Error> {
    search_db(&store.db()?, ns, query)
}
