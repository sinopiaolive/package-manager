use std::time::SystemTime;

use schema::files;

#[derive(Insertable, Queryable, Identifiable, Associations, Debug)]
#[table_name = "files"]
#[primary_key(namespace, name)]
pub struct File {
    pub namespace: String,
    pub name: String,
    pub data: Vec<u8>,
    pub uploaded_on: SystemTime,
}
