#[allow(unused_imports)] use nom::IResult::Done;
use self::VersionConstraint::{Exact, Range};
use version::{Version, base_version, version, bump_last, caret_bump, tilde_bump};
#[allow(unused_imports)] use version::VersionIdentifier;
use std::fmt;

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum VersionConstraint {
    Exact(Version),
    Range(Option<Version>, Option<Version>),
}

impl VersionConstraint {
    pub fn gteq(v: Version) -> VersionConstraint {
        Range(Some(v), None)
    }

    pub fn lt(v: Version) -> VersionConstraint {
        Range(None, Some(v))
    }

    pub fn range(v1: Version, v2: Version) -> VersionConstraint {
        Range(Some(v1), Some(v2))
    }

    pub fn exact(v: Version) -> VersionConstraint {
        Exact(v)
    }

    pub fn all() -> VersionConstraint {
        Range(None, None)
    }

    pub fn as_string(&self) -> String {
        match self {
            &Exact(ref v) => v.as_string(),
            &Range(None, None) => "*".to_string(),
            &Range(Some(ref v), None) => format!(">= {}", v),
            &Range(None, Some(ref v)) => format!("< {}", v),
            &Range(Some(ref v1), Some(ref v2)) => format!(">= {} < {}", v1, v2)
        }
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

named!(x_constraint<VersionConstraint>, ws!(do_parse!(
    v: base_version >>
        alt!(tag!(b".x") | tag!(b".X") | tag!(b".*")) >>
        ({
            let vmin = Version::new(v, vec![], vec![]);
            let vmax = bump_last(&vmin);
            (Range(Some(vmin), Some(vmax)))
        })
)));

named!(caret_constraint<VersionConstraint>, ws!(do_parse!(
    tag!(b"^") >>
        vmin: version >>
        ({
            let vmax = caret_bump(&vmin);
            (Range(Some(vmin), Some(vmax)))
        })
)));

named!(tilde_constraint<VersionConstraint>, ws!(do_parse!(
    tag!(b"~") >>
        vmin: version >>
        ({
            let vmax = tilde_bump(&vmin);
            (Range(Some(vmin), Some(vmax)))
        })
)));

named!(pub version_constraint<VersionConstraint>,
       alt_complete!(open_constraint
                     | caret_constraint
                     | tilde_constraint
                     | x_constraint
                     | closed_constraint
                     | max_constraint
                     | min_constraint
                     | exact_version_constraint));



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
    assert_eq!(version_constraint(b">=1 .0"),
               Done(&b".0"[..], Range(Some(Version::new(vec![1], vec![], vec![])), None)));
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
}

#[test]
fn parse_open_constraint() {
    assert_eq!(version_constraint(b"*"),
               Done(&b""[..], Range(None, None)));
}

#[test]
fn parse_x_constraint() {
    assert_eq!(version_constraint(b"1.2.x"),
               Done(&b""[..], VersionConstraint::range(ver!(1, 2), ver!(1, 3))));
    assert_eq!(version_constraint(b"2.x"),
               Done(&b""[..], VersionConstraint::range(ver!(2), ver!(3))));
}

#[test]
fn parse_caret_constraint() {
    assert_eq!(version_constraint(b"^1.2.3"),
               Done(&b""[..], VersionConstraint::range(ver!(1,2,3), ver!(2))));
    assert_eq!(version_constraint(b"^0.5.2"),
               Done(&b""[..], VersionConstraint::range(ver!(0,5,2), ver!(0,6))));
    let v1 = Version::new(vec![0,0,0,0,8],
                          vec![VersionIdentifier::Alphanumeric("rc1".to_string())],
                          vec![VersionIdentifier::Alphanumeric("wtf".to_string())]);
    assert_eq!(version_constraint(b"^0.0.0.0.8-rc1+wtf"),
               Done(&b""[..], VersionConstraint::range(v1, ver!(0,0,0,0,9))));
}

#[test]
fn parse_tilde_constraint() {
    assert_eq!(version_constraint(b"~1.2.3"),
               Done(&b""[..], VersionConstraint::range(ver!(1,2,3), ver!(1,3))));
    assert_eq!(version_constraint(b"~1.2"),
               Done(&b""[..], VersionConstraint::range(ver!(1,2), ver!(1,3))));
    let v1 = Version::new(vec![1,2,3,4],
                          vec![VersionIdentifier::Alphanumeric("rc1".to_string())],
                          vec![VersionIdentifier::Alphanumeric("wtf".to_string())]);
    assert_eq!(version_constraint(b"~1.2.3.4-rc1+wtf"),
               Done(&b""[..], VersionConstraint::range(v1, ver!(1,3))));
}

#[test]
fn constraint_to_string() {
    assert_eq!(VersionConstraint::all().as_string(), "*".to_string());
    assert_eq!(VersionConstraint::gteq(ver!(1,2,3)).as_string(), ">= 1.2.3".to_string());
    assert_eq!(VersionConstraint::lt(ver!(1,2,3)).as_string(), "< 1.2.3".to_string());
    assert_eq!(VersionConstraint::range(ver!(1,2,3), ver!(2)).as_string(), ">= 1.2.3 < 2".to_string());
}
