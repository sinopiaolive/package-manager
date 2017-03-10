use version::Version;

pub fn ver(s: &str) -> Version {
    Version::from_str(s).unwrap()
}
