use std::sync::Arc;
use list::List;
use manifest::PackageName;
use version::Version;

pub type Path = Arc<List<(PackageName, Version)>>;
