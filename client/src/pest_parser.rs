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

    let manifest_eof_pair = find_in_pairs(pairs, Rule::manifest_eof);
    let manifest_pair = find(manifest_eof_pair, Rule::manifest);
    let (maybe_dependencies_section_pair, maybe_metadata_section_pair) =
        find_section_pairs(manifest_pair)?;
    if let Some(dependencies_section_pair) = maybe_dependencies_section_pair {
        for pair in children(dependencies_section_pair, Rule::dependency) {
            let package_name_pair = find(pair.clone(), Rule::package_name);
            let vc_pair = find(pair, Rule::version_constraint);
            let (package_name, version_constraint) =
                get_dependency(package_name_pair.clone(), vc_pair)?;
            if depset.contains_key(&package_name) {
                return Err(pest::Error::CustomErrorSpan {
                    message: "Duplicate dependency".to_string(),
                    span: package_name_pair.into_span(),
                })
            }
            depset.insert(package_name, version_constraint);
        }
    }
    Ok(depset)
}

fn find_section_pairs<'a>(manifest_pair: Pair<'a>)
    -> Result<(Option<Pair<'a>>, Option<Pair<'a>>), Error<'a>> {
    let mut maybe_dependencies_section_pair: Option<Pair<'a>> = None;
    let mut maybe_metadata_section_pair: Option<Pair<'a>> = None;
    for pair in children(manifest_pair, Rule::manifest_entry) {
        for pair in pair.into_inner() { // only 1 child
            if pair.as_rule() == Rule::dependencies_section {
                if maybe_dependencies_section_pair.is_some() {
                    return Err(pest::Error::CustomErrorSpan {
                        message: "Duplicate \"dependencies\" section".to_string(),
                        span: pair.into_span(),
                    })
                } else {
                    maybe_dependencies_section_pair = Some(pair);
                }
            } else if pair.as_rule() == Rule::metadata_section {
                if maybe_metadata_section_pair.is_some() {
                    return Err(pest::Error::CustomErrorSpan {
                        message: "Duplicate \"package\" section".to_string(),
                        span: pair.into_span(),
                    })
                } else {
                    maybe_metadata_section_pair = Some(pair);
                }
            }
        }
    }
    Ok((maybe_dependencies_section_pair, maybe_metadata_section_pair))
}

fn get_dependency<'a>(
    package_name_pair: Pair<'a>,
    vc_pair: Pair<'a>)
    -> Result<(PackageName, VersionConstraint), Error<'a>> {
    let package_name = PackageName::from_str(package_name_pair.as_str()).ok_or_else(||
        pest::Error::CustomErrorSpan {
            message: "Invalid package name".to_string(),
            span: package_name_pair.into_span(),
        }
    )?;

    let vc_components = children(vc_pair.clone(), Rule::version_constraint_component);
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
    }.ok_or_else(||
        pest::Error::CustomErrorSpan {
            // More error detail would make this much more user-friendly.
            message: "Invalid version constraint".to_string(),
            span: vc_pair.into_span(),
        }
    )?;

    Ok((package_name, version_constraint))
}


fn children<'a>(pair: Pair, rule: pest_parser::Rule) -> Vec<Pair> {
    children_of_pairs(pair.into_inner(), rule)
}

fn children_of_pairs<'a>(pairs: Pairs, rule: pest_parser::Rule) -> Vec<Pair> {
    pairs.filter(|pair| pair.as_rule() == rule).collect()
}

fn find<'a>(pair: Pair<'a>, rule: pest_parser::Rule) -> Pair<'a> {
    let pairs = pair.into_inner();
    find_in_pairs(pairs, rule)
}

fn find_in_pairs<'a>(mut pairs: Pairs<'a>, rule: pest_parser::Rule) -> Pair<'a> {
    pairs.find(|pair| pair.as_rule() == rule)
        .expect(&format!("No child matching rule {:?}", rule)) // TODO closure me
}


pub fn test_parser() {
    let pairs = ManifestParser::parse_str(Rule::manifest_eof, " \n pm 1.0 // yay \n\n\ndependencies { \njs/left-pad: ^1.2.3 // foo\n}").unwrap_or_else(|e| panic!("{}", e));
    print_pairs(pairs, 0);

    println!("dependencies: {:?}", get_dependencies("pm 1.0\ndependencies { \njs/left-pad: ^1.2.3 // foo\n js/right-pad: >=4.5.6 <5.0.0 // foo\n}").unwrap_or_else(|e| panic!("{}", e)));
    // println!("dependencies: {:?}", get_dependencies("pm 1.0 \n\ndependencies { \njs/left-pad: ^1.2.3 // foo\n js/right-pad: >=4.x.6 <5.0.0 // foo\n}").unwrap_or_else(|e| panic!("{}", e)));
    // println!("dependencies: {:?}", get_dependencies("pm 1.0 \n\ndependencies { \njs/left-pad: ^1.2.3 // foo\n js/left-pad: >=4.5.6 <5.0.0 // foo\n}").unwrap_or_else(|e| panic!("{}", e)));
    // println!("dependencies: {:?}", get_dependencies("pm 1.0 \n\ndependencies { \n}\ndependencies{\n}").unwrap_or_else(|e| panic!("{}", e)));
    // println!("dependencies: {:?}", get_dependencies("pm 1.0 \npackage { \n}\npackage{\n}").unwrap_or_else(|e| panic!("{}", e)));
}

fn print_pairs<'a>(pairs: pest::iterators::Pairs<pest_parser::Rule, pest::inputs::StrInput<'a>>, indent: usize) {
    let i = " ".repeat(indent);
    for pair in pairs {
        println!("{}{:?}: {:?}", i, pair.as_rule(), pair.clone().into_span().as_str());
        print_pairs(pair.into_inner(), indent + 2);
    }
}
