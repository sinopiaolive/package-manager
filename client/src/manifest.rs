#![allow(dead_code)]

use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::files::FilesSectionInterpreter;
use crate::manifest_parser::{
    check_block_fields, get_field, get_fields, get_optional_block_field, get_optional_field,
    get_optional_list_field, get_optional_string_field, get_string, parse_manifest, Arguments,
    Pair, Rule,
};
use crate::manifest_parser_error::{PestErrorExt, PestResultExt};
use pm_lib::constraint::VersionConstraint;
use pm_lib::dependencies::Dependency;
use pm_lib::package::PackageName;
use pm_lib::version::Version;
use crate::project::ProjectPaths;

// The Manifest struct represents a parsed manifest file. This struct should
// probably go away in favor of constructing PublicationRequest objects directly
// while parsing.

#[derive(Debug)]
pub struct Manifest {
    pub name: PackageName,
    pub version: Version,

    pub dependencies: Vec<Dependency>,

    pub authors: Vec<String>,
    pub description: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub bugs: Option<String>,
    pub keywords: Vec<String>,

    pub license: Option<String>,
    pub license_file: Option<String>,

    pub readme: Option<(String, String)>,
    pub files: Vec<String>,
}

impl Manifest {
    pub fn from_file(project_paths: &ProjectPaths) -> Result<Self, failure::Error> {
        let data = File::open(&project_paths.manifest).and_then(|mut f| {
            let mut s = String::new();
            f.read_to_string(&mut s).map(|_| s)
        })?;
        Ok(Self::from_str(data, &project_paths.root)?)
    }

    pub fn from_str(manifest_source: String, root: &Path) -> Result<Self, ::failure::Error> {
        let manifest_pair = parse_and_check_manifest(manifest_source)?;

        Ok(Self::from_manifest_pair(&manifest_pair, root)?)
    }

    pub fn from_manifest_pair(manifest_pair: &Pair, root: &Path) -> Result<Self, ::failure::Error> {
        let dependencies = get_dependencies(manifest_pair)?;

        let package_arguments_pair =
            get_optional_field(&manifest_pair, "package").ok_or_else(|| {
                // We use get_optional_field and .ok_or_else to produce a
                // clearer error message.
                format_err!("A `package {{ ... }}` section is required to publish this package")
                    .with_pos(&manifest_pair.clone().into_span().end_pos())
            })?;
        let block_pair = Arguments::from_pair(package_arguments_pair, 0, 0, &[], Some(true))?
            .block
            .expect("validated block presence");

        check_block_fields(
            &block_pair,
            &[
                "name",
                "version",
                "description",
                "authors",
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
            let name_pair = Arguments::get_single(get_field(&block_pair, "name")?)?;

            let name_string = get_string(&name_pair)?;
            PackageName::from_str(&name_string)
                .ok_or_else(|| format_err!("Invalid package name").with_pair(&name_pair))?
        };

        let version = {
            let version_pair = Arguments::get_single(get_field(&block_pair, "version")?)?;
            let version_string = get_string(&version_pair)?;
            Version::from_str(&version_string)
                .ok_or_else(|| format_err!("Invalid version number").with_pair(&version_pair))?
        };

        let description = get_string(&Arguments::get_single(get_field(
            &block_pair,
            "description",
        )?)?)?;

        let homepage = get_optional_string_field(&block_pair, "homepage")?;
        let repository = get_optional_string_field(&block_pair, "repository")?;
        let bugs = get_optional_string_field(&block_pair, "bugs")?;

        let authors = get_optional_list_field(&block_pair, "authors")?
            .into_iter()
            .map(|i| get_string(&i))
            .collect::<Result<_, _>>()?;
        let keywords = get_optional_list_field(&block_pair, "keywords")?
            .into_iter()
            .map(|i| get_string(&i))
            .collect::<Result<_, _>>()?;

        let license = get_optional_string_field(&block_pair, "license")?;
        let license_file = get_optional_string_field(&block_pair, "license_file")?;

        if license.is_none() && license_file.is_none() {
            return Err(::failure::Error::from(
                format_err!("package section needs at least one of license or license_file")
                    .with_pos(&block_pair.into_span().start_pos()),
            ));
        }

        let files_block = Arguments::get_block(get_field(&block_pair, "files")?)?;
        let files = evaluate_files_block(&files_block, root)?;

        Ok(Manifest {
            name,
            version,

            dependencies,

            authors,
            description,
            homepage,
            repository,
            bugs,
            keywords,

            license,
            license_file,

            readme: None, // TODO

            files,
        })
    }
}

pub fn parse_and_check_manifest(manifest_source: String) -> Result<Pair, ::failure::Error> {
    let manifest_pair = parse_manifest(manifest_source)?;

    check_block_fields(
        &manifest_pair,
        &[
            "pm", // TODO do something with this version tag (if present)
            "dependencies",
            "package",
        ],
    )?;

    Ok(manifest_pair)
}

pub fn get_dependencies(manifest_pair: &Pair) -> Result<Vec<Dependency>, ::failure::Error> {
    let mut depset = Vec::<Dependency>::new();
    for (package_name_pair, arguments_pair) in
        get_optional_block_field(&manifest_pair, "dependencies")?
    {
        let arguments = Arguments::from_pair(arguments_pair, 0, 2, &[], Some(false))?;
        let (package_name, version_constraint) =
            make_dependency(&package_name_pair, &arguments.positional_arguments)?;
        for dep in depset.iter() {
            if dep.package_name == package_name {
                return Err(::failure::Error::from(
                    format_err!("Duplicate dependency").with_pair(&package_name_pair),
                ));
            }
        }
        depset.push(Dependency {
            package_name,
            version_constraint,
        });
    }
    Ok(depset)
}

pub fn make_dependency(
    package_name_pair: &Pair,
    vcc_pairs: &[Pair],
) -> Result<(PackageName, VersionConstraint), ::failure::Error> {
    let package_name = PackageName::from_str(package_name_pair.as_str())
        .ok_or_else(|| format_err!("Invalid package name").with_pair(&package_name_pair))?;

    let version_constraint = match vcc_pairs.len() {
        0 => VersionConstraint::from_str("*"),
        1 => {
            let vc_component = vcc_pairs[0].clone();
            VersionConstraint::from_str(vc_component.into_span().as_str())
        }
        2 => {
            let vcc1 = vcc_pairs[0].clone(); // e.g. ">=2.0.0"
            let vcc2 = vcc_pairs[1].clone(); // e.g. "<4.0.0"
            VersionConstraint::from_str(&format!(
                "{} {}",
                vcc1.into_span().as_str(),
                vcc2.into_span().as_str()
            ))
        }
        _ => unreachable!(),
    }
    .ok_or_else(|| {
        format_err!("Invalid version constraint")
            .with_pos(&vcc_pairs[0].clone().into_span().start_pos())
    })?;

    Ok((package_name, version_constraint))
}

pub fn evaluate_files_block(
    files_block_pair: &Pair,
    root: &Path,
) -> Result<Vec<String>, ::failure::Error> {
    let mut file_section_interpreter = FilesSectionInterpreter::new(root.to_path_buf())?;
    let mut file_set = HashSet::<String>::new();
    for (symbol_pair, arguments_pair) in get_fields(&files_block_pair) {
        match symbol_pair.as_str() {
            "add_committed" => {
                let glob_pair = Arguments::get_single(arguments_pair)?;
                let glob = get_string(&glob_pair)?;

                file_section_interpreter
                    .add_committed(&mut file_set, &glob)
                    .pair_context(&glob_pair)?;
            }
            "add_any" => {
                let glob_pair = Arguments::get_single(arguments_pair)?;
                let glob = get_string(&glob_pair)?;

                file_section_interpreter
                    .add_any(&mut file_set, &glob)
                    .pair_context(&glob_pair)?;
            }
            "remove" => {
                let glob_pair = Arguments::get_single(arguments_pair)?;
                let glob = get_string(&glob_pair)?;

                file_section_interpreter
                    .remove(&mut file_set, &glob)
                    .pair_context(&glob_pair)?;
            }
            _ => {
                return Err(::failure::Error::from(
                    format_err!("Expected `add_committed`, `add_any`, or `remove`")
                        .with_pair(&symbol_pair),
                ));
            }
        }
    }
    let mut file_set_vec: Vec<String> = file_set.into_iter().collect();
    file_set_vec.sort_unstable();
    Ok(file_set_vec)
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
