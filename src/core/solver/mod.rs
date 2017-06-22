use std::convert::From;

use registry::Registry;
use manifest::DependencySet;

mod path;
mod constraints;
mod failure;
mod solution;
mod adapter;
mod mappable;
#[cfg(test)]
mod test;

pub use solver::constraints::{Constraint, ConstraintSet};
pub use solver::failure::Failure;
pub use solver::solution::{PartialSolution, Solution, JustifiedVersion};
pub use solver::adapter::RegistryAdapter;
pub use solver::path::Path;
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
    match stack.pop_most_interesting_package(&cheap_failure) {
        None => Ok(solution.clone()),
        Some((stack_tail, package, constraint)) => {
            let mut first_failure = None;
            for (version, path) in constraint.iter() {
                let new_solution = solution.insert(package.clone(),
                                                   JustifiedVersion {
                                                       version: version.clone(),
                                                       path: path.clone(),
                                                   });
                let search_try_version = || {
                    let constraint_set =
                        ra.constraint_set_for(package.clone(), version.clone(), path.clone())?;
                    let (new_deps, _) = stack_tail.and(&constraint_set, &new_solution)?;
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

fn algo1(ra: &RegistryAdapter,
         stack: ConstraintSet,
         solution: &PartialSolution)
         -> Result<(ConstraintSet, bool), Failure> {
    let mut modified = false;
    let mut new_stack = stack.clone();

    for (package, constraint) in stack.iter() {
        let mut indirect_constraint_set = None;
        let mut first_failure = None;
        assert!(!constraint.is_empty());
        for (version, path) in constraint.iter() {
            let cset_result = ra.constraint_set_for(package.clone(), version.clone(), path.clone());
            match cset_result.and_then(|cset| {
                                           new_stack.and(&cset, &solution)?;
                                           Ok(cset)
                                       }) {
                Err(failure) => {
                    // This version is not compatible with our stack and solution,
                    // so exclude it.
                    if first_failure.is_none() {
                        first_failure = Some(failure);
                    }
                    // It is tempting to remove the version from the constraint
                    // in stack. However, this makes it difficult to always
                    // report good conflicts, at least without additional
                    // bookkeeping. So we keep the version around, and it only
                    // doesn't participate in this algorithm.
                }
                Ok(cset) => {
                    // This version is compatible, so `or` its dependencies into
                    // indirect_constraint_set.
                    indirect_constraint_set = match indirect_constraint_set {
                        None => Some(cset),
                        Some(icset) => Some(icset.or(&cset)),
                    }

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
                let (merged_stack, merged_stack_modified) = new_stack.and(&icset, &solution)?;
                modified = modified || merged_stack_modified;
                new_stack = merged_stack;
            }
        }
    }
    Ok((new_stack, modified))
}



#[cfg(test)]
mod unit_test {
    use super::*;
    use test::{pkg, ver, sample_registry};
    use solver::test::{constraint_set, partial_sln};

    use std::sync::Arc;
    use solver::constraints::Constraint;

    #[test]
    fn find_best_solution_set() {
        let problem = deps!(
            down_pad => "^1.0.0",
            left_pad => "^2.0.0"
        );

        assert_eq!(solve(&sample_registry(), &problem),
                   Ok(solution!(
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

        assert_eq!(solve(&sample_registry(), &problem),
                   Err(Failure::conflict(Arc::new(pkg("right_pad")),
                                         Constraint::new()
                                             .insert(Arc::new(ver("1.0.0")),
                                                     list![(Arc::new(pkg("left_pad")),
                                                            Arc::new(ver("1.0.0")))])
                                             .insert(Arc::new(ver("1.0.1")),
                                                     list![(Arc::new(pkg("left_pad")),
                                                            Arc::new(ver("1.0.0")))]),
                                         Constraint::new()
                                             .insert(Arc::new(ver("2.0.0")),
                                                     list![(Arc::new(pkg("lol_pad")),
                                                            Arc::new(ver("1.0.0")))])
                                             .insert(Arc::new(ver("2.0.1")),
                                                     list![(Arc::new(pkg("lol_pad")),
                                                            Arc::new(ver("1.0.0")))]))));
    }

    #[test]
    fn algo1_test() {
        let reg = gen_registry!(
            X => (
                "1" => deps!(
                    A => "1",
                    B => ">= 1 < 3",
                    S => "1"
                ),
                "2" => deps!(
                    B => ">= 2 < 4",
                    C => "1",
                    S => "1"
                ),
                "3" => deps!(
                    Z => "1" // missing package
                ),
                "4" => deps!(
                    S => "2" // conflicts with existing stack and solution
                )
            ),
            A => (
                "1" => deps!()
            ),
            B => (
                "1" => deps!(),
                "2" => deps!(),
                "3" => deps!()
            ),
            C => (
                "1" => deps!()
            ),
            S => (
                "1" => deps!()
            )
        );
        let ra = RegistryAdapter::new(&reg);
        let stack = constraint_set(&[("X", &[("1", &[]), ("2", &[]), ("3", &[]), ("4", &[])])]);
        let ps = partial_sln(&[("S", ("1", &[]))]);
        let expected = constraint_set(&[("X", &[("1", &[]), ("2", &[]), ("3", &[]), ("4", &[])]),
                                        ("B",
                                         &[("1", &[("X", "1")]),
                                           ("2", &[("X", "1")]),
                                           ("3", &[("X", "2")])])]);
        let (new_stack, modified) = algo1(&ra, stack.clone(), &ps).unwrap();
        assert_eq!(new_stack, expected);
        assert!(modified);
    }
}
