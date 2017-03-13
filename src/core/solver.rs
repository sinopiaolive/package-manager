use super::{DependencySet, Registry, VersionSet, VersionConstraint, PackageName};
use error::Error;

// TODO make this a Result
pub fn simple_solver(registry: &Registry, deps: &DependencySet) -> Result<VersionSet, Error> {
    let dep_list = dependency_set_to_list(deps);
    simple_solver_inner(registry, dep_list, VersionSet::new())
}

fn dependency_set_to_list(deps: &DependencySet) -> DependencyList {
    let mut dep_list: DependencyList = deps.iter().map(|(pn, c)| (pn.clone(), c.clone())).collect();
    dep_list.reverse();
    dep_list
}

// Like DependencySet, but has have multiple constraints for the same package.
type DependencyList = Vec<(PackageName, VersionConstraint)>;

fn simple_solver_inner(registry: &Registry,
                       mut deps: DependencyList,
                       mut already_activated: VersionSet)
                       -> Result<VersionSet, Error> {
    match deps.pop() {
        None => Ok(already_activated),
        Some((ref package_name, ref version_constraint)) => {
            if already_activated.get(&package_name).map_or(false, |activated_version| {
                !version_constraint.contains(activated_version)
            }) {
                // We have already activated a version that doesn't satisfy this
                // constraint.
                let selected = already_activated.get(&package_name).unwrap();
                return Err(Error::Custom(format!("{}: selected version {} outside requested \
                                                  bounds {}",
                                                 package_name,
                                                 selected,
                                                 version_constraint)));
            }
            match registry.packages.get(&package_name) {
                None => {
                    // Package name not found in registry.
                    return Err(Error::Custom(format!("package {} not in registry", package_name)));
                }
                Some(ref package) => {
                    let mut matching_versions = version_constraint.all_matching(&package.releases
                        .keys()
                        .map(|v| v.clone())
                        .collect());
                    match matching_versions.pop() {
                        None => {
                            // No versions satisfying the constraint found in
                            // registry.
                            return Err(Error::Custom(format!("package {} has no versions \
                                                              satisfying {}",
                                                             package_name,
                                                             version_constraint)));
                        }
                        Some(version) => {
                            let ref indirect_dependencies = package.releases
                                .get(&version)
                                .unwrap()
                                .manifest
                                .dependencies;
                            deps.extend(dependency_set_to_list(indirect_dependencies));
                            already_activated.insert(package_name.clone(), version);
                        }
                    }
                }
            }
            simple_solver_inner(registry, deps, already_activated)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test::*;
    use linked_hash_map::LinkedHashMap;

    #[test]
    fn test_simple_solver() {
        let deps = &mut LinkedHashMap::new();
        deps.insert(pkg("leftpad/a"), range("^1.0.0"));
        deps.insert(pkg("leftpad/b"), range("^1.0.0"));

        assert!(simple_solver(&gen_registry!{
            a => ( "1.0.0" => deps!() )
            // b missing
        },
                              deps)
            .is_err());
        assert!(simple_solver(&gen_registry!{
            a => ( "1.0.0" => deps!() ),
            b => ( "0.1.0" => deps!() ) // no matching release
        },
                              deps)
            .is_err());
    }
}
