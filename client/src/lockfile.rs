use std::collections::{BTreeMap, BTreeSet};
use std::default::Default;
use std::fmt;
use std::fs::read_to_string;
use std::str::FromStr;
use std::vec::Vec;

use failure;

use pm_lib::dependencies::Dependency;
use pm_lib::package::PackageName;
use pm_lib::version::Version;
use project::ProjectPaths;
use solver::Solution;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lockfile {
    pub meta: LockfileMeta,
    pub locked_dependencies: Vec<LockedDependency>,
}

impl Lockfile {
    pub fn from_file(project_paths: &ProjectPaths) -> Result<Option<Self>, failure::Error> {
        if !project_paths.lockfile.exists() {
            Ok(None)
        } else {
            let lockfile_string = read_to_string(&project_paths.lockfile)?;
            Ok(Some(Lockfile::from_str(&lockfile_string)?))
        }
    }

    pub fn to_solution(&self) -> Result<Solution, failure::Error> {
        let mut solution = BTreeMap::new();
        for dep in &self.locked_dependencies {
            if solution.contains_key(&dep.package_name) {
                bail!("duplicate package in lockfile: {}", &dep.package_name);
            }
            solution.insert(dep.package_name.clone(), dep.version.clone());
        }
        Ok(solution)
    }

    // Return the solution only if the lockfile is consistent with the
    // dependencies provided.
    pub fn to_solution_if_up_to_date(
        &self,
        dependencies: &[Dependency],
    ) -> Result<Option<Solution>, failure::Error> {
        let solution = self.to_solution()?;
        let mut sub_dependencies: BTreeMap<&PackageName, Vec<&Dependency>> = BTreeMap::new();
        for dep in &self.locked_dependencies {
            sub_dependencies.insert(&dep.package_name, dep.dependencies.iter().collect());
        }
        // The `used` set is used to check that all packages in the solution set
        // are indeed necessary to satisfy the dependencies, either directly or
        // indirectly.
        let mut used: BTreeSet<PackageName> = BTreeSet::new();
        // check_dep returns None if the dependency is not satisfied, or
        // Some(more), where `more` is sub-dependencies we still need to check.
        // (Rust does not let us recurse in closures.)
        let check_dep = |dep: &Dependency,
                         solution: &Solution,
                         used: &mut BTreeSet<PackageName>|
         -> Option<&[&Dependency]> {
            match solution.get(&dep.package_name) {
                None => {
                    // Package missing from solution.
                    return None;
                }
                Some(version) => {
                    if !dep.version_constraint.contains(&version) {
                        // Package is in solution but has wrong version.
                        return None;
                    }
                }
            };
            if used.contains(&dep.package_name) {
                // We already checked this package's sub-dependencies, so we
                // don't need to check again. (This prevents infinite
                // recursion.)
                Some(&[])
            } else {
                used.insert(dep.package_name.clone());
                Some(
                    sub_dependencies
                        .get(&dep.package_name)
                        .expect("same keys as solution"),
                )
            }
        };
        let mut dependencies_to_check: Vec<&Dependency> = dependencies.iter().collect();
        while let Some(dep) = dependencies_to_check.pop() {
            match check_dep(&dep, &solution, &mut used) {
                None => return Ok(None),
                Some(more) => {
                    dependencies_to_check.extend_from_slice(more);
                }
            }
        }
        for dep in &self.locked_dependencies {
            if !used.contains(&dep.package_name) {
                return Ok(None);
            }
        }
        Ok(Some(solution))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LockfileMeta {
    install: String,
    update: String,
}

impl Default for LockfileMeta {
    fn default() -> Self {
        LockfileMeta {
            install: "1.0".to_string(),
            update: "1.0".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LockedDependency {
    pub package_name: PackageName,
    pub version: Version,
    pub dependencies: Vec<Dependency>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum LockfileEntry {
    #[serde(rename = "package_manager")]
    Meta(LockfileMeta),
    #[serde(rename = "dependency")]
    Dependency(LockedDependency),
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub enum LockfileEntryGuard<'a> {
    #[serde(rename = "package_manager")]
    Meta {},
    #[serde(rename = "dependency")]
    Dependency { package_name: &'a PackageName },
}

impl LockfileEntry {
    pub fn guard(&self) -> LockfileEntryGuard {
        match self {
            LockfileEntry::Meta(_) => LockfileEntryGuard::Meta {},
            LockfileEntry::Dependency(locked_dependency) => LockfileEntryGuard::Dependency {
                package_name: &locked_dependency.package_name,
            },
        }
    }
}

impl fmt::Display for Lockfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn write_entry(f: &mut fmt::Formatter<'_>, lockfile_entry: &LockfileEntry) -> fmt::Result {
            let entry_string = ::serde_json::to_string(&lockfile_entry).unwrap();
            let guard_string = ::serde_json::to_string(&lockfile_entry.guard()).unwrap();
            writeln!(f, "### {}", guard_string)?;
            writeln!(f, "    {}", entry_string)?;
            writeln!(f, "  # {}", guard_string)?;
            Ok(())
        }

        writeln!(
            f,
            "### THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY."
        )?;

        let meta = LockfileEntry::Meta(self.meta.clone());
        write_entry(f, &meta)?;

        for locked_dependency in &self.locked_dependencies {
            let dependency_entry = LockfileEntry::Dependency(locked_dependency.clone());
            write_entry(f, &dependency_entry)?;
        }
        Ok(())
    }
}

impl FromStr for Lockfile {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut maybe_meta: Option<LockfileMeta> = None;
        let mut dependencies: Vec<LockedDependency> = vec![];
        for line in s.lines() {
            let space: &[_] = &[' ', '\t'];
            let line = line.trim_start_matches(space);
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let entry = ::serde_json::from_str::<LockfileEntry>(line)?;
            match entry {
                LockfileEntry::Meta(m) => {
                    if maybe_meta.is_some() {
                        // TODO allow this to accomodate conflicts; or do we need a separate conflict mode?
                        bail!("duplicate meta entry");
                    }
                    if !dependencies.is_empty() {
                        bail!("meta entry must come before dependencies");
                    }
                    if m.install != "1.0" {
                        bail!("package_manager version {} is required to install dependencies")
                    }
                    maybe_meta = Some(m);
                }
                LockfileEntry::Dependency(d) => {
                    if maybe_meta.is_none() {
                        bail!("missing meta entry");
                    }
                    dependencies.push(d);
                } // TODO: Allow for unrecognized entry types (just check that we
                  // always have a meta entry first). Also allow for merge
                  // conflicts.
            }
        }
        let meta = match maybe_meta {
            // Allow empty lockfiles
            None => LockfileMeta::default(),
            Some(m) => m,
        };
        Ok(Lockfile {
            meta,
            locked_dependencies: dependencies,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pm_lib::test_helpers::*;

    #[test]
    fn roundtrip() {
        let lockfile = Lockfile {
            meta: LockfileMeta::default(),
            locked_dependencies: vec![LockedDependency {
                package_name: pkg("x"),
                version: ver("1.0.0"),
                dependencies: vec![],
            }],
        };
        let serialized = lockfile.to_string();
        assert_eq!(Lockfile::from_str(&serialized).unwrap(), lockfile);
    }
}
