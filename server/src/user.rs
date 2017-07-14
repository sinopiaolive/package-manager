use rand::os::OsRng;
use rand::Rng;

use error::Res;

#[derive(Serialize, Deserialize)]
pub struct User {
    pub password: String,
    pub secret: String,
}

pub fn generate_secret() -> Res<String> {
    Ok(OsRng::new()?.gen_iter::<char>().take(64).collect())
}
