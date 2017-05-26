use std::sync::Arc;
use list::List;
use manifest::PackageName;
use version::Version;

pub type Path = List<(Arc<PackageName>, Arc<Version>)>;
