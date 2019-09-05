use reqwest::{self, Method};
use pm_lib::index::Index;

use REGISTRY_URL;

// This module should probably be renamed or merged into another module.

pub fn fetch_index() -> Result<Index, ::failure::Error> {
    let http = reqwest::Client::new();
    let req = http.request(
        Method::GET,
        &format!("{}/index", REGISTRY_URL)
    );
    let mut res = req.send()?;

    if res.status().is_success() {
        Ok(::serde_json::from_reader(res)?)
    } else {
        bail!("Error: {}", &res.text()?);
    }
}
