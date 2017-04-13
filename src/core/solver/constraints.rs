use std::sync::Arc;
use std::fmt;
use hamt_rs::HamtMap as Map;
use manifest::{PackageName, DependencySet};
use constraint::VersionConstraint;
use list::List;
use solver::path::Path;
use solver::conflict::{NamedConstraint, Conflict};
use linked_hash_map::LinkedHashMap;

#[cfg(test)] use test;

#[derive(Clone, PartialEq, Eq)]
pub struct Constraint {
    sum: VersionConstraint,
    paths: Arc<List<(Path, VersionConstraint)>>,
}

impl Constraint {
    fn new(constraint: &VersionConstraint, path: Path) -> Constraint {
        Constraint {
            sum: constraint.clone(),
            paths: list![(path.clone(), constraint.clone())]
        }
    }

    fn with_path(&self, path: Path) -> Constraint {
        Constraint {
            sum: self.sum.clone(),
            paths: Arc::new(self.paths.iter().map(|(p, c)| (List::append(path.clone(), p.clone()), c.clone())).collect())
        }
    }

    fn add(&self, constraint: &VersionConstraint, path: &Path) -> Constraint {
        Constraint {
            sum: self.sum.and(constraint),
            paths: List::cons((path.clone(), constraint.clone()), self.paths.clone())
        }
    }
}

impl fmt::Debug for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{} [{:?}]", self.sum, self.paths)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Constraints {
    Constraints(Map<PackageName, Constraint>)
}

impl Constraints {
    pub fn with_path(&self, path: Path) -> Constraints {
        let mut out = Map::new();
        for (pkg, constraint) in self.as_map().iter() {
            out = out.plus(pkg.clone(), constraint.with_path(path.clone()))
        }
        Constraints::Constraints(out)
    }

    fn find(&self, pkg: &PackageName) -> Option<&Constraint> {
        match self {
            &Constraints::Constraints(ref m) => m.find(pkg)
        }
    }

    fn plus(&self, pkg: &PackageName, constraint: &Constraint) -> Constraints {
        match self {
            &Constraints::Constraints(ref m) => Constraints::Constraints(m.clone().plus(pkg.clone(), constraint.clone()))
        }
    }

    pub fn as_deps(&self) -> DependencySet {
        let mut out = LinkedHashMap::new();
        for (pkg, constraint) in self.as_map().iter() {
            out.insert(pkg.clone(), constraint.sum.clone());
        }
        out
    }

    pub fn as_paths(&self) -> LinkedHashMap<PackageName, Arc<List<(Path, VersionConstraint)>>> {
        let mut out = LinkedHashMap::new();
        for (pkg, constraint) in self.as_map().iter() {
            out.insert(pkg.clone(), constraint.paths.clone());
        }
        out
    }

    pub fn as_map(&self) -> &Map<PackageName, Constraint> {
        match self {
            &Constraints::Constraints(ref m) => m
        }
    }

    pub fn add(&self,
               path: Path,
               pkg: &PackageName,
               constraint: &VersionConstraint)
               -> Result<Constraints, Conflict> {
        match self.find(pkg) {
            None => {
                let new_cons = Constraint::new(constraint, path);
                Ok(self.plus(pkg, &new_cons))
            },
            Some(old_cons) => {
                let new_cons = old_cons.add(&constraint, &path);
                if new_cons.sum == VersionConstraint::Empty {
                    let conflicting = NamedConstraint {
                        path: path.clone(), package: pkg.clone(), constraint: constraint.clone()
                    };
                    let existing = conflicts_with(self, &conflicting);
                    Err(Conflict { conflicting, existing })
                } else {
                    Ok(self.plus(pkg, &new_cons))
                }
            }
        }
    }

    pub fn merge(&self, other: &Constraints) -> Result<Constraints, Conflict> {
        // Should we collect and return all conflicts from this merge instead of just the first?
        let mut out = self.clone();
        for (pkg, c) in other.as_map().iter() {
            for (ref path, ref constraint) in c.paths.iter() {
                match out.add(path.clone(), pkg, constraint) {
                    Ok(r) => out = r,
                    Err(e) => return Err(e)
                }
            }
        }
        Ok(out)
    }
}

impl fmt::Debug for Constraints {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &Constraints::Constraints(ref m) => {
                write!(f, "Constraints[")?;
                for (pkg, constraint) in m.iter() {
                    write!(f, ":: {} {:?} ", pkg, constraint)?;
                }
                write!(f, "::]")
            }
        }
    }
}

impl Default for Constraints {
    fn default() -> Self {
        Constraints::Constraints(Map::new())
    }
}

impl<'a> From<&'a DependencySet> for Constraints {
    fn from(deps: &'a DependencySet) -> Constraints {
        let mut news = Map::new();
        let empty_path: Path = list![];
        for (pkg, constraint) in deps {
            news = news.plus(pkg.clone(), Constraint::new(&constraint, empty_path.clone()))
        }
        Constraints::Constraints(news)
    }
}

impl Into<DependencySet> for Constraints {
    fn into(self) -> DependencySet {
        self.as_deps()
    }
}

impl Into<LinkedHashMap<PackageName, Arc<List<(Path, VersionConstraint)>>>> for Constraints {
    fn into(self) -> LinkedHashMap<PackageName, Arc<List<(Path, VersionConstraint)>>> {
        self.as_paths()
    }
}

fn conflicts_with(cs: &Constraints, c: &NamedConstraint) -> Arc<List<NamedConstraint>> {
    match cs.find(&c.package) {
        None => List::empty(),
        Some(ref constraint) => {
            Arc::new(constraint.paths.iter()
                     .filter(|&(_, ref c2)| c.constraint.and(c2) == VersionConstraint::Empty)
                     .map(|(p, c2)| NamedConstraint { package: c.package.clone(), path: p, constraint: c2 })
                     .collect())
        }
    }
}

#[test]
fn merge_constraints() {
    let omg_pad = List::singleton((test::pkg("leftpad/omg_pad"), test::ver("1")));
    let c1 = Constraints::from(&deps!(
        left_pad => "^1",
        right_pad => "^1"
    ));
    let c2 = Constraints::from(&deps!(
        up_pad => "^2",
        right_pad => "^1.3"
    )).with_path(omg_pad.clone());
    let c3 = c1.merge(&c2).unwrap();
    assert_eq!(test::unlink(&c3.as_deps()), test::unlink(&deps!(
        left_pad => "^1",
        right_pad => "^1.3",
        up_pad => "^2"
    )));
    assert_eq!(test::unlink(&Into::into(c3.clone())), test::to_mut(&dict!(
        test::pkg("leftpad/left_pad") => list![(List::empty(), test::range("^1"))],
        test::pkg("leftpad/up_pad") => list![(omg_pad.clone(), test::range("^2"))],
        test::pkg("leftpad/right_pad") => list![
            (omg_pad.clone(), test::range("^1.3")),
            (List::empty(), test::range("^1"))
        ]
    )));
}
