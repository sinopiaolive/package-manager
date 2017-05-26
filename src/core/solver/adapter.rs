use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::vec::Vec;
use constraint::VersionConstraint;
use manifest::PackageName;
use version::Version;
use registry::Registry;

struct RegistryAdapter {
    registry: Arc<Registry>,
    cache: RefCell<HashMap<(Arc<PackageName>, Arc<VersionConstraint>), Option<Vec<Arc<Version>>>>>
}

impl RegistryAdapter {
    pub fn new(registry: Arc<Registry>) -> RegistryAdapter {
        RegistryAdapter {
            registry: registry.clone(),
            cache: RefCell::new(HashMap::new())
        }
    }

    pub fn versions_for(&self, package: Arc<PackageName>, constraint: Arc<VersionConstraint>) -> Option<Vec<Arc<Version>>> {
        let key = (package.clone(), constraint.clone());
        let mut cache = self.cache.borrow_mut();
        if let Some(value) = cache.get(&key) {
            return value.clone()
        }
        let value = match self.registry.packages.get(&package) {
            None => None,
            Some(pkg) => Some(pkg.releases.keys().filter(|v| constraint.contains(v)).map(|v| Arc::new(v.clone())).collect())
        };
        cache.insert(key, value.clone());
        value
    }
}
