use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::{self, Connection};
use dotenv;
use std::env;
use std::time::SystemTime;

use auth::AuthSource;
use error::Error;
use package::{Package, PackageOwner, Release};
use schema::{package_owners, package_releases, packages, users};
use search::{search_db, SearchResult};
use user::{User, UserRecord};

embed_migrations!("migrations");

fn get_db<F>(fixture: F) -> PgConnection
where
    F: Fn(&PgConnection) -> Result<(), Error>,
{
    dotenv::from_filename(".env").ok();
    let db = PgConnection::establish(
        &env::var("DATABASE_URL").expect("no DATABASE_URL env var defined"),
    ).expect("can't connect to database");
    db.begin_test_transaction()
        .expect("failed to start test transaction");
    fixture(&db).expect("failed to setup test fixtures");
    db
}

fn test_user_fixture(db: &PgConnection) -> Result<(), Error> {
    diesel::insert_into(users::table)
        .values(&UserRecord::new(
            &User::new(AuthSource::Test, "user"),
            "Test User",
            "test@test.com",
            "https://media.giphy.com/media/Gx2vpQi2WPToc/giphy.gif",
        )).execute(db)?;
    Ok(())
}

fn insert_package(
    db: &PgConnection,
    name: &str,
    owner: &str,
    versions: &[&str],
) -> Result<(), Error> {
    diesel::insert_into(packages::table)
        .values(&Package {
            namespace: "test".to_string(),
            name: name.to_string(),
            deleted: None,
            deleted_on: None,
        }).execute(db)?;
    diesel::insert_into(package_owners::table)
        .values(&PackageOwner {
            namespace: "test".to_string(),
            name: name.to_string(),
            user_id: owner.to_string(),
            added_time: SystemTime::now(),
        }).execute(db)?;
    for version in versions {
        diesel::insert_into(package_releases::table)
            .values(&Release {
                namespace: "test".to_string(),
                name: name.to_string(),
                version: version.to_string(),
                publisher: owner.to_string(),
                publish_time: SystemTime::now(),
                artifact_url: "data:text/plain,lol".to_string(),
                description: name.to_string(),
                license: Some("GPL-3.0+".to_string()),
                license_file: None,
                keywords: vec![name.to_string()],
                manifest: String::new(),
                readme_filename: Some("README".to_string()),
                readme: Some(name.to_string()),
                deprecated: false,
                deprecated_by: None,
                deprecated_on: None,
                deleted: None,
                deleted_on: None,
            }).execute(db)?;
    }
    Ok(())
}

fn packages_fixture(db: &PgConnection) -> Result<(), Error> {
    test_user_fixture(db)?;
    insert_package(db, "left-pad", "test:user", &["1.0", "1.1", "2.0"])?;
    insert_package(db, "right-pad", "test:user", &["1.0", "1.1", "2.0"])?;
    insert_package(db, "profunctor-optics", "test:user", &["1.0", "1.1", "2.0"])?;
    Ok(())
}

#[test]
fn test_package_search() {
    let db = get_db(packages_fixture);
    assert_eq!(
        vec![SearchResult {
            name: "left-pad".to_string(),
            version: "2.0".to_string(),
            publisher: "test:user".to_string(),
            description: "left-pad".to_string(),
        },],
        search_db(&db, "test", vec!["left".to_string(), "pad".to_string()]).unwrap()
    );
    assert_eq!(
        vec![
            SearchResult {
                name: "left-pad".to_string(),
                version: "2.0".to_string(),
                publisher: "test:user".to_string(),
                description: "left-pad".to_string(),
            },
            SearchResult {
                name: "right-pad".to_string(),
                version: "2.0".to_string(),
                publisher: "test:user".to_string(),
                description: "right-pad".to_string(),
            },
        ],
        search_db(&db, "test", vec!["pad".to_string()]).unwrap()
    );
}
