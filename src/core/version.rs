use registry::{Version, VersionIdentifier};
use registry::VersionIdentifier::{Numeric, Alphanumeric};
use nom::{digit, alphanumeric, IResult, ErrorKind};
use std::str::FromStr;
use std::str;

named!(nat<u64>, map_res!(map_res!(digit, str::from_utf8), u64::from_str));

named!(numeric<VersionIdentifier>, map!(nat, |n| Numeric(n)));

named!(base_version<Vec<u64>>, separated_nonempty_list!(char!('.'), nat));

// TODO are any non-alphanumss allowed?
named!(alpha<VersionIdentifier>, map!(map!(map_res!(alphanumeric, str::from_utf8), str::to_string), |n| Alphanumeric(n)));

named!(version_id<VersionIdentifier>, alt!(numeric | alpha));

named!(ext_version<Vec<VersionIdentifier>>, separated_nonempty_list!(char!('.'), version_id));

// lolsob
named!(empty_vec<Vec<VersionIdentifier>>, value!(Vec::new()));

named!(pre_part<Vec<VersionIdentifier>>, alt_complete!(preceded!(
    char!('-'), return_error!(ErrorKind::Custom(1), ext_version)
) | empty_vec));

named!(build_part<Vec<VersionIdentifier>>, alt_complete!(preceded!(
    char!('+'), return_error!(ErrorKind::Custom(1), ext_version)
) | empty_vec));

named!(version<Version>, do_parse!(
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
fn parse_nat() {
    assert_eq!(nat(b"1"), IResult::Done(&b""[..], 1));
    assert_eq!(nat(b"123"), IResult::Done(&b""[..], 123));
    assert!(nat(b"123456789123456789123456789123456789").is_err());
    assert_eq!(nat(b"123lol"), IResult::Done(&b"lol"[..], 123));
    assert!(nat(b"wat").is_err());
}

#[test]
fn parse_base_version() {
    assert_eq!(base_version(b"1"), IResult::Done(&b""[..], vec![1]));
    assert_eq!(base_version(b"1.2.3"), IResult::Done(&b""[..], vec![1, 2, 3]));
    assert_eq!(base_version(b"1.2.3.4.5"), IResult::Done(&b""[..], vec![1, 2, 3, 4, 5]));
}

#[test]
fn parse_full_version() {
    assert_eq!(version(b"1"), IResult::Done(&b""[..], Version::new(vec![1], vec![], vec![])));
    assert_eq!(version(b"1.2.3"), IResult::Done(&b""[..], Version::new(vec![1, 2, 3], vec![], vec![])));
    assert_eq!(version(b"1-2+3"), IResult::Done(&b""[..], Version::new(vec![1], vec![Numeric(2)], vec![Numeric(3)])));
    assert_eq!(version(b"1+3"), IResult::Done(&b""[..], Version::new(vec![1], vec![], vec![Numeric(3)])));
    assert_eq!(version(b"1-2"), IResult::Done(&b""[..], Version::new(vec![1], vec![Numeric(2)], vec![])));
    assert_eq!(version(b"1.2-2.foo+bar.3"), IResult::Done(&b""[..], Version::new(vec![1, 2], vec![Numeric(2), Alphanumeric("foo".to_string())], vec![Alphanumeric("bar".to_string()), Numeric(3)])));
    assert_eq!(version(b"1.2+3.4.lol"), IResult::Done(&b""[..], Version::new(vec![1, 2], vec![], vec![Numeric(3), Numeric(4), Alphanumeric("lol".to_string())])));
    assert_eq!(version(b"1.3-omg.2"), IResult::Done(&b""[..], Version::new(vec![1, 3], vec![Alphanumeric("omg".to_string()), Numeric(2)], vec![])));
    //assert_eq!(version(b"1-123456789123456789123456789123456789"), IResult::Done(&b""[..], Version::new(vec![1], vec![], vec![])));
}
