use nom::IResult::Done;
use self::VersionConstraint::{Exact, Range, Caret, Tilde};
use serde::de::Error;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use version::{Version, version, caret_bump, tilde_bump};
use std::cmp::Ordering;
use std::fmt;
use nom;
use super::error;

#[derive(PartialEq, Eq, Clone, Hash)]
pub enum VersionConstraint {
    Exact(Version),
    Range(Option<Version>, Option<Version>),
    Caret(Version),
    Tilde(Version),
}

impl Serialize for VersionConstraint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_string().as_str())
    }
}

impl<'de> Deserialize<'de> for VersionConstraint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // return Ok(VersionConstraint::Exact(Version::new(vec![1], vec![], vec![])))
        match version_constraint(s.as_bytes()) {
            Done(b"", v) => Ok(v),
            _ => {
                Err(D::Error::custom(format!(
                    "{:?} is not a valid version constraints descriptor",
                    s
                )))
            }
        }
    }
}

impl VersionConstraint {
    pub fn from_str(s: &str) -> Result<VersionConstraint, error::Error> {
        match version_constraint(s.as_bytes()) {
            Done(b"", v) => Ok(v),
            _ => Err(error::Error::Custom(
                format!("invalid version constraint {:?}", s),
            )),
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            &Exact(ref v) => v.as_string(),
            &Range(None, None) => "*".to_string(),
            &Range(Some(ref v), None) => format!(">= {}", v),
            &Range(None, Some(ref v)) => format!("< {}", v),
            &Range(Some(ref v1), Some(ref v2)) => format!(">= {} < {}", v1, v2),
            &Caret(ref v) => format!("^{}", v),
            &Tilde(ref v) => format!("~{}", v),
        }
    }

    pub fn contains(&self, version: &Version) -> bool {
        match self {
            &Exact(ref v) => version == v,
            &Caret(ref v) => contained_in_range(&version, Some(&v), Some(&caret_bump(&v))),
            &Tilde(ref v) => contained_in_range(&version, Some(&v), Some(&tilde_bump(&v))),
            &Range(ref v1, ref v2) => contained_in_range(&version, v1.as_ref(), v2.as_ref()),
        }
    }
}

fn contained_in_range(version: &Version, min: Option<&Version>, max: Option<&Version>) -> bool {
    match (min, max) {
        (None, None) => true,
        (Some(min), None) => version.semver_cmp(min) != Ordering::Less,
        (None, Some(max)) => max_match(version, None, max),
        (Some(min), Some(max)) => {
            version.semver_cmp(min) != Ordering::Less && max_match(version, Some(min), max)
        }
    }
}

/// When testing for membership, the max end of a range is a special case:
/// if it has a pre tag, we match normally. If it doesn't, we need to first
/// strip any pre tag off the version we're testing before matching.
/// This means that `2-pre.1` will be in bounds for `< 2-pre.2` but not
/// for `< 2`. `1.9-pre` would be in bounds for both.
fn max_match(v: &Version, maybe_min: Option<&Version>, max: &Version) -> bool {
    match maybe_min {
        None => v.strip().semver_cmp(max) == Ordering::Less,
        Some(min) if min.strip().semver_cmp(&max.strip()) == Ordering::Less && !max.has_pre() => {
            v.strip().semver_cmp(max) == Ordering::Less
        }
        _ => v.semver_cmp(max) == Ordering::Less,
    }
}

impl fmt::Display for VersionConstraint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl fmt::Debug for VersionConstraint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}


named!(exact_version_constraint<VersionConstraint>, map!(version, VersionConstraint::Exact));

named!(min_constraint<VersionConstraint>, ws!(do_parse!(
    tag!(b">=") >>
        v: version >>
        (Range(Some(v), None))
)));

named!(max_constraint<VersionConstraint>, ws!(do_parse!(
    tag!(b"<") >>
        v: version >>
        (Range(None, Some(v)))
)));

named!(closed_constraint<VersionConstraint>, ws!(do_parse!(
    tag!(b">=") >>
        v1: version >>
        tag!(b"<") >>
        v2: version >>
        (Range(Some(v1), Some(v2)))
)));

named!(open_constraint<VersionConstraint>, ws!(do_parse!(
    tag!(b"*") >>
        (Range(None, None))
)));

named!(caret_constraint<VersionConstraint>, ws!(do_parse!(
    tag!(b"^") >>
        v: version >>
        (Caret(v))
)));

named!(tilde_constraint<VersionConstraint>, ws!(do_parse!(
    tag!(b"~") >>
        v: version >>
        (Tilde(v))
)));

named!(pub version_constraint_unchecked<VersionConstraint>,
       alt_complete!(open_constraint
                     | caret_constraint
                     | tilde_constraint
                     | closed_constraint
                     | max_constraint
                     | min_constraint
                     | exact_version_constraint));

fn version_constraint(input: &[u8]) -> nom::IResult<&[u8], VersionConstraint> {
    match version_constraint_unchecked(input) {
        Done(i, _) if i.len() > 0 => nom::IResult::Error(nom::ErrorKind::Eof),
        Done(_, Range(Some(ref v1), Some(ref v2))) if v1.semver_cmp(v2) != Ordering::Less => {
            nom::IResult::Error(nom::ErrorKind::Custom(1))
        }
        r @ _ => r,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use version::VersionIdentifier;
    use test::{ver, range};

    #[test]
    fn parse_exact_constraint() {
        assert_eq!(version_constraint(b"1.0"),
                   Done(&b""[..], Exact(Version::new(vec![1,0], vec![], vec![]))));
    }

    #[test]
    fn parse_range_constraint() {
        assert_eq!(version_constraint(b">=1.0"),
                   Done(&b""[..], Range(Some(Version::new(vec![1,0], vec![], vec![])), None)));
        assert_eq!(version_constraint(b" >=1.0 "),
                   Done(&b""[..], Range(Some(Version::new(vec![1,0], vec![], vec![])), None)));
        assert_eq!(version_constraint(b">= 1.0"),
                   Done(&b""[..], Range(Some(Version::new(vec![1,0], vec![], vec![])), None)));
        assert_eq!(version_constraint(b"   >=          1.0"),
                   Done(&b""[..], Range(Some(Version::new(vec![1,0], vec![], vec![])), None)));
        assert_eq!(version_constraint(b">=1 .0"), nom::IResult::Error(nom::ErrorKind::Eof));
        assert_eq!(version_constraint(b"<1.0"),
                   Done(&b""[..], Range(None, Some(Version::new(vec![1,0], vec![], vec![])))));

        assert_eq!(version_constraint(b">=1.0 <2.0"),
                   Done(&b""[..], Range(Some(Version::new(vec![1,0], vec![], vec![])),
                                        Some(Version::new(vec![2,0], vec![], vec![])))));
        assert_eq!(version_constraint(b" >= 1.0 < 2.0 "),
                   Done(&b""[..], Range(Some(Version::new(vec![1,0], vec![], vec![])),
                                        Some(Version::new(vec![2,0], vec![], vec![])))));
        assert_eq!(version_constraint(b">=1.0<2.0"),
                   Done(&b""[..], Range(Some(Version::new(vec![1,0], vec![], vec![])),
                                        Some(Version::new(vec![2,0], vec![], vec![])))));
        assert_eq!(version_constraint(b">=2.0<1.0"),
                   nom::IResult::Error(nom::ErrorKind::Custom(1)));
    }

    #[test]
    fn parse_open_constraint() {
        assert_eq!(version_constraint(b"*"),
                   Done(&b""[..], Range(None, None)));
    }

    #[test]
    fn parse_caret_constraint() {
        let v1 = Version::new(
            vec![0,0,0,0,8,0],
            vec![VersionIdentifier::Alphanumeric("rc1".to_string())],
            vec![VersionIdentifier::Alphanumeric("wtf".to_string())],
        );
        assert_eq!(version_constraint(b"^0.0.0.0.8.0-rc1+wtf"),
                   Done(&b""[..], Caret(v1)));
    }

    #[test]
    fn parse_tilde_constraint() {
        let v1 = Version::new(
            vec![1,2,3,4,0],
            vec![VersionIdentifier::Alphanumeric("rc1".to_string())],
            vec![VersionIdentifier::Alphanumeric("wtf".to_string())],
        );
        assert_eq!(version_constraint(b"~1.2.3.4.0-rc1+wtf"),
                   Done(&b""[..], Tilde(v1)));
    }

    #[test]
    fn constraint_as_string() {
        assert_eq!(Exact(ver("1.2.3.0-beta.0+foo")).as_string(), "1.2.3.0-beta.0+foo");

        assert_eq!(Range(None, None).as_string(), "*");
        assert_eq!(Range(Some(ver!(1,2,3)), None).as_string(), ">= 1.2.3");
        assert_eq!(Range(None, Some(ver!(1,2,3))).as_string(), "< 1.2.3");
        assert_eq!(Range(Some(ver!(1,2,3)), Some(ver!(2))).as_string(),
                   ">= 1.2.3 < 2");

        assert_eq!(Caret(ver!(1,2,3,0)).as_string(), "^1.2.3.0");
        assert_eq!(Tilde(ver!(1,2,3,0)).as_string(), "~1.2.3.0");
    }

    #[test]
    fn constraint_contains() {
        assert!(range("1.2.3.0-beta").contains(&ver("1.2.3-beta")));
        assert!(!range("1.2.4").contains(&ver("1.2.3")));

        assert!(range("*").contains(&ver("1.2.3")));
        assert!(range(">=1.2").contains(&ver("1.2.3")));
        assert!(range("<3").contains(&ver("1.2.3")));
        assert!(range(">=1.1.5 <3.0.1.2").contains(&ver("1.2.3")));
        assert!(!range("1").contains(&ver("1.2.3")));
        assert!(!range(">=3").contains(&ver("1.2.3")));
        assert!(!range(">=2 <3").contains(&ver("1.2.3")));
        assert!(!range(">=1 <2-pre").contains(&ver("2")));
        assert!(!range(">=1 <2").contains(&ver("2-beta")));
        assert!(range(">=1 <2-pre").contains(&ver("2-beta")));
        assert!(range(">=2-pre <2").contains(&ver("2-pre")));
        assert!(!range(">=2-pre <2").contains(&ver("2")));

        assert!(range("^1.2.0").contains(&ver("1.2")));
        assert!(range("^1.2.0-pre.1").contains(&ver("1.2")));
        assert!(!range("^1.2.0-pre.1").contains(&ver("1.2.0-pre.0")));
        assert!(range("^1.2.0-pre.1").contains(&ver("1.3-beta")));
        assert!(!range("^1.2.0-pre.1").contains(&ver("2-beta")));
        assert!(range("^0-pre.1").contains(&ver("0.9")));
        assert!(!range("^0-pre.1").contains(&ver("1")));
        assert!(range("^0.0-pre.1").contains(&ver("0.9")));
        assert!(range("^0.0.0.1.2.3").contains(&ver("0.0.0.1.3")));
        assert!(!range("^0.0.0.1.2.3").contains(&ver("0.0.0.2")));

        assert!(range("~0-pre.1").contains(&ver("0.2-pre.0")));
        assert!(!range("~0-pre.1").contains(&ver("1.0")));
        assert!(range("~0.0-pre.1").contains(&ver("0.0.1-pre.0")));
        assert!(!range("~0.0-pre.1").contains(&ver("0.2-pre.0")));
        assert!(range("~1.2.3").contains(&ver("1.2.4")));
        assert!(!range("~1.2.3").contains(&ver("1.3-beta.1")));
        assert!(range("~1.2").contains(&ver("1.2.4")));
        assert!(!range("~1.2").contains(&ver("1.3")));
        assert!(!range("~1.2").contains(&ver("1.3-beta")));
    }
}
