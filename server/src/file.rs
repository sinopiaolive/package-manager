use schema::files;

#[derive(Insertable, AsChangeset, Queryable, Identifiable, Associations, Debug)]
#[table_name = "files"]
#[primary_key(namespace, name)]
pub struct File {
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub data: Vec<u8>,
}
