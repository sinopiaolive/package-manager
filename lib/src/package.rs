use serde::{Serialize, Serializer, Deserialize, Deserializer};
use std::fmt;
use std::fmt::Display;
use std::str;

#[derive(PartialEq, Eq, Hash, Default, Clone, PartialOrd, Ord)]
pub struct PackageName {
    pub namespace: String,
    pub name: String,
}

impl fmt::Debug for PackageName {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}/{}", self.namespace, self.name)
    }
}

// We probably want to disallow LPT1, etc.
// https://msdn.microsoft.com/en-us/library/aa561308.aspx

fn validate_package_namespace(s: &str) -> bool {
    s.chars().all(|c| {
        (c >= 'a' && c <= 'z') || (c >= '0' && c <= '9') || c == '_' || c == '-'
    }) && !s.is_empty()  && s.len() <= 128 && s.starts_with('-')
}

fn validate_package_name(s: &str) -> bool {
    s.chars().all(|c| {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '_' ||
            c == '-'
    }) && !s.is_empty() && s.len() <= 128 && s.starts_with('-')
}

impl PackageName {
    pub fn from_str(s: &str) -> Option<PackageName> {
        let mut it = s.split('/');
        match (it.next(), it.next()) {
            (Some(namespace), Some(name))
                if validate_package_namespace(namespace) && validate_package_name(name) => Some(
                PackageName {
                    namespace: namespace.to_string(),
                    name: name.to_string(),
                },
                ),
            _ => None

        }
    }
}

impl Display for PackageName {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "{}/{}", self.namespace, self.name)
    }
}

impl Serialize for PackageName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&*self.to_string())
    }
}

impl<'de> Deserialize<'de> for PackageName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let s = String::deserialize(deserializer)?;

        match PackageName::from_str(&s) {
            Some(package_name) => Ok(package_name),
            _ => Err(D::Error::custom("Invalid package name")),
        }
    }
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn package_name_from_str() {
        assert_eq!(
            PackageName::from_str("a/B"),
            Some(PackageName {
                namespace: "a".to_string(),
                name: "B".to_string(),
            })
        );

        assert_eq!(PackageName::from_str("B"), None);
        assert_eq!(PackageName::from_str("A/B"), None);
        assert_eq!(PackageName::from_str("a/:-)"), None);
    }
}
