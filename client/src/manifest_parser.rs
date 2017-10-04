#![allow(dead_code,unused_variables,unused_assignments)]

use pest;
use pest::Parser;

use pm_lib::manifest::{PackageName, DependencySet};
use pm_lib::constraint::VersionConstraint;

// Ensure this file recompiles when the grammar is modified.
const _GRAMMAR: &'static str = include_str!("grammar.pest");

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct ManifestParser;

pub type Error<'a> = pest::Error<Rule, pest::inputs::StrInput<'a>>;

pub type Pair<'a> = pest::iterators::Pair<Rule, pest::inputs::StrInput<'a>>;
pub type Pairs<'a> = pest::iterators::Pairs<Rule, pest::inputs::StrInput<'a>>;



// Parse a manifest into just the dependencies. Faster than parse_into_release.
pub fn parse_into_dependencies<'a>(manifest_source: &'a str)
    -> Result<DependencySet, Error<'a>>
{
    let manifest_pair = parse_manifest(manifest_source)?;

    get_dependencies(manifest_pair)
}


pub fn parse_manifest<'a>(manifest_source: &'a str)
    -> Result<Pair<'a>, Error<'a>>
{
    let pairs = ManifestParser::parse_str(Rule::manifest_eof, manifest_source)?;

    let manifest_eof_pair = find_rule_in_pairs(pairs, Rule::manifest_eof);
    let manifest_pair = find_rule(manifest_eof_pair, Rule::manifest);
    Ok(manifest_pair)
}

pub fn get_dependencies<'a>(manifest_pair: Pair<'a>)
    -> Result<DependencySet, Error<'a>>
{
    let (maybe_dependencies_section_pair, _) =
        find_section_pairs(manifest_pair)?;

    let mut depset = DependencySet::new();
    if let Some(dependencies_section_pair) = maybe_dependencies_section_pair {
        for pair in children(dependencies_section_pair, Rule::dependency) {
            let package_name_pair = find_rule(pair.clone(), Rule::package_name);
            let vc_pair = find_rule(pair.clone(), Rule::version_constraint);
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

// Check that there are no unexpected or duplicate fields.
pub fn check_object_fields<'a>(object_pair: Pair<'a>, fields: &'static [&'static str])
    -> Result<(), Error<'a>>
{
    let mut seen = vec![false; fields.len()];
    'pair_loop: for object_entry_pair in children(object_pair, Rule::object_entry) {
        let keyword_pair = find_rule(object_entry_pair.clone(), Rule::keyword);
        let keyword = keyword_pair.as_str();
        for i in 0..fields.len() {
            if keyword == fields[i] {
                if seen[i] {
                    return Err(pest::Error::CustomErrorSpan {
                        message: "Duplicate field".to_string(),
                        span: keyword_pair.clone().into_span(),
                    })
                } else {
                    seen[i] = true;
                    continue 'pair_loop;
                }
            }
        }
        return Err(pest::Error::CustomErrorSpan {
            message: "Unexpected field".to_string(),
            span: keyword_pair.clone().into_span(),
        });
    }
    Ok(())
}

// Return a value pair or an error if the field is missing.
pub fn get_field<'a>(
    object_pair: Pair<'a>,
    field_name: &'static str)
    -> Result<Pair<'a>, Error<'a>>
{
    get_optional_field(object_pair.clone(), field_name)
        .ok_or_else(||
            pest::Error::CustomErrorSpan {
                message: format!("Missing field: {}", field_name).to_string(),
                // We probably want to report this on the line following the
                // opening brace instead.
                span: object_pair.into_span(),
            }
        )
}

pub fn get_optional_field<'a>(
    object_pair: Pair<'a>, field_name: &'static str)
    -> Option<Pair<'a>>
{
    for object_entry_pair in children(object_pair, Rule::object_entry) {
        if find_rule(object_entry_pair.clone(), Rule::keyword).as_str() == field_name {
            return Some(find_rule(object_entry_pair, Rule::value));
        }
    }
    None
}

pub fn get_optional_string_field<'a>(
    object_pair: Pair<'a>, field_name: &'static str)
    -> Result<(Option<String>, Option<Pair<'a>>), Error<'a>>
{
    // option.map would be more concise, but closures break the "?"
    // operator.
    if let Some(field_value_pair) = get_optional_field(object_pair, field_name) {
        let (field_string, field_pair) = get_string(field_value_pair)?;
        Ok((Some(field_string), Some(field_pair)))
    } else {
        Ok((None, None))
    }
}

pub fn get_optional_list_field<'a>(
    object_pair: Pair<'a>, field_name: &'static str)
    -> Result<Vec<(String, Pair<'a>)>, Error<'a>>
{
    let mut string_pair_tuples = vec![];
    if let Some(list_value_pair) = get_optional_field(object_pair.clone(), field_name) {
        let (list, list_pair) = get_list(list_value_pair)?;
        for item_value_pair in list {
            let (string, pair) = get_string(item_value_pair)?;
            string_pair_tuples.push((string, pair));
        }
    }
    Ok(string_pair_tuples)
}


pub fn get_string<'a>(value_pair: Pair<'a>) -> Result<(String, Pair<'a>), Error<'a>> {
    for string_value_pair in children(value_pair.clone(), Rule::string_value) {
        let s = parse_string(string_value_pair.clone())?;
        return Ok((s, string_value_pair));
    }
    Err(pest::Error::CustomErrorSpan {
        message: "Expected string".to_string(),
        span: value_pair.into_span(),
    })
}

pub fn get_list<'a>(value_pair: Pair<'a>) -> Result<(Vec<Pair<'a>>, Pair<'a>), Error<'a>> {
    for list_value_pair in children(value_pair.clone(), Rule::list_value) {
        let v = children(list_value_pair.clone(), Rule::value);
        return Ok((v, list_value_pair));
    }
    Err(pest::Error::CustomErrorSpan {
        message: "Expected list".to_string(),
        span: value_pair.into_span(),
    })
}

pub fn parse_string<'a>(string_value_pair: Pair<'a>) -> Result<String, Error<'a>> {
    let mut s = "".to_string();
    for pair in string_value_pair.into_inner() {
        let c = match pair.as_rule() {
            Rule::literal_character => pair.as_str().chars().next().unwrap(),
            Rule::escaped_quote => '"',
            Rule::escaped_backslash => '\\',
            Rule::escaped_newline => '\n',
            Rule::escaped_tab => '\t',
            Rule::escaped_unicode => {
                let mut hex = "".to_string();
                for hex_pair in children(pair.clone(), Rule::hex) {
                    hex.push_str(hex_pair.as_str());
                }
                let cp = u32::from_str_radix(&hex, 16).expect("parser should not return invalid hex strings");
                match ::std::char::from_u32(cp) {
                    None => {
                        return Err(pest::Error::CustomErrorSpan {
                            message: "Invalid unicode scalar".to_string(),
                            span: pair.into_span(),
                        });
                    },
                    Some(c) => c
                }
            }
            _ => unreachable!("unexpected string character rule; maybe we didn't get a string"),
        };
        s.push(c);
    }
    Ok(s)
}

pub fn find_section_pairs<'a>(manifest_pair: Pair<'a>)
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

pub fn get_dependency<'a>(
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


pub fn children<'a>(pair: Pair, rule: Rule) -> Vec<Pair> {
    children_of_pairs(pair.into_inner(), rule)
}

pub fn children_of_pairs<'a>(pairs: Pairs, rule: Rule) -> Vec<Pair> {
    pairs.filter(|pair| pair.as_rule() == rule).collect()
}

pub fn find_rule<'a>(pair: Pair<'a>, rule: Rule) -> Pair<'a> {
    let pairs = pair.into_inner();
    find_rule_in_pairs(pairs, rule)
}

pub fn find_rule_in_pairs<'a>(mut pairs: Pairs<'a>, rule: Rule) -> Pair<'a> {
    pairs.find(|pair| pair.as_rule() == rule)
        .expect(&format!("No child matching rule {:?}", rule)) // TODO closure me
}
