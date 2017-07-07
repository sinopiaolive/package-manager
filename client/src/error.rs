use std;

use pm_lib::manifest;
use pm_lib::index;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Message(err: &'static str) {
            description(err)
            from()
        }
        Custom(err: String) {
            description(err)
            from()
        }
        Io(err: std::io::Error) {
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
