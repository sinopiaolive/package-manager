use reqwest::{self, Method};
use pm_lib::index;
use pm_lib::index::Index;

use REGISTRY_URL;
use project::read_manifest;
use solver::{Solution, solve};

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

pub fn resolve() -> Result<Solution, ::failure::Error> {
    let manifest = read_manifest()?;
    let index = fetch_index()?;
    let dependencies = index::dependencies_from_slice(&manifest.dependencies);
    let solution = solve(&index, &dependencies)?;
    println!("{:?}", solution);
    Ok(solution)
}
