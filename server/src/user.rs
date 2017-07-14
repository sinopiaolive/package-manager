#[derive(Serialize, Deserialize)]
pub struct User {
    pub password: String,
    pub secret: String,
}

pub fn make_secret() -> String {
    "snape kills dumbledore".to_string()
}
