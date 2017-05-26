use std::sync::Arc;

use registry::Registry;
use manifest::{PackageName, DependencySet, Manifest};
use version::Version;
use constraint::VersionConstraint;

mod path;
mod constraints;
mod conflict;
mod solution;
mod adapter;

use solver::path::Path;
use solver::constraints::{Constraint, ConstraintSet};
use solver::conflict::Failure;
use solver::solution::{PartialSolution, Solution};

fn search(reg: Arc<Registry>, stack: Arc<ConstraintSet>, cheap: bool, solution: PartialSolution) -> Result<PartialSolution, Failure> {
    // FIXME obvs
    Ok(PartialSolution::new())
}

pub fn solve(reg: Arc<Registry>,
             deps: Arc<Manifest>)
             -> Result<Solution, Failure> {
    // FIXME obvs
    Ok(Solution::new())
}



// #[cfg(test)]
// fn sample_registry() -> Rc<Registry> {
//     let reg = gen_registry!(
//         left_pad => (
//             "1.0.0" => deps!(
//                 right_pad => "^1.0.0"
//             ),
//             "2.0.0" => deps!(
//                 right_pad => "^2.0.0"
//             )
//         ),
//         lol_pad => (
//             "1.0.0" => deps!(
//                 right_pad => "^2.0.0"
//             )
//         ),
//         right_pad => (
//             "1.0.0" => deps!(
//                 up_pad => "^1.0.0"
//             ),
//             "1.0.1" => deps!(
//                 up_pad => "^1.0.0"
//             ),
//             "2.0.0" => deps!(
//                 up_pad => "^2.0.0"
//             ),
//             "2.0.1" => deps!(
//                 up_pad => "^2.0.0",
//                 coleft_copad => "^2.0.0"
//             )
//         ),
//         up_pad => (
//             "1.0.0" => deps!(),
//             "2.0.0" => deps!(),
//             "2.1.0" => deps!(
//                 coleft_copad => "^1.0.0"
//             )
//         ),
//         coleft_copad => (
//             "1.0.0" => deps!(),
//             "1.0.1" => deps!(),
//             "1.1.0" => deps!(),
//             "2.0.0" => deps!()
//         ),
//         down_pad => (
//             "1.0.0" => deps!(),
//             "1.2.0" => deps!()
//         )
//     );
//     Rc::new(reg)
// }

// #[cfg(test)]
// fn assert_first_solution(reg: Rc<Registry>, problem: DependencySet, solution: Solution) {
//     let (mut answers, _) = solutions(reg, &problem);
//     assert_eq!(answers.next(), Some(solution));
// }

// #[cfg(test)]
// fn assert_errors(reg: Rc<Registry>, problem: DependencySet, expected_errors: Vec<Conflict>) {
//     let (mut answers, error_cell) = solutions(reg, &problem);
//     assert_eq!(answers.next(), None);
//     let errors = error_cell.borrow().clone();
//     assert_eq!(errors, expected_errors);
// }

// #[test]
// fn find_best_solution_set() {
//     let problem = deps!(
//         down_pad => "^1.0.0",
//         left_pad => "^2.0.0"
//     );

//     assert_first_solution(sample_registry(),
//                           problem,
//                           solution!(
//         left_pad => "2.0.0",
//         down_pad => "1.2.0",
//         right_pad => "2.0.1",
//         up_pad => "2.0.0",
//         coleft_copad => "2.0.0"
//     ));
// }

// #[test]
// fn dependency_conflicts_with_subdependency() {
//     // left_pad has a right_pad constraint conflicting with the top level right_pad constraint
//     let problem = deps!(
//         left_pad => "^1.0.0",
//         right_pad => "^2.0.0"
//     );

//     assert_errors(sample_registry(), problem, vec![
//         Conflict {
//             existing: list![conflict::NamedConstraint {
//                 path: list![(test::pkg("leftpad/left_pad"), test::ver("1.0.0"))],
//                 package: test::pkg("leftpad/right_pad"),
//                 constraint: test::range("^1")
//             }],
//             conflicting: conflict::NamedConstraint {
//                 path: list![],
//                 package: test::pkg("leftpad/right_pad"),
//                 constraint: test::range("^2")
//             }
//         }
//     ]);
// }

// #[test]
// fn conflicting_subdependencies() {
//     // left_pad and lol_pad have conflicting constraints for right_pad,
//     // thus no solution is possible.
//     let problem = deps!(
//         left_pad => "^1.0.0",
//         lol_pad => "^1.0.0"
//     );

//     assert_errors(sample_registry(), problem, vec![
//         Conflict {
//             existing: list![conflict::NamedConstraint {
//                 path: list![(test::pkg("leftpad/left_pad"), test::ver("1.0.0"))],
//                 package: test::pkg("leftpad/right_pad"),
//                 constraint: test::range("^1")
//             }],
//             conflicting: conflict::NamedConstraint {
//                 path: list![(test::pkg("leftpad/lol_pad"), test::ver("1.0.0"))],
//                 package: test::pkg("leftpad/right_pad"),
//                 constraint: test::range("^2")
//             }
//         }
//     ]);
// }
