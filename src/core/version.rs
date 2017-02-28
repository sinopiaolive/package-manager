use registry::{Version, VersionIdentifier};
use registry::VersionIdentifier::{Numeric, Alphanumeric};
use nom::{digit, ErrorKind};
#[allow(unused_imports)] use nom::IResult::Done;
use std::str::FromStr;
use std::str;
use regex::Regex;

named!(nat<u64>, map_res!(map_res!(digit, str::from_utf8), to_u64));

named!(base_version<Vec<u64>>, separated_nonempty_list!(char!('.'), nat));


named!(version_id<VersionIdentifier>, map_res!(map_res!(re_bytes_find!(r"^[a-zA-Z0-9-]+"), str::from_utf8), convert_version_identifier));

fn to_u64(s: &str) -> Result<u64, ()> {
    match u64::from_str(s) {
        Ok(n) => {
            if s.chars().count() > 1 && s.chars().next() == Some('0') {
                Err(())
            } else {
                Ok(n)
            }
        }
        Err(_) => Err(())
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
    assert_eq!(version(b"1-2+3"), Done(&b""[..], Version::new(vec![1], vec![Numeric(2)], vec![Numeric(3)])));
    assert_eq!(version(b"1+3"), Done(&b""[..], Version::new(vec![1], vec![], vec![Numeric(3)])));
    assert_eq!(version(b"1-2"), Done(&b""[..], Version::new(vec![1], vec![Numeric(2)], vec![])));
    assert_eq!(version(b"1.2-2.foo+bar.3"), Done(&b""[..], Version::new(vec![1, 2], vec![Numeric(2), Alphanumeric("foo".to_string())], vec![Alphanumeric("bar".to_string()), Numeric(3)])));
    assert_eq!(version(b"1.2+3.4.lol"), Done(&b""[..], Version::new(vec![1, 2], vec![], vec![Numeric(3), Numeric(4), Alphanumeric("lol".to_string())])));
    assert_eq!(version(b"1.3-omg.2"), Done(&b""[..], Version::new(vec![1, 3], vec![Alphanumeric("omg".to_string()), Numeric(2)], vec![])));
    assert!(version(b"1-123456789123456789123456789123456789").is_err());
    assert!(version(b"1-01").is_err());
    assert!(version(b"1-_not_alphanumeric_").is_err());
    assert!(version(b"1+123456789123456789123456789123456789").is_err());
    assert!(version(b"1+01").is_err());
    assert!(version(b"1+_not_alphanumeric_").is_err());
}
