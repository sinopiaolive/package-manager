use std::fmt;
use std::ops::Add;
use hamt_rs::HamtMap as Map;
use manifest::PackageName;
use version::Version;

#[derive(Clone, PartialEq, Eq)]
pub enum Solution {
    Solution(Map<PackageName, Version>),
}

impl Solution {
    fn plus(self, key: PackageName, value: Version) -> Solution {
        match self {
            // TODO should probably crash hard if trying to overwrite an existing solution?
            Solution::Solution(m) => Solution::Solution(m.plus(key, value)),
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
