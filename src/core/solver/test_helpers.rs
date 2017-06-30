use std::sync::Arc;
use solver::{Path, Constraint, ConstraintSet, JustifiedVersion, PartialSolution};
use test_helpers::{pkg, ver};

pub fn path(l: &[(&str, &str)]) -> Path {
    l.iter()
        .map(|&(p, v)| (Arc::new(pkg(p)), Arc::new(ver(v))))
        .collect()
}

pub fn constraint(l: &[(&str, &[(&str, &str)])]) -> Constraint {
    Constraint(l.iter().map(|&(v, pa)| (ver(v), path(pa))).collect())
}

pub fn constraint_set(l: &[(&str, &[(&str, &[(&str, &str)])])]) -> ConstraintSet {
    ConstraintSet(l.iter().map(|&(p, c)| (pkg(p), constraint(c))).collect())
}

pub fn jver(l: (&str, &[(&str, &str)])) -> JustifiedVersion {
    let (v, pa) = l;
    JustifiedVersion {
        version: Arc::new(ver(v)),
        path: path(pa),
    }
}

pub fn partial_sln(l: &[(&str, (&str, &[(&str, &str)]))]) -> PartialSolution {
    PartialSolution(l.iter().map(|&(p, jv)| (pkg(p), jver(jv))).collect())
}
