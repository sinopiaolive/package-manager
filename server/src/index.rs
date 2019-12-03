use diesel::prelude::*;

use pm_lib::constraint::VersionConstraint;
use pm_lib::index;
use pm_lib::index::Index;
use pm_lib::package::PackageName;
use pm_lib::version::Version;

use crate::package;
use crate::schema::{package_releases, packages, release_dependencies};

use crate::store::Store;

pub fn compute_index(store: &Store) -> Result<Index, ::failure::Error> {
    let mut index = Index::new();
    let db = store.db();
    db.build_transaction()
        .serializable()
        .run::<_, ::failure::Error, _>(|| {
            let package_names = packages::table
                .select((packages::namespace, packages::name))
                .get_results::<(String, String)>(db)?;
            for (namespace, name) in package_names {
                index.insert(PackageName { namespace, name }, index::Package::new());
            }

            let releases = package_releases::table
                .select((
                    package_releases::namespace,
                    package_releases::name,
                    package_releases::version,
                ))
                .get_results::<(String, String, String)>(db)?;
            for (namespace, name, version) in releases {
                let package = index
                    .get_mut(&PackageName { namespace, name })
                    .expect("orphaned release");
                package.insert(
                    Version::from_str(&version).expect("invalid version"),
                    index::Dependencies::new(),
                );
            }

            let dependencies =
                release_dependencies::table.get_results::<package::Dependency>(db)?;
            for dependency in dependencies {
                let package = index
                    .get_mut(&PackageName {
                        namespace: dependency.namespace.clone(),
                        name: dependency.name.clone(),
                    })
                    .expect("orphaned dependency (package key)");
                let release = package
                    .get_mut(&Version::from_str(&dependency.version).expect("invalid version"))
                    .expect("orphaned dependency (version key)");
                let dep_name = PackageName {
                    namespace: dependency.dependency_namespace.clone(),
                    name: dependency.dependency_name.clone(),
                };
                let vc = VersionConstraint::from_str(&dependency.dependency_version_constraint)
                    .expect("invalid version constraint");
                release.insert(dep_name, vc);
            }
            Ok(())
        })?;
    Ok(index)
}
