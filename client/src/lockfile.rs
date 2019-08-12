use solver::Solution;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Lockfile(LockfileVersion, Solution);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum LockfileVersion {
    #[serde(rename = "0.0")]
    Zero, // the only variant
}

#[cfg(test)]
mod test {
    use super::*;
    use ::pm_lib::test_helpers::*;

    #[test]
    fn serialize() {
        let lockfile = &Lockfile(
            LockfileVersion::Zero,
            vec![(pkg("y"), ver("1.0.0")), (pkg("x"), ver("2.0.0"))]
                .into_iter()
                .collect(),
        );
        let json = r#"[
  "0.0",
  {
    "test/x": "2.0.0",
    "test/y": "1.0.0"
  }
]"#;
        assert_eq!(::serde_json::to_string_pretty(lockfile).unwrap(), json);
    }
}
