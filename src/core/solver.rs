use std::ops::Add;
use std::fmt;
use std::collections::HashSet;
use std::iter::IntoIterator;
use std::rc::Rc;
use linked_hash_map::LinkedHashMap;
use hamt_rs::HamtMap;

use registry::Registry;
use manifest::{PackageName, DependencySet};
use version::Version;
use constraint::VersionConstraint;

#[derive(Clone)]
enum SolverError {
    PackageMissing(PackageName),
    VersionMissing(PackageName, Version)
}

#[derive(Clone, PartialEq, Eq)]
pub enum Solution {
    Solution(HamtMap<PackageName, Version>)
}

impl Solution {
    fn plus(self, key: PackageName, value: Version) -> Solution {
        match self {
            Solution::Solution(m) => Solution::Solution(m.plus(key, value))
        }
    }
}

impl Add for Solution {
    type Output = Solution;

    fn add(self, other: Solution) -> Solution {
        match (self, other) {
            (Solution::Solution(a), Solution::Solution(b)) => {
                let mut out = a;
                for (k, v) in b.into_iter() {
                    out = out.plus(k.to_owned(), v.to_owned())
                }
                Solution::Solution(out)
            }
        }
    }
}

impl fmt::Debug for Solution {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Solution( ")?;
        match self {
            &Solution::Solution(ref m) => {
                for (k, v) in m.into_iter() {
                    write!(f, "{}: {}", k, v)?;
                }
            }
        }
        write!(f, ")")
    }
}



fn versions_in_range(reg: &Registry, pkg: &PackageName, constraint: &VersionConstraint) -> Vec<Version> {
    match reg.packages.get(&pkg) {
        None => panic!(SolverError::PackageMissing(pkg.to_owned())),
        Some(p) => {
            let mut vers: Vec<Version> = p.releases.keys().filter(|v| constraint.contains(v)).cloned().collect();
            vers.sort_by(|a, b| b.priority_cmp(a));
            vers
        }
    }
}

fn deps_for(reg: &Registry, pkg: &PackageName, version: &Version) -> DependencySet {
    match reg.packages.get(&pkg) {
        None => panic!(SolverError::PackageMissing(pkg.to_owned())),
        Some(package) => {
            match package.releases.get(&version) {
                None => panic!(SolverError::VersionMissing(pkg.to_owned(), version.to_owned())),
                Some(release) => release.manifest.dependencies.to_owned()
            }
        }
    }
}

fn merge_deps(maybe_a: &Option<DependencySet>, b: &DependencySet) -> Option<DependencySet> {
    match maybe_a {
        &None => None,
        &Some(ref a) => {
            let mut out: DependencySet = LinkedHashMap::new();
            let mut keys = HashSet::new();
            for key in a.keys() {
                keys.insert(key);
            }
            for key in b.keys() {
                keys.insert(key);
            }
            for key in keys {
                match (a.get(key), b.get(key)) {
                    (Some(ac), Some(bc)) => {
                        match ac.and(bc) {
                            VersionConstraint::Empty => return None,
                            c => {
                                out.insert(key.to_owned(), c);
                            }
                        }
                    },
                    (Some(c), None) => {
                        out.insert(key.to_owned(), c.to_owned());
                    },
                    (None, Some(c)) => {
                        out.insert(key.to_owned(), c.to_owned());
                    },
                    _ => unreachable!()
                }
            }
            Some(out)
        }
    }
}

fn constrain_by(deps: &DependencySet, constraints: &DependencySet) -> Option<DependencySet> {
    let mut out: DependencySet = LinkedHashMap::new();
    for key in deps.keys() {
        match (deps.get(key), constraints.get(key)) {
            (Some(deps_c), Some(cons_c)) => {
                match deps_c.and(cons_c) {
                    VersionConstraint::Empty => return None,
                    c => {
                        out.insert(key.to_owned(), c);
                    }
                }
            },
            (Some(c), None) => {
                out.insert(key.to_owned(), c.to_owned());
            },
            _ => unreachable!()
        }
    }
    Some(out)
}

#[derive(Clone)]
struct SolverState {
    registry: Rc<Registry>,
    package: PackageName,
    constraint: VersionConstraint,
    constraint_set: DependencySet
}

fn permute_solutions(mut streams: Vec<SolverState>)
                     -> Box<Iterator<Item = Solution>>
{
    match streams.len() {
        0 => Box::new(::std::iter::empty()),
        1 => solve_with(streams[0].to_owned()),
        _ => {
            let head = streams[0].to_owned();
            let tail = streams.split_off(1);
            Box::new(solve_with(head)
                     .flat_map(move |a| permute_solutions(tail.to_owned())
                               .map(move |b| a.to_owned() + b)))
        }
    }
}

fn solve_with(state: SolverState)
              -> Box<Iterator<Item = Solution>>
{
    let reg = state.registry;
    let pkg = state.package;
    let constraint = state.constraint;
    let const_set = state.constraint_set;
    let versions = versions_in_range(&*reg, &pkg, &constraint);
    Box::new(versions.into_iter().flat_map(move |version| {
        let maybe_deps = constrain_by(&deps_for(&*reg, &pkg, &version), &const_set);
        let maybe_const_set = merge_deps(&maybe_deps, &const_set);
        let it: Box<Iterator<Item = Solution>> = match (maybe_deps, maybe_const_set) {
            (Some(mut deps), Some(new_const_set)) => {
                let mut dep_streams = Vec::new();
                for entry in deps.entries() {
                    let state = SolverState {
                        registry: reg.clone(),
                        package: entry.key().to_owned(),
                        constraint: entry.get().to_owned(),
                        constraint_set: new_const_set.to_owned()
                    };
                    dep_streams.push(state)
                }
                let me = Solution::Solution(HamtMap::new().plus(pkg.to_owned(), version));
                if dep_streams.len() == 0 {
                    Box::new(::std::iter::once(me))
                } else {
                    Box::new(permute_solutions(dep_streams).map(move |s| s + me.to_owned()))
                }
            },
            _ => permute_solutions(vec![])
        };
        it
    }))
}

pub fn solutions(reg: Rc<Registry>, deps: DependencySet) -> Box<Iterator<Item = Solution>> {
    let mut dep_streams = Vec::new();
    let mut mut_deps = deps.to_owned();
    for entry in mut_deps.entries() {
        let state = SolverState {
            registry: reg.clone(),
            package: entry.key().to_owned(),
            constraint: entry.get().to_owned(),
            constraint_set: deps.to_owned()
        };
        dep_streams.push(state);
    }
    Box::new(permute_solutions(dep_streams))
}



#[test]
fn find_solution_set() {
    let reg = gen_registry!(
        left_pad => (
            "1.0.0" => deps!(
                right_pad => "^1.0.0"
            ),
            "2.0.0" => deps!(
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

    let problem = deps!(
        down_pad => "^1.0.0",
        left_pad => "^2.0.0"
    );

    let mut answers = solutions(Rc::new(reg), problem);

    assert_eq!(answers.next(), Some(solution!(
        left_pad => "2.0.0",
        down_pad => "1.2.0",
        right_pad => "2.0.1",
        up_pad => "2.0.0",
        coleft_copad => "2.0.0"
    )));
}
