use super::{DependencySet, Registry, VersionSet, VersionConstraint, PackageName};
#[allow(unused_imports)] use linked_hash_map::LinkedHashMap;

// TODO make this a Result
pub fn simple_solver(registry: &Registry, deps: &DependencySet) -> Option<VersionSet> {
    let dep_list = dependency_set_to_list(deps);
    simple_solver_inner(registry, dep_list, VersionSet::new())
}

fn dependency_set_to_list(deps: &DependencySet) -> DependencyList {
    let mut dep_list: DependencyList =
        deps.iter().map(|(pn, c)| (pn.clone(), c.clone())).collect();
    dep_list.reverse();
    dep_list
}

// Like DependencySet, but has have multiple constraints for the same package.
type DependencyList = Vec<(PackageName, VersionConstraint)>;

fn simple_solver_inner(registry: &Registry, mut deps: DependencyList, mut already_activated: VersionSet) -> Option<VersionSet> {
    match deps.pop() {
        None => Some(already_activated),
        Some((ref package_name, ref version_constraint)) => {
            if already_activated.get(&package_name).map_or(false,
                |activated_version| !version_constraint.contains(activated_version)) {
                // We have already activated a version that doesn't satisfy this
                // constraint.
                return None;
            }
            match registry.packages.get(&package_name) {
                None => {
                    // Package name not found in registry.
                    return None;
                }
                Some(ref package) => {
                    let mut matching_versions =
                            version_constraint.all_matching(&package.releases.keys()
                            .map(|v| v.clone()).collect());
                    match matching_versions.pop() {
                        None => {
                            // No versions satisfying the constraint found in
                            // registry.
                            return None;
                        }
                        Some(version) => {
                            let ref indirect_dependencies =
                                package.releases.get(&version).unwrap()
                                .manifest.dependencies;
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

#[test]
fn test_simple_solver() {
    let deps = &mut LinkedHashMap::new();
    deps.insert(
        PackageName{namespace: Some("leftpad".to_string()), name: "a".to_string()},
        VersionConstraint::Range(Some(ver!(1, 0, 0)), Some(ver!(2, 0, 0))));
    deps.insert(
        PackageName{namespace: Some("leftpad".to_string()), name: "b".to_string()},
        VersionConstraint::Range(Some(ver!(1, 0, 0)), Some(ver!(2, 0, 0))));
    assert_eq!(simple_solver(&gen_registry!{
        a => (
            "1.0.0" => (
            )
        )
        // b missing
    }, deps), None);
}
