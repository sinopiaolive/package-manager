use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use rmp_serde::{self, encode};
use serde_json;

use version::Version;
use constraint::VersionConstraint;
use package::PackageName;



quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Io(err: ::std::io::Error) {
            cause(err)
            description(err.description())
            from()
        }
        Custom(err: String) {
            description(err)
            from()
        }
        FromRMP(err: rmp_serde::decode::Error) {
            cause(err)
            description(err.description())
            from()
        }
        ToRMP(err: rmp_serde::encode::Error) {
            cause(err)
            description(err.description())
            from()
        }
    }
}

pub type Index = BTreeMap<PackageName, Package>;
pub type Package = BTreeMap<Version, Dependencies>;
pub type Dependencies = BTreeMap<PackageName, VersionConstraint>;

pub fn read_index(path: &Path) -> Result<Arc<Index>, Error> {
    let mut f = File::open(path)?;
    let mut s = Vec::new();
    f.read_to_end(&mut s)?;
    Ok(Arc::new(rmp_serde::from_slice(&s)?))
}

pub fn write_to<W>(i: &Index, wr: &mut W) -> Result<(), Error>
where
    W: Write,
{
    encode::write(wr, i)?;
    Ok(())
}

pub fn write_index(i: &Index, path: &Path) -> Result<(), Error> {
    fs::create_dir_all(path)?;
    let mut f = File::create(path)?;
    write_to(i, &mut f)
}

pub fn read_json<P>(path: P) -> Result<Arc<Index>, Error>
where
    P: AsRef<Path>,
{
    let mut f = File::open(path)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    serde_json::from_str(&s)
        .map_err(|e| Error::Custom(format!("{}", e)))
        .map(Arc::new)
}


#[cfg(test)]
mod unit_test {
    use super::*;
    use test::Bencher;

    #[bench] #[ignore]
    fn read_cargo_index(_b: &mut Bencher) {
        read_index(::std::path::Path::new("../client/test/cargo.rmp")).unwrap();
    }
}
