use list::List;
use manifest::PackageName;
use version::Version;

pub type Path = List<(PackageName, Version)>;
