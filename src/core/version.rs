use nom::{digit, ErrorKind};
use nom::IResult::Done;
use std::fmt;
use std::str::FromStr;
use std::str;
use std::clone::Clone;
use regex::Regex;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use std::fmt::Display;
use serde::de::Error;
use std::cmp::{Ord, PartialOrd, Ordering};
use std::hash::{Hash, Hasher};
use super::error;

use self::VersionIdentifier::{Numeric, Alphanumeric};

#[derive(Eq, Clone)]
pub struct Version {
    pub fields: Vec<u64>,
    pub prerelease: Vec<VersionIdentifier>,
    pub build: Vec<VersionIdentifier>,
}

impl Version {
    pub fn new(v: Vec<u64>, p: Vec<VersionIdentifier>, b: Vec<VersionIdentifier>) -> Version {
        Version {
            fields: v,
            prerelease: p,
            build: b,
        }
    }

    pub fn from_str(s: &str) -> Result<Version, error::Error> {
        match version(s.as_bytes()) {
            Done(b"", v) => Ok(v),
            _ => Err(error::Error::Custom(
                format!("invalid version string {:?}", s),
            )),
        }
    }

    pub fn pre(&self, pre: &[&str]) -> Version {
        Version::new(
            self.fields.clone(),
            pre.iter()
                .map(|s| convert_version_identifier(s).unwrap())
                .collect(),
            self.build.clone(),
        )
    }

    pub fn strip(&self) -> Version {
        Version::new(self.fields.clone(), vec![], vec![])
    }

    pub fn has_pre(&self) -> bool {
        !self.prerelease.is_empty()
    }

    pub fn as_string(&self) -> String {
        let mut s = String::new();
        s.push_str(&*self.fields
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join("."));
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
            s.push_str(&*self.build
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join("."));
        }
        s
    }

    pub fn normalized_fields(&self) -> &[u64] {
        let mut max = self.fields.len();
        while max > 1 && self.fields[max - 1] == 0 {
            max -= 1;
        }
        &self.fields[0..max]
    }

    pub fn semver_cmp(&self, other: &Version) -> Ordering {
        match self.normalized_fields().cmp(&other.normalized_fields()) {
            Ordering::Equal => {
                match (self.has_pre(), other.has_pre()) {
                    (false, true) => Ordering::Greater,
                    (true, false) => Ordering::Less,
                    (false, false) => Ordering::Equal,
                    (true, true) => self.prerelease.cmp(&other.prerelease),
                }
            }
            a => a,
        }
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Version) -> bool {
        self.normalized_fields() == other.normalized_fields() &&
            self.prerelease == other.prerelease
    }
}

impl Hash for Version {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.normalized_fields().hash(state);
        self.prerelease.hash(state);
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
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_string().as_str())
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match version(s.as_bytes()) {
            Done(b"", v) => Ok(v),
            _ => Err(D::Error::custom(
                format!("{:?} is not a valid version descriptor", s),
            )),
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Version) -> Ordering {
        match (self.has_pre(), other.has_pre()) {
            (true, false) => Ordering::Greater, // sort prerelease versions last
            (false, true) => Ordering::Less,
            _ => other.semver_cmp(&self), // sort highest versions first
        }
    }
}


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

impl PartialOrd for VersionIdentifier {
    fn partial_cmp(&self, other: &VersionIdentifier) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VersionIdentifier {
    fn cmp(&self, other: &VersionIdentifier) -> Ordering {
        match (self, other) {
            (&Numeric(_), &Alphanumeric(_)) => Ordering::Less,
            (&Alphanumeric(_), &Numeric(_)) => Ordering::Greater,
            (&Numeric(ref a), &Numeric(ref b)) => a.cmp(b),
            (&Alphanumeric(ref a), &Alphanumeric(ref b)) => a.cmp(b),
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
            return Version::new(parts, vec![], vec![]);
        }
    }
    Version::new(vec![1], vec![], vec![]) // version 0
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
    bump_last(&Version::new(
        v.fields.iter().map(|i| *i).take(2).collect(),
        vec![],
        vec![],
    ))
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


#[cfg(test)]
mod test {
    use super::*;
    use test::ver;

    #[test]
    fn weird_constructors() {
        assert_eq!(ver("0.0.0.5.2").as_string(), "0.0.0.5.2".to_string());
        assert_eq!(ver("1.2.3").pre(&["rc1"][..]).as_string(), "1.2.3-rc1".to_string());
    }

    #[test]
    fn normalized_fields() {
        assert_eq!(&ver("1.2.3").normalized_fields(), &[1, 2, 3]);
        assert_eq!(&ver("0.0.0.1.2.0.3.0").normalized_fields(), &[0, 0, 0, 1, 2, 0, 3]);
        assert_eq!(&ver("1.2.0.0.0-beta.1+foo").normalized_fields(), &[1, 2]);
        assert_eq!(&ver("0.0.0.0.0").normalized_fields(), &[0]);
        assert_eq!(&ver("0").normalized_fields(), &[0]);
    }

    #[test]
    fn version_equality() {
        assert!(ver("1.2.3") == ver("1.2.3"));
        assert!(ver("1.2.3") != ver("1.2.4"));
        assert!(ver("1.2.3.0.0") == ver("1.2.3"));
        assert!(ver("1.2.3") == ver("1.2.3.0.0"));
        assert!(ver("1.2.3-pre.0") == ver("1.2.3-pre.0"));
        assert!(ver("1.2.3-pre.1") != ver("1.2.3-pre.2"));
        assert!(ver("1.2.3-pre.1+foo") == ver("1.2.3-pre.1+bar"));
    }

    #[test]
    fn semver_ordering() {
        assert_eq!(ver("1.2.3").semver_cmp(&ver("1.2.3")), Ordering::Equal);
        assert_eq!(ver("1.2.3.0").semver_cmp(&ver("1.2.3.0.0")), Ordering::Equal);
        assert_eq!(ver("1.2").semver_cmp(&ver("1.2.3")), Ordering::Less);
        assert_eq!(ver("1.2.3").semver_cmp(&ver("1.2")), Ordering::Greater);
        assert_eq!(ver("1.2.3").semver_cmp(&ver("1.2.4")), Ordering::Less);
        assert_eq!(ver("1.3.2").semver_cmp(&ver("1.2.3")), Ordering::Greater);
        assert_eq!(ver("1.3.2-rc1").semver_cmp(&ver("1.2.3")), Ordering::Greater);
        assert_eq!(ver("1.2.3-rc1").semver_cmp(&ver("1.2.3")), Ordering::Less);
        assert_eq!(ver("1.3.2").semver_cmp(&ver("1.2.3-rc1")), Ordering::Greater);
        assert_eq!(ver("1.2.3").semver_cmp(&ver("1.2.3-rc1")), Ordering::Greater);
        assert_eq!(ver("1.2.3-rc1").semver_cmp(&ver("1.2.3-rc2")), Ordering::Less);
        assert_eq!(ver("1.2.3-rc1").semver_cmp(&ver("1.2.3-rc1.foo")), Ordering::Less);
        assert_eq!(ver("1.2.3-rc1").semver_cmp(&ver("1.2.3-beta1")), Ordering::Greater);
    }

    #[test]
    fn priority_ordering() {
        assert_eq!(ver("1.2.3.0").cmp(&ver("1.2.3")), Ordering::Equal);
        assert_eq!(ver("1.2").cmp(&ver("1.2.3")), Ordering::Greater);
        assert_eq!(ver("1.2.3").cmp(&ver("1.2")), Ordering::Less);
        assert_eq!(ver("1.3-rc").cmp(&ver("1.2.3")), Ordering::Greater);
        assert_eq!(ver("1.2.3").cmp(&ver("1.3-rc")), Ordering::Less);
        assert_eq!(ver("1.2.3-rc").cmp(&ver("1.2.3")), Ordering::Greater);
        assert_eq!(ver("1.2.3").cmp(&ver("1.2.3-rc")), Ordering::Less);
        assert_eq!(ver("1.3-rc").cmp(&ver("1.3-rc.2")), Ordering::Greater);
    }

    #[test]
    fn test_bump_last() {
        assert_eq!(bump_last(&ver("1.2.3.0")), ver("1.2.3.1"));
        assert_eq!(bump_last(&ver("1.2.3")), ver("1.2.4"));
        assert_eq!(bump_last(&ver("1.2")), ver("1.3"));
        assert_eq!(bump_last(&ver("3")), ver("4"));
        assert_eq!(bump_last(&ver("1.2.3-beta2+lol")), ver("1.2.4"));
    }

    #[test]
    fn test_caret_bump() {
        assert_eq!(caret_bump(&ver("1.2.3")), ver("2"));
        assert_eq!(caret_bump(&ver("0.1.2")), ver("0.2"));
        assert_eq!(caret_bump(&ver("0.0.3")), ver("0.0.4"));
        assert_eq!(caret_bump(&ver("0.0")), ver("1"));
        assert_eq!(caret_bump(&ver("0")), ver("1"));
        assert_eq!(caret_bump(&ver("0.1.2.3-beta2+lol")), ver("0.2"));
        assert_eq!(caret_bump(&ver("0-beta2+lol")), ver("1"));
    }

    #[test]
    fn test_tilde_bump() {
        assert_eq!(tilde_bump(&ver("1.0.0")), ver("1.1"));
        assert_eq!(tilde_bump(&ver("1.0")), ver("1.1"));
        assert_eq!(tilde_bump(&ver("1")), ver("2"));
        assert_eq!(tilde_bump(&ver("1.2.3-beta2+lol")), ver("1.3"));
        assert_eq!(tilde_bump(&ver("0-beta2+lol")), ver("1"));
        assert_eq!(tilde_bump(&ver("0.0-beta2+lol")), ver("0.1"));
        assert_eq!(tilde_bump(&ver("0.0.0-beta2+lol")), ver("0.1"));
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
        assert_eq!(version(b"1+3"),
                   Done(&b""[..], Version::new(vec![1], vec![], vec![Numeric(3)])));
        assert_eq!(version(b"1-2"),
                   Done(&b""[..], Version::new(vec![1], vec![Numeric(2)], vec![])));
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
}
