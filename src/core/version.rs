use nom::{digit, ErrorKind};
#[allow(unused_imports)]
use nom::IResult::Done;
use std::fmt;
use std::str::FromStr;
use std::str;
use std::clone::Clone;
use regex::Regex;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use std::fmt::Display;
use serde::de::Error;

use self::VersionIdentifier::{Numeric, Alphanumeric};

#[derive(Eq, Hash, Clone)]
pub struct Version {
    pub fields: Vec<u64>,
    pub prerelease: Vec<VersionIdentifier>,
    pub build: Vec<VersionIdentifier>, // TODO Vec?
}

macro_rules! ver {
    ( $( $x:expr ),* ) => {{
        let mut version_parts = Vec::new();
        $(
            version_parts.push($x);
        )*
        Version::new(version_parts, vec![], vec![])
    }};
}

impl Version {
    pub fn new(v: Vec<u64>, p: Vec<VersionIdentifier>, b: Vec<VersionIdentifier>) -> Version {
        Version {
            fields: v,
            prerelease: p,
            build: b,
        }
    }

    pub fn as_string(&self) -> String {
        let mut s = String::new();
        s.push_str(&*self.fields.iter().map(|f| f.to_string()).collect::<Vec<_>>().join("."));
        if self.prerelease.len() > 0 {
            s.push_str("-");
            s.push_str(&*self.prerelease
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join("."));
        }
        if self.build.len() > 0 {
            s.push_str("+");
            s.push_str(&*self.build.iter().map(|f| f.to_string()).collect::<Vec<_>>().join("."));
        }
        s
    }

    pub fn normalise(&self) -> Version {
        let mut trunc = self.fields.clone();
        loop {
            match trunc.pop() {
                None => break,
                Some(0u64) => (),
                Some(i) => {
                    trunc.push(i);
                    break
                }
            }
        }
        if trunc.is_empty() {
            ver!(0)
        } else {
            Version::new(trunc, self.prerelease.clone(), self.build.clone())
        }
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Version) -> bool {
        let v1 = self.normalise();
        let v2 = other.normalise();
        v1.fields == v2.fields && v1.prerelease == v2.prerelease && v1.build == v2.build
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(self.as_string().as_str())
    }
}

impl Deserialize for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        let s = String::deserialize(deserializer)?;
        match version(s.as_bytes()) {
            Done(b"", v) => Ok(v),
            _ => Err(D::Error::custom(format!("{:?} is not a valid version descriptor", s))),
        }
    }
}

// impl PartialEq for Version {
//     fn eq(&self, other: &Version) -> bool {

//     }
// }

// impl Eq for Version {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
#[serde(deny_unknown_fields)]
pub enum VersionIdentifier {
    Numeric(u64),
    Alphanumeric(String),
}

impl Display for VersionIdentifier {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        match *self {
            VersionIdentifier::Numeric(ref n) => write!(f, "{}", n),
            VersionIdentifier::Alphanumeric(ref s) => write!(f, "{}", s),
        }
    }
}



/// Increment the last component of a version.
///
/// For instance, bumping `1` would yield `2`, and bumping `1.2.3.4` would yield `1.2.3.5`.
///
/// This drops all tags from the version.
pub fn bump_last(v: &Version) -> Version {
    let mut v1 = Version::new(v.fields.clone(), vec![], vec![]);
    match v1.fields.pop() {
        None => v1,
        Some(next) => {
            v1.fields.push(next + 1);
            v1
        }
    }
}

/// Increment the first non-zero component, drop the rest.
///
/// This is the effect of Semver's caret operator `^`.
///
/// Eg. `caret_bump(1.2.3)` yields `2`, and `caret_bump(0.0.2.1)` yields `0.0.3`.
/// `caret_bump(1.2.3-beta1)` also yields `2`.
///
/// This drops all tags from the version.
pub fn caret_bump(v: &Version) -> Version {
    let mut parts = Vec::new();
    for next in v.fields.iter() {
        let i = *next;
        if i == 0u64 {
            parts.push(i);
        } else {
            parts.push(i + 1);
            return Version::new(parts, vec![], vec![])
        }
    }
    Version::new(vec![], vec![], vec![])
}

/// If there are two components or less, bump the last one.
/// If there are more than two components, bump the second and drop the rest.
///
/// This is the effect of Semver's tilde operator `~`.
///
/// Eg. `tilde_bump(1.2)` and `tilde_bump(1.2.3)` both yield `1.3`,
/// and `tilde_bump(1)` yields `2`.
///
/// This drops all tags from the version.
pub fn tilde_bump(v: &Version) -> Version {
    bump_last(&Version::new(v.fields.iter().map(|i| *i).take(2).collect(), vec![], vec![]))
}



named!(nat<u64>, map_res!(map_res!(digit, str::from_utf8), to_u64));

named!(pub base_version<Vec<u64>>, separated_nonempty_list!(char!('.'), nat));

named!(version_id<VersionIdentifier>,
       map_res!(map_res!(re_bytes_find!(r"^[a-zA-Z0-9-]+"),
                         str::from_utf8), convert_version_identifier));

fn to_u64(s: &str) -> Result<u64, ()> {
    match u64::from_str(s) {
        Ok(n) => {
            if s.chars().count() > 1 && s.chars().next() == Some('0') {
                Err(())
            } else {
                Ok(n)
            }
        }
        Err(_) => Err(()),
    }
}

fn convert_version_identifier(s: &str) -> Result<VersionIdentifier, ()> {
    let numeric_version_id_re: Regex = Regex::new(r"^[0-9]+$").unwrap();
    if numeric_version_id_re.is_match(s) {
        to_u64(s).map(Numeric)
    } else {
        Ok(Alphanumeric(s.to_string()))
    }
}

named!(ext_version<Vec<VersionIdentifier>>, separated_nonempty_list!(char!('.'), version_id));

// lolsob
named!(empty_vec<Vec<VersionIdentifier>>, value!(Vec::new()));

named!(pre_part<Vec<VersionIdentifier>>, alt_complete!(preceded!(
    char!('-'), return_error!(ErrorKind::Custom(1), ext_version)
) | empty_vec));

named!(build_part<Vec<VersionIdentifier>>, alt_complete!(preceded!(
    char!('+'), return_error!(ErrorKind::Custom(1), ext_version)
) | empty_vec));

named!(pub version<Version>, do_parse!(
    fields: base_version >>
    pre: pre_part >>
    build: build_part >>
    (Version {
        fields: fields,
        prerelease: pre,
        build: build
    })
));



#[test]
fn normalise_version() {
    assert_eq!(Version::normalise(&ver!(1,2,3)), ver!(1,2,3));
    assert_eq!(Version::normalise(&ver!(0,0,0,1,2,0,3,0)), ver!(0,0,0,1,2,0,3));
    assert_eq!(Version::normalise(&ver!(1,2,0,0,0)), ver!(1,2));
    assert_eq!(Version::normalise(&ver!(0,0,0,0,0)), ver!(0));
}

fn version_equality() {
    assert!(ver!(1,2,3) == ver!(1,2,3));
    assert!(ver!(1,2,3,0,0) == ver!(1,2,3));
}

#[test]
fn test_bump_last() {
    assert_eq!(bump_last(&ver!(1,2,3)), ver!(1,2,4));
    assert_eq!(bump_last(&ver!(1,2)), ver!(1,3));
    assert_eq!(bump_last(&ver!(3)), ver!(4));
    assert_eq!(bump_last(&Version::new(vec![1,2,3],
                                       vec![Alphanumeric("beta2".to_string())],
                                       vec![Alphanumeric("lol".to_string())])),
               ver!(1,2,4));
}

#[test]
fn test_caret_bump() {
    assert_eq!(caret_bump(&ver!(1,2,3)), ver!(2));
    assert_eq!(caret_bump(&ver!(0,1,2)), ver!(0,2));
    assert_eq!(caret_bump(&ver!(0,0,3)), ver!(0,0,4));
    assert_eq!(caret_bump(&Version::new(vec![0,1,2,3],
                                        vec![Alphanumeric("beta2".to_string())],
                                        vec![Alphanumeric("lol".to_string())])),
               ver!(0,2));
}

#[test]
fn test_tilde_bump() {
    assert_eq!(tilde_bump(&ver!(1,2,3)), ver!(1,3));
    assert_eq!(tilde_bump(&ver!(1,2)), ver!(1,3));
    assert_eq!(tilde_bump(&ver!(1)), ver!(2));
    assert_eq!(tilde_bump(&Version::new(vec![1,2,3],
                                        vec![Alphanumeric("beta2".to_string())],
                                        vec![Alphanumeric("lol".to_string())])),
               ver!(1,3));
}

#[test]
fn parse_nat() {
    assert_eq!(nat(b"1"), Done(&b""[..], 1));
    assert_eq!(nat(b"123"), Done(&b""[..], 123));
    assert!(nat(b"123456789123456789123456789123456789").is_err());
    assert_eq!(nat(b"123lol"), Done(&b"lol"[..], 123));
    assert!(nat(b"wat").is_err());
}

#[test]
fn parse_version_identifier() {
    assert_eq!(version_id(b"1"), Done(&b""[..], Numeric(1)));
    assert_eq!(version_id(b"-1"), Done(&b""[..], Alphanumeric("-1".to_string())));
    assert_eq!(version_id(b"1a-"), Done(&b""[..], Alphanumeric("1a-".to_string())));
    assert_eq!(version_id(b"1a-_"), Done(&b"_"[..], Alphanumeric("1a-".to_string())));
    assert!(version_id(b"00").is_err());
    assert!(version_id(b"01").is_err());
    assert!(version_id(b"123456789123456789123456789123456789").is_err());
}

#[test]
fn parse_base_version() {
    assert_eq!(base_version(b"1"), Done(&b""[..], vec![1]));
    assert_eq!(base_version(b"1.2.3"), Done(&b""[..], vec![1, 2, 3]));
    assert_eq!(base_version(b"1.2.3.4.5"), Done(&b""[..], vec![1, 2, 3, 4, 5]));
}

#[test]
fn parse_full_version() {
    assert_eq!(version(b"1"), Done(&b""[..], Version::new(vec![1], vec![], vec![])));
    assert_eq!(version(b"1.2.3"), Done(&b""[..], Version::new(vec![1, 2, 3], vec![], vec![])));
    assert_eq!(version(b"1-2+3"),
               Done(&b""[..], Version::new(vec![1], vec![Numeric(2)], vec![Numeric(3)])));
    assert_eq!(version(b"1+3"), Done(&b""[..], Version::new(vec![1], vec![], vec![Numeric(3)])));
    assert_eq!(version(b"1-2"), Done(&b""[..], Version::new(vec![1], vec![Numeric(2)], vec![])));
    assert_eq!(version(b"1.2-2.foo+bar.3"),
               Done(&b""[..], Version::new(vec![1, 2],
                                           vec![Numeric(2), Alphanumeric("foo".to_string())],
                                           vec![Alphanumeric("bar".to_string()), Numeric(3)])));
    assert_eq!(version(b"1.2+3.4.lol"),
               Done(&b""[..], Version::new(vec![1, 2], vec![],
                                           vec![Numeric(3), Numeric(4),
                                                Alphanumeric("lol".to_string())])));
    assert_eq!(version(b"1.3-omg.2"),
               Done(&b""[..], Version::new(vec![1, 3],
                                           vec![Alphanumeric("omg".to_string()), Numeric(2)],
                                           vec![])));
    assert!(version(b"1-123456789123456789123456789123456789").is_err());
    assert!(version(b"1-01").is_err());
    assert!(version(b"1-_not_alphanumeric_").is_err());
    assert!(version(b"1+123456789123456789123456789123456789").is_err());
    assert!(version(b"1+01").is_err());
    assert!(version(b"1+_not_alphanumeric_").is_err());
}
