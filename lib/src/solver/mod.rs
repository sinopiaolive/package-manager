use crate::index::{Dependencies, Index};

#[macro_use]
pub mod test_helpers;
mod adapter;
mod constraints;
mod error;
mod failure;
mod mappable;
mod path;
mod solution;

pub use solver::adapter::RegistryAdapter;
pub use solver::constraints::{Constraint, ConstraintSet};
pub use solver::error::{Conflict, Error};
pub use solver::failure::Failure;
use solver::mappable::Mappable;
pub use solver::path::Path;
pub use solver::solution::{JustifiedVersion, PartialSolution, Solution};

fn search(
    ra: &RegistryAdapter,
    mut stack: ConstraintSet,
    solution: &PartialSolution,
) -> Result<PartialSolution, Failure> {
    let mut cheap_failure;
    loop {
        match cheap_attempt(ra, &stack, solution) {
            Ok(sln) => {
                return Ok(sln);
            }
            Err(failure) => {
                cheap_failure = failure;
                let (new_stack, modified) = infer_indirect_dependencies(ra, &stack, solution)?;
                if !modified {
                    break;
                } else {
                    stack = new_stack.clone();
                }
            }
        }
    }
    match stack.pop(&Some(cheap_failure)) {
        None => Ok(solution.clone()),
        Some((stack_tail, package, constraint)) => {
            let mut first_failure = None;
            for (version, path) in constraint.iter() {
                let new_solution = solution.insert(
                    package.clone(),
                    JustifiedVersion {
                        version: version.clone(),
                        path: (*path).clone(),
                    },
                );
                let try_version = || {
                    let constraint_set = ra.constraint_set_for(&package, &version, path)?;
                    let (new_deps, _) = stack_tail.and(&constraint_set, &new_solution)?;
                    Ok(search(ra, new_deps, &new_solution)?)
                };
                match try_version() {
                    Err(failure) => {
                        if first_failure.is_none() {
                            first_failure = Some(failure);
                        }
                        continue;
                    }
                    Ok(out) => return Ok(out),
                }
            }
            Err(first_failure.expect("unreachable: constraint should never be empty"))
        }
    }
}

// Try naively picking the highest version of each package without any
// backtracking, to see if we're done. If this doesn't work, return the first
// conflict.
fn cheap_attempt(
    ra: &RegistryAdapter,
    stack_ref: &ConstraintSet,
    solution_ref: &PartialSolution,
) -> Result<PartialSolution, Failure> {
    let mut stack = stack_ref.clone();
    let mut solution = solution_ref.clone();
    loop {
        match stack.pop(&None) {
            None => return Ok(solution),
            Some((stack_tail, package, constraint)) => {
                let (version, path) = constraint
                    .get_min()
                    .expect("unreachable: constraints should never be empty");
                solution = solution.insert(
                    package.clone(),
                    JustifiedVersion {
                        version: version.clone(),
                        path: (*path).clone(),
                    },
                );
                let constraint_set = ra.constraint_set_for(&package, &version, path)?;
                stack = stack_tail.and(&constraint_set, &solution)?.0;
            }
        }
    }
}

pub fn solve(reg: &Index, deps: &Dependencies) -> Result<Solution, Error> {
    let ra = RegistryAdapter::new(reg);
    solve_inner(&ra, &deps).map_err(|failure| Error::from_failure(&reg, &deps, &ra, failure))
}

fn solve_inner(ra: &RegistryAdapter, deps: &Dependencies) -> Result<Solution, Failure> {
    let constraint_set = ra.constraint_set_from(deps)?;
    let partial_solution = search(&ra, constraint_set, &PartialSolution::new())?;
    Ok(Solution::from(partial_solution))
}

fn infer_indirect_dependencies(
    ra: &RegistryAdapter,
    stack: &ConstraintSet,
    solution: &PartialSolution,
) -> Result<(ConstraintSet, bool), Failure> {
    let mut modified = false;
    let mut new_stack = stack.clone();

    for (package, constraint) in stack.iter() {
        let mut indirect_constraint_set = None;
        let mut first_failure = None;
        assert!(!constraint.is_empty());
        for (version, path) in constraint.iter() {
            let cset_result = ra.constraint_set_for(&package, &version, path);
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
    use self::test_helpers::sample_registry;
    use super::*;
    use crate::index::{Dependencies, Index, Package};
    use crate::test_helpers::{pkg, range, ver};
    use solver::constraints::Constraint;
    use solver::test_helpers::{constraint_set, partial_sln, path};
    use std::sync::Arc;

    #[test]
    fn find_best_solution_set() {
        let sample_reg = sample_registry();
        let sample_ra = RegistryAdapter::new(&sample_reg);

        let problem = deps!(
            down_pad => "^1.0.0",
            left_pad => "^2.0.0"
        );

        assert_eq!(
            solve_inner(&sample_ra, &problem),
            Ok(solution!(
                left_pad => "2.0.0",
                down_pad => "1.2.0",
                right_pad => "2.0.1",
                up_pad => "2.0.0",
                coleft_copad => "2.0.0"
            ))
        );
    }

    #[test]
    fn conflicting_subdependencies() {
        let sample_reg = sample_registry();
        let sample_ra = RegistryAdapter::new(&sample_reg);

        // left_pad and lol_pad have conflicting constraints for right_pad,
        // thus no solution is possible.
        let problem = deps!(
            left_pad => "^1.0.0",
            lol_pad => "^1.0.0"
        );

        assert_eq!(
            solve_inner(&sample_ra, &problem),
            Err(Failure::conflict(
                Arc::new(pkg("right_pad")),
                Constraint::new()
                    .insert(Arc::new(ver("1.0.0")), path(&[("left_pad", "1.0.0")]))
                    .insert(Arc::new(ver("1.0.1")), path(&[("left_pad", "1.0.0")])),
                Constraint::new()
                    .insert(Arc::new(ver("2.0.0")), path(&[("lol_pad", "1.0.0")]))
                    .insert(Arc::new(ver("2.0.1")), path(&[("lol_pad", "1.0.0")])),
            ))
        );
    }

    #[test]
    #[ignore]
    fn large_number_of_dependencies_does_not_cause_stack_overflow() {
        let n = 2000;

        let mut reg = Index::new();
        for i in 0..n {
            let mut deps = Dependencies::new();
            if i != n - 1 {
                deps.insert(pkg(&format!("P{}", i + 1)), range("^1"));
            }
            let mut package = Package::new();
            package.insert(ver("1"), deps);
            reg.insert(pkg(&format!("P{}", i)), package);
        }
        let problem = deps! {
            P0 => "^1"
        };
        solve(&reg, &problem).unwrap();
    }

    #[test]
    fn infer_indirect_dependencies_test() {
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
        let expected = constraint_set(&[
            ("X", &[("1", &[]), ("2", &[]), ("3", &[]), ("4", &[])]),
            (
                "B",
                &[
                    ("1", &[("X", "1")]),
                    ("2", &[("X", "1")]),
                    ("3", &[("X", "2")]),
                ],
            ),
        ]);
        let (new_stack, modified) = infer_indirect_dependencies(&ra, &stack, &ps).unwrap();
        assert_eq!(new_stack, expected);
        assert!(modified);
    }
}
