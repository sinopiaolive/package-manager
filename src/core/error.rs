use std;
use toml;

use manifest::PackageName;
use version::Version;

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
        PackageMissing(pkg: PackageName) {
            display(me) -> ("package missing: {}", pkg)
            description("package missing")
        }
        VersionMissing(pkg: PackageName, ver: Version) {
            display(me) -> ("version missing: {} {}", pkg, ver)
            description("version missing")
        }
        Io(err: std::io::Error) {
            cause(err)
            description(err.description())
            from()
        }
        FromToml(err: toml::de::Error) {
            cause(err)
            description(err.description())
            from()
        }
        ToToml(err: toml::ser::Error) {
            cause(err)
            description(err.description())
            from()
        }
        Other(err: Box<std::error::Error>) {
            cause(&**err)
            description(err.description())
        }
    }
}
