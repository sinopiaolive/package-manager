use std;
use reqwest;

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
