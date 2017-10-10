#![allow(dead_code)]

use std::path::{Path,PathBuf};
use pest;
use files::FileCollection;
use pm_lib::manifest::{PackageName, DependencySet};
use pm_lib::version::Version;
use manifest_parser::{Pair, Rule, parse_manifest, get_dependencies, find_section_pairs, find_rule, get_field, check_object_fields, get_string, get_optional_list_field, get_optional_string_field};

use error::Error;

// The Manifest struct represents a parsed manifest file.

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Manifest {
    pub name: PackageName,
    pub version: Version,

    pub dependencies: DependencySet,

    pub authors: Vec<String>,
    pub description: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub bugs: Option<String>,
    pub keywords: Vec<String>,

    pub license: Option<String>,
    pub license_file: Option<String>,

    pub readme_file: Option<String>,
    pub readme_file_contents: Option<String>,
    pub files: Vec<PathBuf>,
}

impl Manifest {
    pub fn from_str(manifest_source: String, root: &Path)
        -> Result<Self, Error>
    {
        let manifest_pair = parse_manifest(manifest_source)?;

        Ok(Self::from_manifest_pair(manifest_pair, root)?)
    }

    pub fn from_manifest_pair(manifest_pair: Pair, root: &Path)
        -> Result<Self, Error>
    {
        let dependencies = get_dependencies(manifest_pair.clone())?;

        let (_maybe_dependencies_section_pair, maybe_metadata_section_pair) =
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
            "repository",
            "bugs",

            "license",
            "license_file",

            "files",
        ])?;

        let name = {
            let name_pair = get_field(object_pair.clone(), "name")?;
            let name_string = get_string(name_pair.clone())?;
            PackageName::from_str(&name_string).ok_or_else(||
                pest::Error::CustomErrorSpan {
                    message: "Invalid package name".to_string(),
                    span: name_pair.clone().into_span(),
                }
            )?
        };

        let version = {
            let version_pair = get_field(object_pair.clone(), "version")?;
            let version_string = get_string(version_pair.clone())?;
            Version::from_str(&version_string).ok_or_else(||
                pest::Error::CustomErrorSpan {
                    message: "Invalid version number".to_string(),
                    span: version_pair.clone().into_span(),
                }
            )?
        };

        let description = get_string(get_field(object_pair.clone(), "description")?)?;

        let homepage = get_optional_string_field(object_pair.clone(), "homepage")?;
        let repository = get_optional_string_field(object_pair.clone(), "repository")?;
        let bugs = get_optional_string_field(object_pair.clone(), "bugs")?;

        let authors = get_optional_list_field(object_pair.clone(), "authors")?
            .into_iter().map(|item_pair| get_string(item_pair)).collect::<Result<_, _>>()?;
        let keywords = get_optional_list_field(object_pair.clone(), "keywords")?
            .into_iter().map(|item_pair| get_string(item_pair)).collect::<Result<_, _>>()?;

        let mut file_collection = FileCollection::new(root.to_path_buf())?;
        for glob_pair in get_optional_list_field(object_pair.clone(), "files")?.into_iter() {
            let glob = get_string(glob_pair.clone())?;
            match file_collection.process_glob(&glob) {
                Err(glob_error) => {
                    // We should try to preserve the structure here rather than
                    // stringifying it.
                    Err(pest::Error::CustomErrorSpan {
                        message: format!("{}", glob_error).to_string(),
                        span: glob_pair.clone().into_span(),
                    })?;
                },
                Ok(()) => { },
            }
        }
        let files: Vec<PathBuf> = file_collection.get_selected_files().into_iter()
            .map(|path_string| PathBuf::from(path_string)).collect();

        Ok(Manifest {
            name: name,
            version: version,

            dependencies: dependencies,

            authors: authors,
            description: description,
            homepage: homepage,
            repository: repository,
            bugs: bugs,
            keywords: keywords,

            license: None,
            license_file: None,

            readme_file: None,
            readme_file_contents: None,

            files: files,
        })
    }
}


pub fn test_reader() {
    // let pairs = ManifestParser::parse_str(Rule::manifest_eof, " \n pm 1.0 // yay \n\n\ndependencies { \njs/left-pad: ^1.2.3 // foo\n}").unwrap_or_else(|e| panic!("{}", e));
    // print_pairs(pairs, 0);

    println!("release: {:?}", Manifest::from_str(r#"
        pm 1.0
        dependencies {
            js/left-pad ^1.2.3 // foo
            // bar
            js/right-pad >=4.5.6 <5.0.0
        }
        package {
            name "js/foo"
            version "0.0.0"
            description "The foo package."
            files [ "**/src/**/*.rs" "!**/src/*.rs" ]
        }
    "#.to_string(), &Path::new(".")).unwrap_or_else(|e| panic!("{}", e)));
}

fn print_pairs(pairs: ::pest::iterators::Pairs<Rule, ::pest::inputs::StringInput>, indent: usize) {
    let i = " ".repeat(indent);
    for pair in pairs {
        println!("{}{:?}: {:?}", i, pair.as_rule(), pair.clone().into_span().as_str());
        print_pairs(pair.into_inner(), indent + 2);
    }
}
