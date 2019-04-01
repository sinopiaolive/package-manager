use diesel::pg::types::sql_types::Array;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;

use pm_lib::version::Version;

use im::OrdMap as Map;

use error::Error;
use store::Store;

#[derive(QueryableByName, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct SearchResult {
    #[sql_type = "Text"]
    pub name: String,
    #[sql_type = "Text"]
    pub version: String,
    #[sql_type = "Text"]
    pub publisher: String,
    #[sql_type = "Text"]
    pub description: String,
}

pub fn search_db(
    db: &PgConnection,
    ns: &str,
    query: Vec<String>,
) -> Result<Vec<SearchResult>, Error> {
    let result: Vec<SearchResult> = sql_query(
        "
select package_releases.name,
       package_releases.version,
       package_releases.publisher,
       package_releases.description,
       package_releases.publish_time
from package_releases, (
  select package_search($1, $2)) as result
    where result.package_search = package_releases.name
      and package_releases.namespace = $1;
",
    ).bind::<Text, _>(ns)
    .bind::<Array<Text>, _>(query)
    .get_results(db)?;
    Ok(group_by_semver(result))
}

pub fn search(store: &Store, ns: &str, query: Vec<String>) -> Result<Vec<SearchResult>, Error> {
    search_db(&store.db(), ns, query)
}

fn group_by_semver(results: Vec<SearchResult>) -> Vec<SearchResult> {
    let mut groups = Map::<String, Map<Version, SearchResult>>::new();
    for result in results {
        groups.entry(result.name.clone()).or_default().insert(
            Version::from_str(&result.version).expect("illegal version string during search"),
            result.clone(),
        );
    }
    groups
        .values()
        .map(|m| m.get_min().unwrap().1.clone())
        .collect()
}
