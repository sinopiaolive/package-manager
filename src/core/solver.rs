use super::{DependencySet, Registry, VersionSet};
//use linked_hash_map::LinkedHashMap;
//use std::collections::HashMap;

// TODO make this a Result
pub fn solver(registry: &Registry, deps: &DependencySet) -> Option<VersionSet> {
    simple_solver(registry, deps.clone(), VersionSet::new())
}

fn simple_solver(registry: &Registry, mut deps: DependencySet, mut already_activated: VersionSet) -> Option<VersionSet> {
    match deps.pop_front() {
        None => Some(already_activated),
        Some((package_name, version_constraint)) => {
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
                    match version_constraint.all_matching(&package.releases.keys().map(|v| v.clone()).collect()).pop() {
                        None => {
                            // No versions satisfying the constraint found in
                            // registry.
                            return None;
                        }
                        Some(version) => {
                            already_activated.insert(package_name, version);
                        }
                    }
                }
            }
            simple_solver(registry, deps, already_activated)
        }
    }
}
