use std::sync::Arc;
use im::conslist::ConsList;
use manifest::PackageName;
use version::Version;

/// A dependency chain of packages.
///
/// Note that the dependency graph flows right-to-left, so that new dependencies
/// are cons'ed onto the beginning. That is, the following dependency chain:
///
/// `A 1 -> B 1 -> C 1`
///
/// is stored as the following Path object:
///
/// `[(C, 1), (B, 1), (A, 1)]`
pub type Path = ConsList<(Arc<PackageName>, Arc<Version>)>;
