use std::fmt;
use std::sync::Arc;
use std::ops::Add;
use manifest::PackageName;
use version::Version;
use solver::path::Path;
use immutable_map::map::TreeMap as Map;



pub struct JustifiedVersion {
    pub version: Arc<Version>,
    pub path: Path
}

pub type PartialSolution = Map<Arc<PackageName>, JustifiedVersion>;

pub type Solution = Map<Arc<PackageName>, Arc<Version>>;

// impl Add for Solution {
//     type Output = Solution;

//     fn add(self, other: Solution) -> Solution {
//         match (self, other) {
//             (Solution::Solution(a), Solution::Solution(b)) => {
//                 let mut out = a;
//                 for (k, v) in b.into_iter() {
//                     out = out.plus(k.to_owned(), v.to_owned())
//                 }
//                 Solution::Solution(out)
//             }
//         }
//     }
// }

// impl fmt::Debug for Solution {
//     fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
//         write!(f, "Solution( ")?;
//         match self {
//             &Solution::Solution(ref m) => {
//                 for (k, v) in m.into_iter() {
//                     write!(f, "{}: {}", k, v)?;
//                 }
//             }
//         }
//         write!(f, ")")
//     }
// }
