use std;
use reqwest;
use toml;

use pm_lib::manifest;
use pm_lib::index;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Message(err: String) {
            description(err)
            from()
            from(s: &'static str) -> (s.to_string())
        }
        Server(err: String) {
            description(err)
        }
        Io(err: std::io::Error) {
            cause(err)
            description(err.description())
            from()
        }
        Http(err: reqwest::Error) {
            cause(err)
            description(err.description())
            from()
        }
        Json(err: ::serde_json::error::Error) {
            cause(err)
            description(err.description())
            from()
        }
        TomlDe(err: toml::de::Error) {
            cause(err)
            description(err.description())
            from()
        }
        TomlSer(err: toml::ser::Error) {
            cause(err)
            description(err.description())
            from()
        }
        FromManifestError(err: manifest::Error) {
            cause(err)
            description(err.description())
            from()
        }
        FromIndexError(err: index::Error) {
            cause(err)
            description(err.description())
            from()
        }
    }
}
