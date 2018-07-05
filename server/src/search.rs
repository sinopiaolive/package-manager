use diesel::expression::sql;
use diesel::pg::types::sql_types::Array;
use diesel::prelude::*;
use diesel::types::Text;

use pm_lib::version::Version;

use im::OrdMap as Map;

use error::Error;
use store::Store;

#[derive(Queryable, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct SearchResult {
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub description: String,
}

pub fn search_db(
    db: &PgConnection,
    ns: &str,
    query: Vec<String>,
) -> Result<Vec<SearchResult>, Error> {
    let search = sql::<(Text, Text, Text, Text)>(
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
        .bind::<Array<Text>, _>(query);
    let result = search.load::<SearchResult>(db)?;
    Ok(group_by_semver(result))
}

pub fn search(store: &Store, ns: &str, query: Vec<String>) -> Result<Vec<SearchResult>, Error> {
    search_db(&store.db()?, ns, query)
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
