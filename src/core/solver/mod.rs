use std::sync::Arc;

use registry::Registry;
use manifest::{PackageName, DependencySet};
#[cfg(test)] use test;

mod path;
mod constraints;
mod failure;
mod solution;
mod adapter;

use solver::path::Path;
use solver::constraints::{Constraint, ConstraintSet};
use solver::failure::Failure;
use solver::solution::{PartialSolution, Solution, JustifiedVersion};
use solver::adapter::RegistryAdapter;

fn search(ra: Arc<RegistryAdapter>,
          stack: &ConstraintSet,
          cheap: bool,
          solution: &PartialSolution)
          -> Result<PartialSolution, Failure> {
    // TODO replace .delete_min with a smarter strategy
    match stack.delete_min() {
        None => Ok(solution.clone()),
        Some((stack_tail, (package, constraint))) => {
            let mut first_failure: Option<Failure> = None;
            for (version, path) in constraint {
                let new_solution = solution.insert(package.clone(), JustifiedVersion {
                    version: version.clone(),
                    path: path.clone()
                });
                let search_try_version = || {
                    let constraint_set = ra.constraint_set_for(package.clone(), version.clone(), path.clone())?;
                    let new_deps = merge(&stack_tail, &constraint_set, &new_solution)?;
                    Ok(search(ra.clone(), &new_deps, cheap, &new_solution)?)
                };
                if cheap {
                    // Only try the best version.
                    return search_try_version();
                } else {
                    match search_try_version() {
                        Err(failure) => {
                            if first_failure.is_none() {
                                first_failure = Some(failure);
                            }
                            continue;
                        }
                        Ok(out) => return Ok(out),
                    }
                }
            }
            Err(first_failure.expect("unreachable: constraint should never be empty"))
        }
    }
}

fn merge(existing: &ConstraintSet,
         new: &ConstraintSet,
         solution: &PartialSolution)
         -> Result<ConstraintSet, Failure> {
    let mut out = existing.clone();
    for (package, new_constraint) in new {
        if contained_in(package.clone(), new_constraint, solution)? {
            continue;
        }
        out = match out.get(package) {
            None => out.insert(package.clone(), new_constraint.clone()),
            Some(ref existing_constraint) => {
                out.insert(package.clone(),
                           merge_constraints(package.clone(),
                                             &existing_constraint,
                                             &new_constraint)?)
            }
        }
    }
    Ok(out)
}

fn contained_in(package: Arc<PackageName>,
                constraint: &Constraint,
                solution: &PartialSolution)
                -> Result<bool, Failure> {
    match solution.get(&package.clone()) {
        None => Ok(false),
        Some(&JustifiedVersion {
                 ref version,
                 ref path,
             }) if !constraint.contains_key(&version.clone()) => {
            let exact_constraint = Constraint::new().insert(version.clone(), path.clone());
            Err(Failure::conflict(package.clone(), exact_constraint, constraint.clone()))
        }
        _ => Ok(true),
    }
}

fn merge_constraints(package: Arc<PackageName>,
                     a: &Constraint,
                     b: &Constraint)
                     -> Result<Constraint, Failure> {
    let mut out = Constraint::new();
    for (version, a_path) in a {
        if let Some(ref b_path) = b.get(version) {
            let shortest_path = if a_path.length() <= b_path.length() {
                a_path
            } else {
                b_path
            };
            out = out.insert(version.clone(), shortest_path.clone());
        }
    }

    if out.is_empty() {
        Err(Failure::conflict(package.clone(), a.clone(), b.clone()))
    } else {
        Ok(out)
    }
}

pub fn solve(reg: Arc<Registry>, deps: Arc<DependencySet>) -> Result<Solution, Failure> {
    let ra = Arc::new(RegistryAdapter::new(reg.clone()));
    let constraint_set = dependency_set_to_constraint_set(ra.clone(), deps.clone())?;
    match search(ra.clone(), &constraint_set, false, &PartialSolution::new()) {
        Err(failure) => {
            // TODO need to handle failure here
            Err(failure)
        }
        Ok(partial_solution) => Ok(partial_solution_to_solution(partial_solution)),
    }
}

fn dependency_set_to_constraint_set(ra: Arc<RegistryAdapter>,
                                    deps: Arc<DependencySet>)
                                    -> Result<ConstraintSet, Failure> {
    let mut constraint_set = ConstraintSet::new();
    for (package, version_constraint) in &*deps {
        let package_arc = Arc::new(package.clone());
        let version_constraint_arc = Arc::new(version_constraint.clone());
        let constraint = ra.constraint_for(package_arc.clone(),
                                           version_constraint_arc.clone(),
                                           Path::default())?;
        constraint_set = constraint_set.insert(package_arc.clone(), constraint);
    }

    Ok(constraint_set)
}

// Strip all paths from a PartialSolution to obtain a Solution
fn partial_solution_to_solution(partial_solution: PartialSolution) -> Solution {
    partial_solution
        .iter()
        .map(|(package_name, justified_version)| {
                 (package_name.clone(), justified_version.version.clone())
             })
        .collect()
}


#[cfg(test)]
fn sample_registry() -> Arc<Registry> {
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
    Arc::new(reg)
}

#[test]
fn find_best_solution_set() {
    let problem = deps!(
        down_pad => "^1.0.0",
        left_pad => "^2.0.0"
    );

    assert_eq!(solve(sample_registry(), Arc::new(problem)), Ok(solution!(
        left_pad => "2.0.0",
        down_pad => "1.2.0",
        right_pad => "2.0.1",
        up_pad => "2.0.0",
        coleft_copad => "2.0.0"
    )));
}

#[test]
fn conflicting_subdependencies() {
    // left_pad and lol_pad have conflicting constraints for right_pad,
    // thus no solution is possible.
    let problem = deps!(
        left_pad => "^1.0.0",
        lol_pad => "^1.0.0"
    );

    assert_eq!(solve(sample_registry(), Arc::new(problem)), Err(
        Failure::conflict(
            Arc::new(test::pkg("leftpad/right_pad")),
            Constraint::new()
                .insert(Arc::new(test::ver("1.0.0")), list![(Arc::new(test::pkg("leftpad/left_pad")), Arc::new(test::ver("1.0.0")))])
                .insert(Arc::new(test::ver("1.0.1")), list![(Arc::new(test::pkg("leftpad/left_pad")), Arc::new(test::ver("1.0.0")))]),
            Constraint::new()
                .insert(Arc::new(test::ver("2.0.0")), list![(Arc::new(test::pkg("leftpad/lol_pad")), Arc::new(test::ver("1.0.0")))])
                .insert(Arc::new(test::ver("2.0.1")), list![(Arc::new(test::pkg("leftpad/lol_pad")), Arc::new(test::ver("1.0.0")))]),
        ))
    );
}
