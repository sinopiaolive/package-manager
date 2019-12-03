#![allow(dead_code, unused_variables, unused_assignments)]

use pest;
use pest::Parser;

use crate::manifest_parser_error::{ManifestParserError, PestErrorExt};

// Ensure this file recompiles when the grammar is modified.
const _GRAMMAR: &str = include_str!("grammar.pest");

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct ManifestParser;

pub type Pair = pest::iterators::Pair<Rule, pest::inputs::StringInput>;
pub type Pairs = pest::iterators::Pairs<Rule, pest::inputs::StringInput>;

pub fn parse_manifest(manifest_source: String) -> Result<Pair, ManifestParserError> {
    let parser_input = ::std::rc::Rc::new(::pest::inputs::StringInput::new(manifest_source));
    let mut pairs = ManifestParser::parse(Rule::manifest_eof, parser_input)?;
    let manifest_eof_pair = pairs.next().expect("returns exactly one pair");
    let manifest_pair = find_rule(manifest_eof_pair, Rule::manifest);
    Ok(manifest_pair)
}

/// Check that there are no unexpected or duplicate fields.
pub fn check_block_fields(
    block_pair: &Pair,
    fields: &'static [&'static str],
) -> Result<(), ManifestParserError> {
    let symbol_pairs = get_fields(&block_pair)
        .into_iter()
        .map(|(symbol_pair, _arguments_pair)| symbol_pair)
        .collect::<Vec<_>>();
    check_keys(&symbol_pairs, "field", fields)
}

/// Check that there are no unexpected or duplicate options.
pub fn check_option_names(
    options_pair: Pair,
    names: &'static [&'static str],
) -> Result<(), ManifestParserError> {
    let name_pairs = children(options_pair, Rule::option)
        .into_iter()
        .map(|option_pair| find_rule(option_pair, Rule::option_name))
        .collect::<Vec<_>>();
    check_keys(&name_pairs, "option", names)
}

// Helper function for block and option checking. `element_type` is "field" or
// "option".
fn check_keys(
    name_pairs: &[Pair],
    element_type: &'static str,
    names: &'static [&'static str],
) -> Result<(), ManifestParserError> {
    let mut seen = vec![false; names.len()];
    'pair_loop: for name_pair in name_pairs {
        let name = name_pair.as_str();
        for i in 0..names.len() {
            if name == names[i] {
                if seen[i] {
                    return Err(format_err!("Duplicate {}", element_type).with_pair(&name_pair));
                } else {
                    seen[i] = true;
                    continue 'pair_loop;
                }
            }
        }
        return Err(format_err!("Unexpected {}", element_type).with_pair(&name_pair));
    }
    Ok(())
}

pub fn get_option(options_pair: Pair, name: &'static str) -> Option<Pair> {
    for option_pair in children(options_pair, Rule::option) {
        if find_rule(option_pair.clone(), Rule::option_name).as_str() == name {
            return Some(option_pair);
        }
    }
    None
}

pub fn get_flag_option(
    options_pair: Pair,
    name: &'static str,
) -> Result<bool, ManifestParserError> {
    if let Some(option_pair) = get_option(options_pair, name) {
        if find_optional_rule(option_pair.clone(), Rule::option_value).is_some() {
            Err(format_err!("Unexpected value")
                .with_span(&find_rule(option_pair, Rule::equal).into_span()))
        } else {
            Ok(true)
        }
    } else {
        Ok(false)
    }
}

// Return an arguments pair or an error if the field is missing.
pub fn get_field(block_pair: &Pair, field_name: &'static str) -> Result<Pair, ManifestParserError> {
    get_optional_field(block_pair, field_name)
        .ok_or_else(|| format_err!("Missing field: {}", field_name).with_pair(&block_pair))
}

pub fn get_optional_field(block_pair: &Pair, field_name: &'static str) -> Option<Pair> {
    for (symbol_pair, arguments_pair) in get_fields(&block_pair) {
        if symbol_pair.as_str() == field_name {
            return Some(arguments_pair);
        }
    }
    None
}

pub struct Arguments {
    pub positional_arguments_pair: Pair,
    pub positional_arguments: Vec<Pair>,
    pub options: Pair,
    pub block: Option<Pair>,
}

impl Arguments {
    /// Get a single positional argument (without any other arguments).
    pub fn get_single(arguments_pair: Pair) -> Result<Pair, ManifestParserError> {
        let arguments = Arguments::from_pair(arguments_pair, 1, 1, &[], Some(false))?;
        Ok(arguments.positional_arguments[0].clone())
    }

    /// Get a block (without any other arguments).
    pub fn get_block(arguments_pair: Pair) -> Result<Pair, ManifestParserError> {
        let arguments = Arguments::from_pair(arguments_pair, 0, 0, &[], Some(true))?;
        Ok(arguments.block.expect("validated block presence"))
    }

    /// Create an `Arguments` instance from `arguments_pair`, validating that
    /// the right number of arguments is supplied. If `expect_block` is `None`,
    /// the block is optional.
    pub fn from_pair(
        arguments_pair: Pair,
        min_positional_arguments: usize,
        max_positional_arguments: usize,
        expect_options: &'static [&'static str],
        expect_block: Option<bool>,
    ) -> Result<Arguments, ManifestParserError> {
        let positional_arguments_pair =
            find_rule(arguments_pair.clone(), Rule::positional_arguments);
        let positional_arguments =
            children(positional_arguments_pair.clone(), Rule::positional_argument);

        // TODO: Rework into going argument-by-argument, then reporting:
        // Expected argument
        // Unexpected argument
        // Expected block
        if min_positional_arguments == max_positional_arguments {
            if positional_arguments.len() != min_positional_arguments {
                return Err(format_err!(
                    "Expected {} argument(s), found {}",
                    min_positional_arguments,
                    positional_arguments.len()
                )
                .with_pair(&positional_arguments_pair));
            }
        } else {
            if positional_arguments.len() < min_positional_arguments {
                return Err(format_err!(
                    "Expected at least {} argument(s), found {}",
                    min_positional_arguments,
                    positional_arguments.len()
                )
                .with_pair(&positional_arguments_pair));
            }
            if positional_arguments.len() > max_positional_arguments {
                return Err(format_err!(
                    "Expected at most {} argument(s), found {}",
                    min_positional_arguments,
                    positional_arguments.len()
                )
                .with_pair(&positional_arguments_pair));
            }
        }

        let options_pair = find_rule(arguments_pair.clone(), Rule::options);
        check_option_names(options_pair.clone(), expect_options)?;

        let maybe_block = find_optional_rule(arguments_pair.clone(), Rule::block);
        match expect_block {
            None => {}
            Some(true) => {
                if maybe_block.is_none() {
                    return Err(format_err!("Expected `{{`")
                        .with_pos(&arguments_pair.into_span().end_pos()));
                }
            }
            Some(false) => {
                if let Some(block) = maybe_block {
                    return Err(format_err!("Unexpected block").with_pair(&block));
                }
            }
        }

        Ok(Arguments {
            positional_arguments_pair,
            positional_arguments,
            options: options_pair,
            block: maybe_block,
        })
    }
}

pub fn get_optional_block_field(
    block_pair: &Pair,
    field_name: &'static str,
) -> Result<Vec<(Pair, Pair)>, ManifestParserError> {
    get_optional_field(&block_pair, field_name).map_or(Ok(vec![]), |arguments_pair| {
        let arguments = Arguments::from_pair(arguments_pair, 0, 0, &[], Some(true))?;
        Ok(get_fields(
            &arguments.block.expect("validated block presence"),
        ))
    })
}

pub fn get_optional_list_field(
    block_pair: &Pair,
    field_name: &'static str,
) -> Result<Vec<Pair>, ManifestParserError> {
    get_optional_field(&block_pair, field_name).map_or(Ok(vec![]), |arguments_pair| {
        let argument_pair = Arguments::get_single(arguments_pair)?;
        Ok(get_list(&argument_pair)?)
    })
}

pub fn get_optional_string_field(
    block_pair: &Pair,
    field_name: &'static str,
) -> Result<Option<String>, ManifestParserError> {
    get_optional_field(&block_pair, field_name).map_or(Ok(None), |arguments_pair| {
        let argument_pair = Arguments::get_single(arguments_pair)?;
        Ok(Some(get_string(&argument_pair)?))
    })
}

pub fn get_fields(block_pair: &Pair) -> Vec<(Pair, Pair)> {
    let fields_pair = find_optional_rule(block_pair.clone(), Rule::fields_newline_terminated)
        .unwrap_or_else(|| find_rule(block_pair.clone(), Rule::fields_not_newline_terminated));
    children(fields_pair, Rule::field)
        .into_iter()
        .map(|field| {
            (
                find_rule(field.clone(), Rule::symbol),
                find_rule(field, Rule::arguments),
            )
        })
        .collect()
}

pub fn get_string(pair: &Pair) -> Result<String, ManifestParserError> {
    if let Some(string_pair) = children(pair.clone(), Rule::string).first() {
        let string_contents = parse_string(string_pair.clone())?;
        Ok(string_contents)
    } else {
        Err(format_err!("Expected string").with_pair(&pair))
    }
}

pub fn get_list(pair: &Pair) -> Result<Vec<Pair>, ManifestParserError> {
    if let Some(list_pair) = children(pair.clone(), Rule::list).first() {
        let list_item_pairs = children(list_pair.clone(), Rule::list_item);
        Ok(list_item_pairs)
    } else {
        Err(format_err!("Expected list").with_pair(&pair))
    }
}

pub fn get_version_constraint_component(pair: &Pair) -> Result<String, ManifestParserError> {
    if let Some(vcc_pair) = children(pair.clone(), Rule::version_constraint_component).first() {
        let version_constraint_component = vcc_pair.as_str().to_string();
        Ok(version_constraint_component)
    } else {
        Err(format_err!("Expected version range").with_pair(&pair))
    }
}

pub fn parse_string(string_pair: Pair) -> Result<String, ManifestParserError> {
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
                let cp = u32::from_str_radix(&hex, 16)
                    .expect("parser should not return invalid hex strings");
                match ::std::char::from_u32(cp) {
                    None => {
                        return Err(format_err!("Invalid unicode scalar").with_pair(&pair));
                    }
                    Some(c) => c,
                }
            }
            _ => unreachable!("unexpected string character rule; maybe we didn't get a string"),
        };
        s.push(c);
    }
    Ok(s)
}

pub fn children(pair: Pair, rule: Rule) -> Vec<Pair> {
    children_of_pairs(pair.into_inner(), rule)
}

pub fn children_of_pairs(pairs: Pairs, rule: Rule) -> Vec<Pair> {
    pairs.filter(|pair| pair.as_rule() == rule).collect()
}

pub fn find_rule(pair: Pair, rule: Rule) -> Pair {
    find_optional_rule(pair, rule).unwrap_or_else(||
            // Closure makes error message formatting lazy.
            panic!("No child matches rule {:?}", rule))
}

pub fn find_optional_rule(pair: Pair, rule: Rule) -> Option<Pair> {
    pair.into_inner()
        .find(|child_pair| child_pair.as_rule() == rule)
}
