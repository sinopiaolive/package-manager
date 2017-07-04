use std::convert::From;

use index::{Index, Dependencies};

mod path;
mod constraints;
mod failure;
mod error;
mod solution;
mod adapter;
mod mappable;
#[cfg(test)]
mod test_helpers;

pub use solver::constraints::{Constraint, ConstraintSet};
pub use solver::failure::Failure;
pub use solver::solution::{PartialSolution, Solution, JustifiedVersion};
pub use solver::adapter::RegistryAdapter;
pub use solver::path::Path;
pub use solver::error::Error;
use solver::mappable::Mappable;

fn search(
    ra: &RegistryAdapter,
    mut stack: ConstraintSet,
    cheap: bool,
    solution: &PartialSolution,
) -> Result<PartialSolution, Failure> {
    let mut cheap_failure = None;
    if !cheap {
        loop {
            match search(ra, stack.clone(), true, solution) {
                Ok(sln) => {
                    return Ok(sln);
                }
                Err(failure) => {
                    cheap_failure = Some(failure);
                    let (new_stack, modified) =
                        infer_indirect_dependencies(ra, stack.clone(), solution)?;
                    if !modified {
                        break;
                    } else {
                        stack = new_stack.clone();
                    }
                }
            }
        }
    }
    match stack.pop(&cheap_failure) {
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
                let search_try_version = || {
                    let constraint_set = ra.constraint_set_for(
                        package.clone(),
                        version.clone(),
                        (*path).clone(),
                    )?;
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
            Err(first_failure.expect(
                "unreachable: constraint should never be empty",
            ))
        }
    }
}

pub fn solve(reg: &Index, deps: &Dependencies) -> Result<Solution, Error> {
    let ra = RegistryAdapter::new(reg);
    solve_inner(&ra, &deps).map_err(|failure| Error::from_failure(&reg, &deps, &ra, failure))
}


fn solve_inner(ra: &RegistryAdapter, deps: &Dependencies) -> Result<Solution, Failure> {
    let constraint_set = ra.constraint_set_from(deps)?;
    let partial_solution = search(&ra, constraint_set.clone(), false, &PartialSolution::new())?;
    Ok(Solution::from(partial_solution))
}

fn infer_indirect_dependencies(
    ra: &RegistryAdapter,
    stack: ConstraintSet,
    solution: &PartialSolution,
) -> Result<(ConstraintSet, bool), Failure> {
    let mut modified = false;
    let mut new_stack = stack.clone();

    for (package, constraint) in stack.iter() {
        let mut indirect_constraint_set = None;
        let mut first_failure = None;
        assert!(!constraint.is_empty());
        for (version, path) in constraint.iter() {
            let cset_result =
                ra.constraint_set_for(package.clone(), version.clone(), (*path).clone());
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
    use test_helpers::{pkg, ver, range, sample_registry};
    use solver::test_helpers::{constraint_set, partial_sln};
    use index::{Index, Package, Dependencies};
    use test::Bencher;
    use std::sync::Arc;
    use solver::constraints::Constraint;
    use std::path;
    use solver::error::{Error, Conflict};

    #[bench]
    fn resolve_something_real(b: &mut Bencher) {
        let reg = ::index::read_index(path::Path::new("test/cargo.rmp")).unwrap();

        let problem = deps!{
            tokio_proto => "^0",
            hyper => "^0.11",
            url => "^1"
        };

        b.iter(|| {
            assert_eq!(
                solve(&reg, &problem),
                Ok(solution!{
                    base64 => "0.6.0",
                    byteorder => "1.0.0",
                    bytes => "0.4.4",
                    cfg_if => "0.1.1",
                    futures => "0.1.14",
                    futures_cpupool => "0.1.5",
                    httparse => "1.2.3",
                    hyper => "0.11.0",
                    idna => "0.1.2",
                    iovec => "0.1.0",
                    kernel32_sys => "0.2.2",
                    language_tags => "0.2.2",
                    lazycell => "0.4.0",
                    libc => "0.2.24",
                    log => "0.3.8",
                    matches => "0.1.6",
                    mime => "0.3.2",
                    mio => "0.6.9",
                    miow => "0.2.1",
                    net2 => "0.2.29",
                    num_cpus => "1.6.2",
                    percent_encoding => "1.0.0",
                    rand => "0.3.15",
                    redox_syscall => "0.1.18",
                    safemem => "0.2.0",
                    scoped_tls => "0.1.0",
                    slab => "0.3.0",
                    smallvec => "0.2.1",
                    take => "0.1.0",
                    time => "0.1.37",
                    tokio_core => "0.1.8",
                    tokio_io => "0.1.2",
                    tokio_proto => "0.1.1",
                    tokio_service => "0.1.0",
                    unicase => "2.0.0",
                    unicode_bidi => "0.3.3",
                    unicode_normalization => "0.1.5",
                    url => "1.5.1",
                    winapi => "0.2.8",
                    ws2_32_sys => "0.2.1"
                })
            );
        });
    }

    #[bench]
    fn deep_conflict(b: &mut Bencher) {
        let reg = ::index::read_index(path::Path::new("test/cargo.rmp")).unwrap();

        let problem = deps!{
            rocket => "^0.2.8",
            hyper_rustls => "^0.8"
        };

        b.iter(|| {
            assert_eq!(
                solve(&reg, &problem),
                Err(Error::Conflict(Conflict {
                    package: Arc::new(pkg("hyper")),
                    existing: range("^0.11"),
                    existing_path: conslist![(Arc::new(pkg("hyper_rustls")), Arc::new(ver("0.8.0")))],
                    conflicting: range("^0.10.4"),
                    conflicting_path: conslist![(Arc::new(pkg("rocket")), Arc::new(ver("0.2.8")))]
                }))
            );
        });
    }


    #[test]
    fn find_best_solution_set() {
        let sample_reg = sample_registry();
        let sample_ra = RegistryAdapter::new(&sample_reg);

        let problem =
            deps!(
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
        let problem =
            deps!(
            left_pad => "^1.0.0",
            lol_pad => "^1.0.0"
        );

        assert_eq!(
            solve_inner(&sample_ra, &problem),
            Err(Failure::conflict(
                Arc::new(pkg("right_pad")),
                Constraint::new()
                    .insert(
                        Arc::new(ver("1.0.0")),
                        conslist![(Arc::new(pkg("left_pad")), Arc::new(ver("1.0.0")))],
                    )
                    .insert(
                        Arc::new(ver("1.0.1")),
                        conslist![(Arc::new(pkg("left_pad")), Arc::new(ver("1.0.0")))],
                    ),
                Constraint::new()
                    .insert(
                        Arc::new(ver("2.0.0")),
                        conslist![(Arc::new(pkg("lol_pad")), Arc::new(ver("1.0.0")))],
                    )
                    .insert(
                        Arc::new(ver("2.0.1")),
                        conslist![(Arc::new(pkg("lol_pad")), Arc::new(ver("1.0.0")))],
                    ),
            ))
        );
    }

    #[test]
    fn large_number_of_dependencies_does_not_cause_stackoverflow() {
        // Recursion depth. 1000 doesn't work for Jo here. We may want to
        // eventually move the solver into a thread or rewrite it to be
        // stackless.
        let n = 500;

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
        let problem = deps!{
            P0 => "^1"
        };
        solve(&reg, &problem).unwrap();
    }

    #[test]
    fn infer_indirect_dependencies_test() {
        let reg =
            gen_registry!(
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
        let expected = constraint_set(
            &[
                ("X", &[("1", &[]), ("2", &[]), ("3", &[]), ("4", &[])]),
                (
                    "B",
                    &[
                        ("1", &[("X", "1")]),
                        ("2", &[("X", "1")]),
                        ("3", &[("X", "2")]),
                    ],
                ),
            ],
        );
        let (new_stack, modified) = infer_indirect_dependencies(&ra, stack.clone(), &ps).unwrap();
        assert_eq!(new_stack, expected);
        assert!(modified);
    }
}
