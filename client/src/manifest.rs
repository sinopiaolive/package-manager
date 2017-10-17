#![allow(dead_code)]

use std::path::Path;
use pest;
use files::FileCollection;
use git::GitScmProvider;
use pm_lib::manifest::License;
use pm_lib::package::PackageName;
use pm_lib::constraint::VersionConstraint;
use pm_lib::version::Version;
use pm_lib::index::Dependencies;
use manifest_parser::{
    Pair, Rule, Arguments,
    parse_manifest,
    get_field, check_block_fields,
    get_optional_field, get_optional_block_field, get_optional_list_field, get_optional_string_field,
    get_string,
};

use error::Error;

// The Manifest struct represents a parsed manifest file.

#[derive(Debug)]
pub struct Manifest {
    pub name: PackageName,
    pub version: Version,

    pub dependencies: Dependencies,

    pub authors: Vec<String>,
    pub description: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub bugs: Option<String>,
    pub keywords: Vec<String>,

    pub license: License,

    pub readme: Option<(String, String)>,
    pub files: Vec<String>,
}

impl Manifest {
    pub fn from_str(manifest_source: String, root: &Path) -> Result<Self, Error> {
        let manifest_pair = parse_and_check_manifest(manifest_source)?;

        Ok(Self::from_manifest_pair(manifest_pair, root)?)
    }

    pub fn from_manifest_pair(manifest_pair: Pair, root: &Path) -> Result<Self, Error> {
        let dependencies = get_dependencies(manifest_pair.clone())?;

        let package_arguments_pair = get_optional_field(manifest_pair.clone(), "package")
            .ok_or_else(|| {
                // We use get_optional_field and .ok_or_else to produce a
                // clearer error message.
                pest::Error::CustomErrorPos {
                    message: "A `package { ... }` section is required to publish this package"
                        .to_string(),
                    pos: manifest_pair.clone().into_span().end_pos(),
                }
            })?;
        let block_pair = Arguments::from_pair(package_arguments_pair, 0, 0, Some(true))?
            .block.expect("validated block presence");

        check_block_fields(
            block_pair.clone(),
            &[
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
            ],
        )?;

        let name = {
            let name_pair = Arguments::get_single(get_field(block_pair.clone(), "name")?)?;

            let name_string = get_string(name_pair.clone())?;
            PackageName::from_str(&name_string).ok_or_else(|| {
                pest::Error::CustomErrorSpan {
                    message: "Invalid package name".to_string(),
                    span: name_pair.clone().into_span(),
                }
            })?
        };

        let version = {
            let version_pair = Arguments::get_single(get_field(block_pair.clone(), "version")?)?;
            let version_string = get_string(version_pair.clone())?;
            Version::from_str(&version_string).ok_or_else(|| {
                pest::Error::CustomErrorSpan {
                    message: "Invalid version number".to_string(),
                    span: version_pair.clone().into_span(),
                }
            })?
        };

        let description = get_string(Arguments::get_single(get_field(block_pair.clone(), "description")?)?)?;

        let homepage = get_optional_string_field(block_pair.clone(), "homepage")?;
        let repository = get_optional_string_field(block_pair.clone(), "repository")?;
        let bugs = get_optional_string_field(block_pair.clone(), "bugs")?;

        let authors = get_optional_list_field(block_pair.clone(), "authors")?
            .into_iter()
            .map(|item_pair| get_string(item_pair))
            .collect::<Result<_, _>>()?;
        let keywords = get_optional_list_field(block_pair.clone(), "keywords")?
            .into_iter()
            .map(|item_pair| get_string(item_pair))
            .collect::<Result<_, _>>()?;

        let license = get_optional_string_field(block_pair.clone(), "license")?;
        let license_file = get_optional_string_field(block_pair.clone(), "license_file")?;

        let license_field = match (license, license_file) {
            (Some(tag), None) => License::SPDX(tag),
            (None, Some(file)) => License::File(file),
            (Some(tag), Some(file)) => License::SPDXAndFile(tag, file),
            (None, None) => {
                return Err(Error::from(pest::Error::CustomErrorPos {
                    message: "package section needs at least one of license or license_file"
                        .to_string(),
                    pos: block_pair.clone().into_span().start_pos(),
                }))
            }
        };

        let mut file_collection = FileCollection::new(root.to_path_buf())?;
        let git_scm_provider = GitScmProvider::new(root)?;
        // git_scm_provider.check_repo_is_pristine()?;
        for committed_file in git_scm_provider.ls_files()? {
            file_collection.add_file(committed_file)?;
        }
        for glob_pair in get_optional_list_field(block_pair.clone(), "files")?
            .into_iter()
        {
            let glob = get_string(glob_pair.clone())?;
            match file_collection.process_glob(&glob) {
                Err(glob_error) => {
                    // We should try to preserve the structure here rather than
                    // stringifying it.
                    return Err(Error::from(pest::Error::CustomErrorSpan {
                        message: format!("{}", glob_error),
                        span: glob_pair.clone().into_span(),
                    }));
                }
                Ok(()) => { }
            }
        }
        let files = file_collection.get_selected_files();

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

            license: license_field,

            readme: None,

            files: files,
        })
    }
}

pub fn parse_and_check_manifest(manifest_source: String)
    -> Result<Pair, Error>
{
    let manifest_pair = parse_manifest(manifest_source)?;

    check_block_fields(manifest_pair.clone(), &[
        "pm", // TODO do something with this version tag (if present)
        "dependencies",
        "package"
    ])?;

    Ok(manifest_pair)
}

pub fn get_dependencies(manifest_pair: Pair)
    -> Result<Dependencies, Error>
{
    let mut depset = Dependencies::new();
    for (package_name_pair, arguments_pair) in get_optional_block_field(manifest_pair, "dependencies")? {
        let arguments = Arguments::from_pair(arguments_pair, 0, 2, Some(false))?;
        let (package_name, version_constraint) =
            make_dependency(package_name_pair.clone(), arguments.positional_arguments)?;
        if depset.contains_key(&package_name) {
            return Err(Error::from(pest::Error::CustomErrorSpan {
                message: "Duplicate dependency".to_string(),
                span: package_name_pair.into_span(),
            }));
        }
        depset.insert(package_name, version_constraint);
    }
    Ok(depset)
}

pub fn make_dependency(
    package_name_pair: Pair,
    vcc_pairs: Vec<Pair>)
    -> Result<(PackageName, VersionConstraint), Error> {
    let package_name = PackageName::from_str(package_name_pair.as_str()).ok_or_else(||
        pest::Error::CustomErrorSpan {
            message: "Invalid package name".to_string(),
            span: package_name_pair.into_span(),
        }
    )?;

    let version_constraint = match vcc_pairs.len() {
        0 => {
            VersionConstraint::from_str("*")
        }
        1 => {
            let vc_component = vcc_pairs[0].clone();
            VersionConstraint::from_str(vc_component.into_span().as_str())
        }
        2 => {
            let vcc1 = vcc_pairs[0].clone(); // e.g. ">=2.0.0"
            let vcc2 = vcc_pairs[1].clone(); // e.g. "<4.0.0"
            VersionConstraint::from_str(&format!(
                "{} {}", vcc1.into_span().as_str(), vcc2.into_span().as_str()))
        }
        _ => unreachable!()
    }.ok_or_else(||
        Error::from(pest::Error::CustomErrorPos {
            // More error detail would make this much more user-friendly.
            message: "Invalid version constraint".to_string(),
            pos: vcc_pairs[0].clone().into_span().start_pos(),
        })
    )?;

    Ok((package_name, version_constraint))
}



pub fn test_reader() {
    println!(
        "release: {:?}",
        Manifest::from_str(r#"
            dependencies {
                js/left-pad ^1.2.3 // foo
                // bar
                js/right-pad >=4.5.6 <5.0.0
            }
            package {
                name "js/foo"
                version "1.2.3"
                description "The foo package."
                license "MIT"
                files [ "!test/**" ]
            } // commment
        "#.to_string(), &Path::new(".")).unwrap_or_else(|e| panic!("{}", e))
    );
}

fn print_pairs(pairs: ::pest::iterators::Pairs<Rule, ::pest::inputs::StringInput>, indent: usize) {
    let i = " ".repeat(indent);
    for pair in pairs {
        println!(
            "{}{:?}: {:?}",
            i,
            pair.as_rule(),
            pair.clone().into_span().as_str()
        );
        print_pairs(pair.into_inner(), indent + 2);
    }
}
