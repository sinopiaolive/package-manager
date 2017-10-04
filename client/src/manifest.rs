#![allow(dead_code,unused_variables,unused_assignments)]

use std::path::PathBuf;
use pest;
use pm_lib::manifest::{PackageName, DependencySet};
use pm_lib::version::Version;
use manifest_parser::*; // we should expand this later

// The Manifest struct represents a parsed manifest file.

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Release {
    pub name: PackageName,
    pub version: Version,

    pub dependencies: DependencySet,

    pub authors: Vec<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub bugs: Option<String>,
    // We should infer the repository.
    //pub repository: Option<String>,
    pub keywords: Vec<String>,

    pub license: Option<String>,
    pub license_files: Vec<PathBuf>,

    pub readme_contents: String,
    pub files: Vec<PathBuf>,
}

impl Release {
    pub fn from_str<'a>(manifest_source: &'a str)
        -> Result<Self, Error<'a>>
    {
        let manifest_pair = parse_manifest(manifest_source)?;

        Self::from_manifest_pair(manifest_pair)
    }

    pub fn from_manifest_pair<'a>(manifest_pair: Pair<'a>)
        -> Result<Self, Error<'a>>
    {
        let dependencies = get_dependencies(manifest_pair.clone())?;

        let (maybe_dependencies_section_pair, maybe_metadata_section_pair) =
            find_section_pairs(manifest_pair.clone())?;

        let metadata_section_pair = maybe_metadata_section_pair.ok_or_else(||
            pest::Error::CustomErrorPos {
                message: "A `package { ... }` section is required to publish this package".to_string(),
                pos: manifest_pair.clone().into_span().end_pos(),
            }
        )?;

        let object_pair = find_rule(metadata_section_pair, Rule::object);
        check_object_fields(object_pair.clone(), &[
            "name",
            "version",

            "description",
            "keywords",
            "homepage",
            "bugs",
            "repository",

            "license",
            "license_file",
        ])?;

        let name = {
            let (name_string, name_pair)
                = get_string(get_field(object_pair.clone(), "name")?)?;
            PackageName::from_str(&name_string).ok_or_else(||
                pest::Error::CustomErrorSpan {
                    message: "Invalid package name".to_string(),
                    span: name_pair.clone().into_span(),
                }
            )?
        };

        let version = {
            let (version_string, version_pair)
                = get_string(get_field(object_pair.clone(), "version")?)?;
            Version::from_str(&version_string).ok_or_else(||
                pest::Error::CustomErrorSpan {
                    message: "Invalid version number".to_string(),
                    span: version_pair.clone().into_span(),
                }
            )?
        };

        let (description, description_pair)
            = get_optional_string_field(object_pair.clone(), "description")?;
        let (homepage, homepage_pair)
            = get_optional_string_field(object_pair.clone(), "homepage")?;
        let (bugs, bugs_pair)
            = get_optional_string_field(object_pair.clone(), "bugs")?;

        let authors = get_optional_list_field(object_pair.clone(), "authors")?
            .into_iter().map(|(s, s_pair)| s).collect();
        let keywords = get_optional_list_field(object_pair.clone(), "keywords")?
            .into_iter().map(|(s, s_pair)| s).collect();

        let release = Release {
            name: name,
            version: version,

            dependencies: dependencies,

            authors: authors,
            description: description,
            homepage: homepage,
            bugs: bugs,
            keywords: keywords,

            license: None,
            license_files: vec![],

            readme_contents: "README contents go here".to_string(),
            files: vec![],
        };

        Ok(release)
    }
}


pub fn test_reader() {
    // let pairs = ManifestParser::parse_str(Rule::manifest_eof, " \n pm 1.0 // yay \n\n\ndependencies { \njs/left-pad: ^1.2.3 // foo\n}").unwrap_or_else(|e| panic!("{}", e));
    // print_pairs(pairs, 0);

    println!("release: {:?}", Release::from_str(r#"
        pm 1.0
        dependencies {
            js/left-pad: ^1.2.3 // foo
            // bar
            js/right-pad: >=4.5.6 <5.0.0
        }
        package {
            name: "js/foo"
            version: "0.0.0"
        }
    "#).unwrap_or_else(|e| panic!("{}", e)));
}

fn print_pairs<'a>(pairs: ::pest::iterators::Pairs<Rule, ::pest::inputs::StrInput<'a>>, indent: usize) {
    let i = " ".repeat(indent);
    for pair in pairs {
        println!("{}{:?}: {:?}", i, pair.as_rule(), pair.clone().into_span().as_str());
        print_pairs(pair.into_inner(), indent + 2);
    }
}
