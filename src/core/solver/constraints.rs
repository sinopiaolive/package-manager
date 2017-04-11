use std::sync::Arc;
use std::fmt;
use hamt_rs::HamtMap as Map;
use manifest::{PackageName, DependencySet};
use constraint::VersionConstraint;
use list::List;
use solver::path::Path;
use solver::conflict::{Constraint, Conflict};

#[cfg(test)] use test;

#[derive(Clone, PartialEq, Eq)]
pub struct Constraints {
    pub sum: Map<PackageName, VersionConstraint>,
    pub paths: Map<PackageName, Arc<List<(Path, VersionConstraint)>>>,
}

impl fmt::Debug for Constraints {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Constraints[")?;
        for (pkg, constraint) in self.sum.iter() {
            write!(f, ".. {} {} ", pkg, constraint)?;
        }
        write!(f, "..] ← [")?;
        for (pkg, paths) in self.paths.iter() {
            for (path, constraint) in paths.iter() {
                write!(f, ":: {} {} from {:?} ", pkg, constraint, path)?;
            }
        }
        write!(f, "::]")
    }
}

impl Default for Constraints {
    fn default() -> Self {
        Constraints {
            sum: Map::new(),
            paths: Map::new(),
        }
    }
}

impl<'a> From<&'a DependencySet> for Constraints {
    fn from(deps: &'a DependencySet) -> Constraints {
        let mut sum = Map::new();
        let mut paths = Map::new();
        let empty_path: Path = List::empty();
        for (pkg, cons) in deps {
            sum = sum.plus(pkg.clone(), cons.clone());
            paths = paths.plus(pkg.clone(),
                               List::cons((empty_path.clone(), cons.clone()), List::empty()))
        }
        Constraints { sum, paths }
    }
}

impl Constraints {
    pub fn with_path(&self, path: Path) -> Constraints {
        let mut out = Map::new();
        for (pkg, constraints) in self.paths.iter() {
            out = out.plus(pkg.clone(), Arc::new(constraints.iter().map(|(p, c)| (List::append(path.clone(), p), c)).collect()))
        }
        Constraints {
            sum: self.sum.clone(),
            paths: out
        }
    }

    pub fn add(&self,
               path: Path,
               pkg: PackageName,
               constraint: VersionConstraint)
               -> Result<Constraints, Conflict> {
        let sum = match self.sum.find(&pkg) {
            None => self.sum.clone().plus(pkg.clone(), constraint.clone()),
            Some(c2) => {
                match c2.and(&constraint) {
                    VersionConstraint::Empty => return {
                        let conflicting = Constraint {
                            path: path.clone(), package: pkg.clone(), constraint: constraint.clone()
                        };
                        let existing = conflicts_with(self, &conflicting);
                        Err(Conflict {
                            conflicting,
                            existing
                        })
                    },
                    new_cons => self.sum.clone().plus(pkg.clone(), new_cons.clone()),
                }
            }
        };
        let empty_path: Arc<List<(Path, VersionConstraint)>> = List::empty();
        let old_paths = match self.paths.find(&pkg) {
            None => empty_path.clone(),
            Some(p) => p.clone()
        };
        let paths =
            self.paths.clone().plus(pkg.clone(),
                                    List::cons((path.clone(), constraint.clone()), old_paths.clone()));
        Ok(Constraints {
            sum: sum,
            paths: paths,
        })
    }

    pub fn merge(&self, other: &Constraints) -> Result<Constraints, Conflict> {
        // Should we collect and return all conflicts from this merge instead of just the first?
        let mut out = self.clone();
        for (pkg, paths) in other.paths.iter() {
            for (ref path, ref constraint) in paths.iter() {
                match out.add(path.clone(), pkg.clone(), constraint.clone()) {
                    Ok(r) => out = r,
                    Err(e) => return Err(e)
                }
            }
        }
        Ok(out)
    }
}

fn conflicts_with(cs: &Constraints, c: &Constraint) -> Arc<List<Constraint>> {
    match cs.paths.find(&c.package) {
        None => List::empty(),
        Some(paths) => {
            Arc::new(paths.iter()
                     .filter(|&(_, ref c2)| c.constraint.and(c2) == VersionConstraint::Empty)
                     .map(|(p, c2)| Constraint { package: c.package.clone(), path: p, constraint: c2 })
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
    assert_eq!(test::to_mut(&c3.sum), test::unlink(&deps!(
        left_pad => "^1",
        right_pad => "^1.3",
        up_pad => "^2"
    )));
    assert_eq!(test::to_mut(&c3.paths), test::to_mut(&dict!(
        test::pkg("leftpad/left_pad") => list![(List::empty(), test::range("^1"))],
        test::pkg("leftpad/up_pad") => list![(omg_pad.clone(), test::range("^2"))],
        test::pkg("leftpad/right_pad") => list![
            (omg_pad.clone(), test::range("^1.3")),
            (List::empty(), test::range("^1"))
        ]
    )));
}
