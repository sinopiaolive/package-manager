#[cfg(test)] use std::sync::Arc;
use std::convert::From;

use registry::Registry;
use manifest::DependencySet;
#[cfg(test)] use test;

mod path;
mod constraints;
mod failure;
mod solution;
mod adapter;
mod mappable;

use solver::constraints::ConstraintSet;
#[cfg(test)] use solver::constraints::Constraint;
use solver::failure::Failure;
use solver::solution::{PartialSolution, Solution, JustifiedVersion};
use solver::adapter::RegistryAdapter;
use solver::mappable::Mappable;

fn search(ra: &RegistryAdapter,
          mut stack: ConstraintSet,
          cheap: bool,
          solution: &PartialSolution)
          -> Result<PartialSolution, Failure> {
    let mut cheap_failure = None;
    if !cheap {
        loop {
            match search(ra, stack.clone(), true, solution) {
                Ok(sln) => {
                    return Ok(sln);
                }
                Err(failure) => {
                    cheap_failure = Some(failure);
                    let (new_stack, modified) = algo1(ra, stack.clone(), solution)?;
                    if !modified {
                        break;
                    } else {
                        stack = new_stack.clone();
                    }
                }
            }
        }
    }
    // TODO replace .pop with a smarter strategy based on cheap_failure
    match stack.pop() {
        None => Ok(solution.clone()),
        Some((stack_tail, (package, constraint))) => {
            let mut first_failure = None;
            for (version, path) in constraint.iter() {
                let new_solution = solution.insert(package.clone(), JustifiedVersion {
                    version: version.clone(),
                    path: path.clone()
                });
                let search_try_version = || {
                    let constraint_set = ra.constraint_set_for(package.clone(), version.clone(), path.clone())?;
                    let (new_deps, _) = stack_tail.merge(&constraint_set, &new_solution)?;
                    Ok(search(ra.clone(), new_deps, cheap, &new_solution)?)
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

pub fn solve(reg: &Registry, deps: &DependencySet) -> Result<Solution, Failure> {
    let ra = RegistryAdapter::new(reg);
    let constraint_set = ra.constraint_set_from(deps)?;
    match search(&ra, constraint_set.clone(), false, &PartialSolution::new()) {
        Err(failure) => {
            // TODO need to handle failure here
            Err(failure)
        }
        Ok(partial_solution) => Ok(Solution::from(partial_solution)),
    }
}

fn algo1(ra: &RegistryAdapter, stack: ConstraintSet, solution: &PartialSolution) -> Result<(ConstraintSet, bool), Failure> {
    let mut modified = false;
    let mut new_stack = stack.clone();

    for (package, constraint) in stack.iter() {
        let mut new_constraint = constraint.clone();
        let mut indirect_constraint_set = None;
        let mut first_failure = None;
        assert!(!constraint.is_empty());
        for (version, path) in constraint.iter() {
            let cset = ra.constraint_set_for(package.clone(), version.clone(), path.clone())?;
            if let Err(failure) = new_stack.merge(&cset, &solution) {
                // This version is not compatible with our stack and solution,
                // so exclude it.
                if first_failure.is_none() {
                    first_failure = Some(failure);
                }
                // We're not justifying the absence of `version` with some path.
                // Do we need to be able to record this type of thing?
                new_constraint = new_constraint.remove(&version).unwrap().0;
            } else {
                // This version is compatible, so `or` its dependencies into
                // indirect_constraint_set.
                indirect_constraint_set = match indirect_constraint_set {
                    None => Some(cset),
                    Some(icset) => Some(icset) //.or(cset) TODONEXT
                }
            }
        }
        match indirect_constraint_set {
            None => {
                // None of the possible versions were compatible with our stack
                // and solution.
                return Err(first_failure.unwrap());
            }
            Some(icset) => {
                if new_constraint.len() != constraint.len() {
                    // We excluded some versions.
                    assert!(!new_constraint.is_empty());
                    modified = true;
                    new_stack = new_stack.insert(package.clone(), new_constraint);
                }
                let (merged_stack, merged_stack_modified) = new_stack.merge(&icset, &solution)?;
                modified = modified || merged_stack_modified;
                new_stack = merged_stack;
            }
        }
    }
    Ok((new_stack, modified))
}



#[test]
fn find_best_solution_set() {
    let problem = deps!(
        down_pad => "^1.0.0",
        left_pad => "^2.0.0"
    );

    assert_eq!(solve(&test::sample_registry(), &problem), Ok(solution!(
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

    assert_eq!(solve(&test::sample_registry(), &problem), Err(
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
