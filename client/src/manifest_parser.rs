#![allow(dead_code,unused_variables,unused_assignments)]

use pest;
use pest::Parser;

// Ensure this file recompiles when the grammar is modified.
const _GRAMMAR: &'static str = include_str!("grammar.pest");

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct ManifestParser;

pub type Error = pest::Error<Rule, pest::inputs::StringInput>;

pub type Pair = pest::iterators::Pair<Rule, pest::inputs::StringInput>;
pub type Pairs = pest::iterators::Pairs<Rule, pest::inputs::StringInput>;


pub fn parse_manifest(manifest_source: String)
    -> Result<Pair, Error>
{
    let parser_input = ::std::rc::Rc::new(::pest::inputs::StringInput::new(manifest_source));
    let pairs = ManifestParser::parse(Rule::manifest_eof, parser_input)?;

    let manifest_eof_pair = find_rule_in_pairs(pairs, Rule::manifest_eof);
    let manifest_pair = find_rule(manifest_eof_pair, Rule::manifest);
    Ok(manifest_pair)
}

// Check that there are no unexpected or duplicate fields.
pub fn check_block_fields(block_pair: Pair, fields: &'static [&'static str])
    -> Result<(), Error>
{
    let mut seen = vec![false; fields.len()];
    'pair_loop: for (symbol_pair, _arguments_pair) in get_fields(block_pair) {
        let symbol = symbol_pair.as_str();
        for i in 0..fields.len() {
            if symbol == fields[i] {
                if seen[i] {
                    return Err(pest::Error::CustomErrorSpan {
                        message: "Duplicate field".to_string(),
                        span: symbol_pair.clone().into_span(),
                    })
                } else {
                    seen[i] = true;
                    continue 'pair_loop;
                }
            }
        }
        return Err(pest::Error::CustomErrorSpan {
            message: "Unexpected field".to_string(),
            span: symbol_pair.clone().into_span(),
        });
    }
    Ok(())
}

// Return an arguments pair or an error if the field is missing.
pub fn get_field(
    block_pair: Pair,
    field_name: &'static str)
    -> Result<Pair, Error>
{
    get_optional_field(block_pair.clone(), field_name)
        .ok_or_else(||
            pest::Error::CustomErrorSpan {
                message: format!("Missing field: {}", field_name).to_string(),
                // We probably want to report this on the line following the
                // opening brace instead.
                span: block_pair.into_span(),
            }
        )
}

pub fn get_optional_field(
    block_pair: Pair, field_name: &'static str)
    -> Option<Pair>
{
    for (symbol_pair, arguments_pair) in get_fields(block_pair) {
        if symbol_pair.as_str() == field_name {
            return Some(arguments_pair);
        }
    }
    None
}

pub fn get_single_argument(arguments_pair: Pair)
    -> Result<Pair, Error>
{
    let arguments = Arguments::from_pair(arguments_pair, 1, 1)?;
    Ok(arguments.positional_arguments[0].clone())
}

pub struct Arguments {
    pub positional_arguments_pair: Pair,
    pub positional_arguments: Vec<Pair>,
    // pub options_pair: Pair,
    // pub options: ...
    // pub block: Option<Pair>
}

impl Arguments {
    pub fn from_pair(
        arguments_pair: Pair,
        min_positional_arguments: usize, max_positional_arguments: usize,
        ) -> Result<Self, Error>
    {
        let positional_arguments_pair = find_rule(arguments_pair, Rule::positional_arguments);
        let positional_arguments = children(positional_arguments_pair.clone(), Rule::positional_argument);

        if min_positional_arguments == max_positional_arguments {
            if positional_arguments.len() != min_positional_arguments {
                return Err(Error::from(pest::Error::CustomErrorSpan {
                    message: format!("Expected {} argument(s), found {}",
                        min_positional_arguments, positional_arguments.len()).to_string(),
                    span: positional_arguments_pair.into_span()
                }));
            }
        } else {
            if !(positional_arguments.len() >= min_positional_arguments) {
                return Err(Error::from(pest::Error::CustomErrorSpan {
                    message: format!("Expected at least {} argument(s), found {}",
                        min_positional_arguments, positional_arguments.len()).to_string(),
                    span: positional_arguments_pair.into_span()
                }));
            }
            if !(positional_arguments.len() <= max_positional_arguments) {
                return Err(Error::from(pest::Error::CustomErrorSpan {
                    message: format!("Expected at most {} argument(s), found {}",
                        min_positional_arguments, positional_arguments.len()).to_string(),
                    span: positional_arguments_pair.into_span(),
                }));
            }
        }

        Ok(Arguments {
            positional_arguments_pair,
            positional_arguments,
        })
    }
}

pub fn get_optional_list_field(
    block_pair: Pair, field_name: &'static str)
    -> Result<Vec<Pair>, Error>
{
    get_optional_field(block_pair, field_name)
        .map_or(Ok(vec![]), |arguments_pair| {
            let argument_pair = get_single_argument(arguments_pair)?;
            Ok(get_list(argument_pair)?)
        })
}

pub fn get_optional_string_field(block_pair: Pair, field_name: &'static str)
    -> Result<Option<String>, Error>
{
    get_optional_field(block_pair, field_name)
        .map_or(Ok(None), |arguments_pair| {
            let argument_pair = get_single_argument(arguments_pair)?;
            Ok(Some(get_string(argument_pair)?))
        })
}

pub fn get_fields(block_pair: Pair) -> Vec<(Pair, Pair)> {
    let mut v = Vec::new();
    for block_entry in children(block_pair, Rule::block_entry) {
        if let Some(field) = maybe_find_rule(block_entry, Rule::field) {
            v.push((
                find_rule(field.clone(), Rule::symbol),
                find_rule(field.clone(), Rule::arguments)
            ));
        }
    }
    v
}

pub fn get_string(pair: Pair) -> Result<String, Error> {
    for string_pair in children(pair.clone(), Rule::string) {
        let string_contents = parse_string(string_pair.clone())?;
        return Ok(string_contents);
    }
    Err(pest::Error::CustomErrorSpan {
        message: "Expected string".to_string(),
        span: pair.into_span(),
    })
}

pub fn get_list(pair: Pair) -> Result<Vec<Pair>, Error> {
    for list_pair in children(pair.clone(), Rule::list) {
        let list_item_pairs = children(list_pair.clone(), Rule::list_item);
        return Ok(list_item_pairs);
    }
    Err(pest::Error::CustomErrorSpan {
        message: "Expected list".to_string(),
        span: pair.into_span(),
    })
}

pub fn get_version_constraint_component(pair: Pair) -> Result<String, Error> {
    for vcc_pair in children(pair.clone(), Rule::version_constraint_component) {
        let version_constraint_component = vcc_pair.as_str().to_string();
        return Ok(version_constraint_component);
    }
    Err(pest::Error::CustomErrorSpan {
        message: "Expected version range".to_string(),
        span: pair.into_span(),
    })
}

pub fn parse_string(string_pair: Pair) -> Result<String, Error> {
    let mut s = "".to_string();
    for pair in string_pair.into_inner() {
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

pub fn find_section_pairs(manifest_pair: Pair)
    -> Result<(Option<Pair>, Option<Pair>), Error> {
    let mut maybe_dependencies_block_pair: Option<Pair> = None;
    let mut maybe_metadata_block_pair: Option<Pair> = None;
    for pair in children(manifest_pair, Rule::manifest_entry) {
        for pair in pair.into_inner() { // only 1 child
            if pair.as_rule() == Rule::dependencies_section {
                if maybe_dependencies_block_pair.is_some() {
                    return Err(pest::Error::CustomErrorSpan {
                        message: "Duplicate \"dependencies\" section".to_string(),
                        span: pair.into_span(),
                    })
                } else {
                    maybe_dependencies_block_pair = Some(find_rule(pair, Rule::block));
                }
            } else if pair.as_rule() == Rule::metadata_section {
                if maybe_metadata_block_pair.is_some() {
                    return Err(pest::Error::CustomErrorSpan {
                        message: "Duplicate \"package\" section".to_string(),
                        span: pair.into_span(),
                    })
                } else {
                    maybe_metadata_block_pair = Some(find_rule(pair, Rule::block));
                }
            }
        }
    }
    Ok((maybe_dependencies_block_pair, maybe_metadata_block_pair))
}


pub fn children(pair: Pair, rule: Rule) -> Vec<Pair> {
    children_of_pairs(pair.into_inner(), rule)
}

pub fn children_of_pairs(pairs: Pairs, rule: Rule) -> Vec<Pair> {
    pairs.filter(|pair| pair.as_rule() == rule).collect()
}

pub fn find_rule(pair: Pair, rule: Rule) -> Pair {
    let pairs = pair.into_inner();
    find_rule_in_pairs(pairs, rule)
}

pub fn maybe_find_rule(pair: Pair, rule: Rule) -> Option<Pair> {
    let pairs = pair.into_inner();
    maybe_find_rule_in_pairs(pairs, rule)
}

pub fn find_rule_in_pairs(mut pairs: Pairs, rule: Rule) -> Pair {
    maybe_find_rule_in_pairs(pairs, rule)
        .expect(&format!("No child matching rule {:?}", rule)) // TODO closure me

}

pub fn maybe_find_rule_in_pairs(mut pairs: Pairs, rule: Rule) -> Option<Pair> {
    pairs.find(|pair| pair.as_rule() == rule)
}
