use std::iter::IntoIterator;
use std::rc::Rc;
use std::cell::RefCell;
use hamt_rs::HamtMap as Map;

use registry::Registry;
use manifest::{PackageName, DependencySet};
use version::Version;
use constraint::VersionConstraint;
use list::{List, cons};

#[cfg(test)] use test;

mod path;
mod constraints;
mod conflict;
mod solution;

use self::path::Path;
use self::constraints::Constraints;
use self::conflict::{NamedConstraint, Conflict};
use self::solution::Solution;

pub type Errors = Rc<RefCell<Vec<Conflict>>>;








struct RegistryAdapter {
    // caching
}
impl RegistryAdapter {
    get_depset(...)
}
solve(registry: Registry, deps) -> Result<Solution,





#[derive(Clone)]
pub enum SolverError {
    PackageMissing(PackageName),
    VersionMissing(PackageName, Version),
}

#[derive(Clone)]
struct SolverState {
    registry: Rc<Registry>,
    package: PackageName,
    constraint: VersionConstraint,
    constraint_set: Constraints,
    path: Path,
    remaining: List<NamedConstraint>
}

fn versions_in_range(reg: &Registry,
                     pkg: &PackageName,
                     constraint: &VersionConstraint)
                     -> Vec<Version> {
    match reg.packages.get(&pkg) {
        None => panic!(SolverError::PackageMissing(pkg.to_owned())),
        Some(p) => {
            let mut vers: Vec<Version> =
                p.releases.keys().filter(|v| constraint.contains(v)).cloned().collect();
            vers.sort_by(|a, b| b.priority_cmp(a));
            vers
        }
    }
}

fn deps_for(reg: &Registry, pkg: &PackageName, version: &Version, path: Path) -> Constraints {
    match reg.packages.get(&pkg) {
        None => panic!(SolverError::PackageMissing(pkg.to_owned())),
        Some(package) => {
            match package.releases.get(&version) {
                None => panic!(SolverError::VersionMissing(pkg.to_owned(), version.to_owned())),
                Some(release) => Constraints::from(&release.manifest.dependencies).with_path(path),
            }
        }
    }
}

fn merge_deps(a: &Constraints,
              b: &Constraints,
              errors: Errors)
              -> Option<Constraints> {
    match a.merge(b) {
        Err(conflict) => {
            errors.borrow_mut().push(conflict);
            None
        }
        Ok(c) => Some(c)
    }
}

fn permute_solutions(mut streams: Vec<SolverState>,
                     errors: Errors)
                     -> Box<Iterator<Item = Solution>> {
    match streams.len() {
        0 => Box::new(::std::iter::empty()),
        1 => solve_with(streams[0].to_owned(), errors.clone()),
        _ => {
            let head = streams[0].to_owned();
            let tail = streams.split_off(1);
            Box::new(solve_with(head, errors.clone()).flat_map(move |a| {
                permute_solutions(tail.to_owned(), errors.clone()).map(move |b| a.to_owned() + b)
            }))
        }
    }
}

fn solve_with(state: SolverState, errors: Errors) -> Box<Iterator<Item = Solution>> {
    let reg = state.registry;
    let pkg = state.package;
    let constraint = state.constraint;
    let const_set = state.constraint_set;
    let path = state.path;
    let remaining = state.remaining;
    let versions = versions_in_range(&*reg, &pkg, &constraint);
    if versions.is_empty() {
        // TODO don't panic
        panic!("no versions found oops");
        // errors.borrow_mut().push(Conflict {
        //     path: path.clone(),
        //     package: pkg.clone(),
        //     existing: VersionConstraint::Empty,
        //     conflicting: constraint.clone(),
        // });
        // Box::new(::std::iter::empty::<Solution>())
    } else {
        Box::new(versions.into_iter().flat_map(move |version| {
            let new_path = cons((pkg.clone(), version.clone()), &path);
            let current_deps = deps_for(&*reg, &pkg, &version, new_path.clone());
            // let maybe_deps = constrain_by(&deps_for(&*reg, &pkg, &version),
            //                               &const_set,
            //                               path.clone(),
            //                               errors.clone());
            let maybe_const_set = merge_deps(&current_deps, &const_set, errors.clone());
            let it: Box<Iterator<Item = Solution>> = match maybe_const_set {
                Some(new_const_set) => {
                    let mut dep_streams = Vec::new();
                    for (k, v) in current_deps.as_deps().iter() {
                        let next_state = SolverState {
                            registry: reg.clone(),
                            package: k.to_owned(),
                            constraint: v.to_owned(),
                            constraint_set: new_const_set.clone(),
                            path: new_path.clone(),
                            remaining: remaining.clone(),
                        };
                        dep_streams.push(next_state)
                    }
                    let me = Solution::Solution(Map::new().plus(pkg.to_owned(), version));
                    if dep_streams.len() == 0 {
                        Box::new(::std::iter::once(me))
                    } else {
                        Box::new(permute_solutions(dep_streams, errors.clone())
                            .map(move |s| s + me.to_owned()))
                    }
                }
                _ => Box::new(::std::iter::empty()),
            };
            it
        }))
    }
}

pub fn solutions(reg: Rc<Registry>,
                 deps: &DependencySet)
                 -> (Box<Iterator<Item = Solution>>, Errors) {
    let errors = Rc::new(RefCell::new(vec![]));
    let mut it = deps.iter();
    match it.next() {
        None => (Box::new(::std::iter::once(Solution::empty())), errors.clone()),
        Some((k, v)) => {
            let state = SolverState {
                registry: reg.clone(),
                package: k.clone(),
                constraint: v.clone(),
                constraint_set: Constraints::from(deps),
                path: list![],
                remaining: it.map(|(k, v)| NamedConstraint::new(k, v)).collect()
            };
            (solve_with(state, errors.clone()), errors.clone())
        }
    }
}



#[cfg(test)]
fn sample_registry() -> Rc<Registry> {
    let reg = gen_registry!(
        left_pad => (
            "1.0.0" => deps!(
                right_pad => "^1.0.0"
            ),
            "2.0.0" => deps!(
                right_pad => "^2.0.0"
            )
        ),
        lol_pad => (
            "1.0.0" => deps!(
                right_pad => "^2.0.0"
            )
        ),
        right_pad => (
            "1.0.0" => deps!(
                up_pad => "^1.0.0"
            ),
            "1.0.1" => deps!(
                up_pad => "^1.0.0"
            ),
            "2.0.0" => deps!(
                up_pad => "^2.0.0"
            ),
            "2.0.1" => deps!(
                up_pad => "^2.0.0",
                coleft_copad => "^2.0.0"
            )
        ),
        up_pad => (
            "1.0.0" => deps!(),
            "2.0.0" => deps!(),
            "2.1.0" => deps!(
                coleft_copad => "^1.0.0"
            )
        ),
        coleft_copad => (
            "1.0.0" => deps!(),
            "1.0.1" => deps!(),
            "1.1.0" => deps!(),
            "2.0.0" => deps!()
        ),
        down_pad => (
            "1.0.0" => deps!(),
            "1.2.0" => deps!()
        )
    );
    Rc::new(reg)
}

#[cfg(test)]
fn assert_first_solution(reg: Rc<Registry>, problem: DependencySet, solution: Solution) {
    let (mut answers, _) = solutions(reg, &problem);
    assert_eq!(answers.next(), Some(solution));
}

#[cfg(test)]
fn assert_errors(reg: Rc<Registry>, problem: DependencySet, expected_errors: Vec<Conflict>) {
    let (mut answers, error_cell) = solutions(reg, &problem);
    assert_eq!(answers.next(), None);
    let errors = error_cell.borrow().clone();
    assert_eq!(errors, expected_errors);
}

#[test]
fn find_best_solution_set() {
    let problem = deps!(
        down_pad => "^1.0.0",
        left_pad => "^2.0.0"
    );

    assert_first_solution(sample_registry(),
                          problem,
                          solution!(
        left_pad => "2.0.0",
        down_pad => "1.2.0",
        right_pad => "2.0.1",
        up_pad => "2.0.0",
        coleft_copad => "2.0.0"
    ));
}

#[test]
fn dependency_conflicts_with_subdependency() {
    // left_pad has a right_pad constraint conflicting with the top level right_pad constraint
    let problem = deps!(
        left_pad => "^1.0.0",
        right_pad => "^2.0.0"
    );

    assert_errors(sample_registry(), problem, vec![
        Conflict {
            existing: list![conflict::NamedConstraint {
                path: list![(test::pkg("leftpad/left_pad"), test::ver("1.0.0"))],
                package: test::pkg("leftpad/right_pad"),
                constraint: test::range("^1")
            }],
            conflicting: conflict::NamedConstraint {
                path: list![],
                package: test::pkg("leftpad/right_pad"),
                constraint: test::range("^2")
            }
        }
    ]);
}

#[test]
fn conflicting_subdependencies() {
    // left_pad and lol_pad have conflicting constraints for right_pad,
    // thus no solution is possible.
    let problem = deps!(
        left_pad => "^1.0.0",
        lol_pad => "^1.0.0"
    );

    assert_errors(sample_registry(), problem, vec![
        Conflict {
            existing: list![conflict::NamedConstraint {
                path: list![(test::pkg("leftpad/left_pad"), test::ver("1.0.0"))],
                package: test::pkg("leftpad/right_pad"),
                constraint: test::range("^1")
            }],
            conflicting: conflict::NamedConstraint {
                path: list![(test::pkg("leftpad/lol_pad"), test::ver("1.0.0"))],
                package: test::pkg("leftpad/right_pad"),
                constraint: test::range("^2")
            }
        }
    ]);
}
