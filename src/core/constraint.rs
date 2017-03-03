#[allow(unused_imports)] use nom::IResult::Done;
use self::VersionConstraint::{Exact, Range, Empty};
use serde::de::Error;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use version::{Version, base_version, version, bump_last, caret_bump, tilde_bump};
#[allow(unused_imports)]
use version::VersionIdentifier;
use std::fmt;

#[derive(PartialEq, Eq)]
pub enum VersionConstraint {
    Exact(Version),
    Range(Option<Version>, Option<Version>),
    Empty,
}

impl Serialize for VersionConstraint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(self.as_string().as_str())
    }
}

impl Deserialize for VersionConstraint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        let s = String::deserialize(deserializer)?;
        match version_constraint(s.as_bytes()) {
            Done(b"", v) => Ok(v),
            _ => Err(D::Error::custom(format!("{:?} is not a valid version constraints descriptor", s))),
        }
    }
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

    pub fn all() -> VersionConstraint {
        Range(None, None)
    }

    pub fn as_string(&self) -> String {
        match self {
            &Exact(ref v) => v.as_string(),
            &Range(None, None) => "*".to_string(),
            &Range(Some(ref v), None) => format!(">= {}", v),
            &Range(None, Some(ref v)) => format!("< {}", v),
            &Range(Some(ref v1), Some(ref v2)) => format!(">= {} < {}", v1, v2),
            &Empty => "∅".to_string(),
        }
    }

    pub fn contains(&self, v: &Version) -> bool {
        match self {
            &Empty => false,
            &Range(None, None) => true,
            &Exact(ref o) => v == o,
            &Range(Some(ref min), None) => v >= min,
            &Range(None, Some(ref max)) => v < max,
            &Range(Some(ref min), Some(ref max)) => v >= min && v < max,
        }
    }

    pub fn and(&self, other: &VersionConstraint) -> VersionConstraint {
        match (self, other) {
            (&Empty, _) => Empty,
            (_, &Empty) => Empty,
            (&Exact(ref a), &Exact(ref b)) if a == b => Exact(a.clone()),
            (&Exact(_), &Exact(_)) => Empty,
            (&Range(_, _), &Exact(ref v)) => {
                if self.contains(v) {
                    Exact(v.clone())
                } else {
                    Empty
                }
            }
            (&Exact(ref v), &Range(_, _)) => {
                if other.contains(v) {
                    Exact(v.clone())
                } else {
                    Empty
                }
            }
            (&Range(ref min1, ref max1), &Range(ref min2, ref max2)) => {
                Range(min_opt(min1, min2).clone(), max_opt(max1, max2).clone())
            }
        }
    }
}

fn min_opt<'a, A: Ord>(a: &'a Option<A>, b: &'a Option<A>) -> &'a Option<A> {
    match (a, b) {
        (&Some(ref a_val), &Some(ref b_val)) => if a_val > b_val { a } else { b },
        (&Some(_), &None) => a,
        (&None, &Some(_)) => b,
        (&None, &None) => a,
    }
}

fn max_opt<'a, A: Ord>(a: &'a Option<A>, b: &'a Option<A>) -> &'a Option<A> {
    match (a, b) {
        (&Some(ref a_val), &Some(ref b_val)) => if a_val < b_val { a } else { b },
        (&Some(_), &None) => a,
        (&None, &Some(_)) => b,
        (&None, &None) => a,
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
    assert_eq!(VersionConstraint::range(ver!(1,2,3), ver!(2)).as_string(),
               ">= 1.2.3 < 2".to_string());
}

#[test]
fn constraint_contains() {
    let v = ver!(1, 2, 3);
    assert!(VersionConstraint::all().contains(&v));
    assert!(VersionConstraint::gteq(ver!(1, 2)).contains(&v));
    assert!(VersionConstraint::lt(ver!(3)).contains(&v));
    assert!(VersionConstraint::range(ver!(1, 1, 5), ver!(3, 0, 1, 2)).contains(&v));
    assert!(!VersionConstraint::lt(ver!(1)).contains(&v));
    assert!(!VersionConstraint::gteq(ver!(3)).contains(&v));
    assert!(!VersionConstraint::range(ver!(2), ver!(3)).contains(&v));
}

#[test]
fn constraint_and() {
    assert_eq!(VersionConstraint::all().and(&VersionConstraint::all()), VersionConstraint::all());
    assert_eq!(VersionConstraint::all().and(&Exact(ver!(1, 2, 3))),
               Exact(ver!(1, 2, 3)));
    assert_eq!(Exact(ver!(1, 2, 3)).and(&Exact(ver!(1, 3, 5))),
               Empty);
    assert_eq!(Exact(ver!(1, 2, 3)).and(&VersionConstraint::range(ver!(1), ver!(2))),
               Exact(ver!(1, 2, 3)));
    assert_eq!(Exact(ver!(1, 2, 3)).and(&VersionConstraint::range(ver!(2), ver!(3))),
               Empty);
    assert_eq!(VersionConstraint::gteq(ver!(1, 2, 3)).and(&VersionConstraint::all()),
               VersionConstraint::gteq(ver!(1, 2, 3)));
    assert_eq!(VersionConstraint::gteq(ver!(1, 2, 3)).and(&VersionConstraint::gteq(ver!(1, 5))),
               VersionConstraint::gteq(ver!(1, 5)));
    assert_eq!(VersionConstraint::gteq(ver!(1, 2, 3)).and(&VersionConstraint::gteq(ver!(1, 0))),
               VersionConstraint::gteq(ver!(1, 2, 3)));
    assert_eq!(VersionConstraint::lt(ver!(1, 2, 3)).and(&VersionConstraint::lt(ver!(1, 0))),
               VersionConstraint::lt(ver!(1, 0)));
    assert_eq!(VersionConstraint::lt(ver!(1, 2, 3)).and(&VersionConstraint::lt(ver!(1, 5))),
               VersionConstraint::lt(ver!(1, 2, 3)));
    assert_eq!(VersionConstraint::range(ver!(1, 2, 3), ver!(1, 8))
                 .and(&VersionConstraint::range(ver!(1, 5), ver!(2))),
               VersionConstraint::range(ver!(1, 5), ver!(1, 8)));
}
