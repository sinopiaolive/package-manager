#![allow(dead_code)]

use pest;
use pest::Parser;
use pest_parser;

use pm_lib::manifest::{PackageName, DependencySet};
use pm_lib::constraint::VersionConstraint;

// Ensure this file recompiles when the grammar is modified.
const _GRAMMAR: &'static str = include_str!("pest_grammar.pest");

#[derive(Parser)]
#[grammar = "pest_grammar.pest"]
pub struct ManifestParser;

pub type Error<'a> = pest::Error<pest_parser::Rule, pest::inputs::StrInput<'a>>;

type Pair<'a> = pest::iterators::Pair<pest_parser::Rule, pest::inputs::StrInput<'a>>;
type Pairs<'a> = pest::iterators::Pairs<pest_parser::Rule, pest::inputs::StrInput<'a>>;

pub fn get_dependencies<'a>(manifest_source: &'a str)
    -> Result<DependencySet, Error<'a>> {
    let pairs = ManifestParser::parse_str(Rule::manifest_eof, manifest_source)?;
    let mut depset = DependencySet::new();
    for pair in children_of_pairs(pairs, Rule::manifest_eof) {
        for pair in children(pair, Rule::manifest) {
            for pair in children(pair, Rule::manifest_entry) {
                for pair in children(pair, Rule::dependencies) {
                    for pair in children(pair, Rule::dependency) {
                        let (package_name, version_constraint) =
                            get_dependency(pair)?;
                        if depset.contains_key(&package_name) {
                            panic!(); // TODO
                        }
                        depset.insert(package_name, version_constraint);
                    }
                }
            }
        }
    }
    Ok(depset)
}

fn get_dependency<'a>(pair: Pair<'a>) -> Result<(PackageName, VersionConstraint), Error<'a>> {
    let package_name_pair = find(pair.clone(), Rule::package_name);
    let package_name = PackageName::from_str(package_name_pair.into_span().as_str()).expect("TODO");

    let vc_pair = find(pair, Rule::version_constraint);
    let vc_components = children(vc_pair, Rule::version_constraint_component);
    let version_constraint = match vc_components.len() {
        1 => {
            let vc_component = vc_components.into_iter().next().unwrap();
            VersionConstraint::from_str(vc_component.into_span().as_str())
        }
        2 => {
            let mut iter = vc_components.into_iter();
            let vcc1 = iter.next().unwrap(); // e.g. ">=2.0.0"
            let vcc2 = iter.next().unwrap(); // e.g. "<4.0.0"
            VersionConstraint::from_str(&format!(
                "{} {}", vcc1.into_span().as_str(), vcc2.into_span().as_str()))
        }
        _ => unreachable!()
    }.expect("TODO");

    Ok((package_name, version_constraint))
}


fn children<'a>(pair: Pair, rule: pest_parser::Rule) -> Vec<Pair> {
    children_of_pairs(pair.into_inner(), rule)
}

fn children_of_pairs<'a>(pairs: Pairs, rule: pest_parser::Rule) -> Vec<Pair> {
    pairs.filter(|pair| pair.as_rule() == rule).collect()
}

fn find<'a>(pair: Pair<'a>, rule: pest_parser::Rule) -> Pair<'a> {
    pair.into_inner().find(|pair| pair.as_rule() == rule)
        .expect(&format!("No child matching rule {:?}", rule))
}


pub fn test_parser() {
    let pairs = ManifestParser::parse_str(Rule::manifest_eof, " \n\ndependencies { \njs/left-pad: ^1.2.3 // foo\n}").unwrap_or_else(|e| panic!("{}", e));
    print_pairs(pairs, 0);

    println!("dependencies: {:?}", get_dependencies(" \n\ndependencies { \njs/left-pad: ^1.2.3 // foo\n js/right-pad: >=4.5.6 <5.0.0 // foo\n}").unwrap_or_else(|e| panic!("{}", e)));
}

fn print_pairs<'a>(pairs: pest::iterators::Pairs<pest_parser::Rule, pest::inputs::StrInput<'a>>, indent: usize) {
    let i = " ".repeat(indent);
    for pair in pairs {
        println!("{}{:?}: {:?}", i, pair.as_rule(), pair.clone().into_span().as_str());
        print_pairs(pair.into_inner(), indent + 2);
    }
}
