#![allow(dead_code)]

use std;
use reqwest;
use toml;
use rmp_serde;

use files::GlobError;
use git::GitError;
use git2;
use pm_lib::index;

use manifest_parser;

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
        MessagePack(err: rmp_serde::encode::Error) {
            cause(err)
            description(err.description())
            from()
        }
        ManifestParser(err: manifest_parser::Error) {
            display("While processing the manifest:\n{}", err)
            from()
        }
        Glob(err: GlobError) {
            cause(err)
            display("{}", err)
        }
        Libgit2(err: git2::Error) {
            cause(err)
            display("{}", err)
            from()
        }
        Git(err: GitError) {
            cause(err)
            display("{}", err)
            from()
        }
        FromIndexError(err: index::Error) {
            cause(err)
            description(err.description())
            from()
        }
    }
}
