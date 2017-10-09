#![allow(dead_code)]

use glob::{Pattern,PatternError,MatchOptions};
use std::collections::{BTreeSet, HashSet};
use std::path::{Path,PathBuf};
use std::fmt;
use std::error::Error as StdError;
use error::Error;

pub struct FileCollection {
    pub root: PathBuf,
    pub files_on_disk: BTreeSet<String>,
    pub directories_on_disk: BTreeSet<String>,

    // This needs to be a HashSet because BTreeSet doesn't have a .retain
    // method. When we iterate, we need to make sure to avoid non-deterministic
    // behavior due to ordering.
    pub selected_files: HashSet<String>,
}

impl FileCollection {
    pub fn new(root: PathBuf) -> Result<Self, Error>
    {
        let (files, directories) = walk_dir(&root)?;
        Ok(FileCollection {
            root: root,
            files_on_disk: files,
            directories_on_disk: directories,
            selected_files: HashSet::new(),
        })
    }

    // For use by SCM providers.
    pub fn add_file(&mut self, file: String) -> Result<(), ()> {
        if self.files_on_disk.contains(&file) {
            self.selected_files.insert(file);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn parse_glob(&self, maybe_negative_glob: &str)
        -> Result<(bool, Pattern), GlobError>
    {
        // The various checks we perform here are purely for usability to avoid
        // accidentally having globs that cannot match anything.
        if maybe_negative_glob.is_empty() {
            return Err(GlobError::Empty);
        }
        let (negate, glob)
            = if maybe_negative_glob.starts_with('!') {
                (true, &maybe_negative_glob[1..])
            } else {
                (false, maybe_negative_glob)
            };
        if glob.contains('{') || glob.contains('}') {
            return Err(GlobError::Braces);
        }
        if glob.contains('\\') {
            return Err(GlobError::Backslash);
        }
        if glob.contains("[^") {
            return Err(GlobError::Caret);
        }
        if glob.starts_with("/")
            || (glob.len() >= 2 && glob.as_bytes()[1] == b':')
        {
            return Err(GlobError::Absolute);
        }
        match Pattern::new(glob) {
            Err(pattern_error) => Err(GlobError::PatternError(pattern_error)),
            Ok(pattern) => Ok((negate, pattern))
        }
    }

    pub fn process_glob(&mut self, glob: &str) -> Result<(), GlobError> {
        let match_options = MatchOptions {
            case_sensitive: true,
            // foo/*/bar should not match foo/a/b/bar
            require_literal_separator: true,
            // * should not match .dotfile
            require_literal_leading_dot: true,
        };
        let (negate, pattern) = self.parse_glob(glob)?;
        let mut did_match = false;
        if !negate {
            for file in self.files_on_disk.iter() {
                if pattern.matches_with(&file, &match_options) {
                    did_match = true;
                    self.selected_files.insert(file.to_string());
                }
            }
        } else {
            self.selected_files.retain(|file| {
                // Given file a/b/c, check if the glob matches a/b/c, a/b or a
                // to enable excluding entire directories.
                let mut slice: &str = &file;
                loop {
                    if pattern.matches_with(slice, &match_options) {
                        did_match = true;
                        break false;
                    } else {
                        let mut it = slice.rsplitn(2, '/');
                        it.next();
                        match it.next() {
                            Some(s) => {
                                slice = s;
                            }
                            None => {
                                break true;
                            }
                        }
                    }
                }
            });
        }
        if !did_match {
            // For improved usability, we may want more specific errors
            //
            // * if a positive glob matches one of self.directories_on_disk,
            //   (we can suggest `{}/**`), and
            // * if a glob matches case-insensitively.
            return Err(GlobError::NotFound);
        }
        Ok(())
    }

    pub fn get_selected_files(&self) -> Vec<String> {
        let mut v: Vec<String> = self.selected_files.clone().into_iter().collect();
        v.sort_unstable();
        v
    }
}

fn walk_dir(root: &Path)
    -> Result<(BTreeSet<String>, BTreeSet<String>), Error>
{
    let mut files = BTreeSet::new();
    let mut directories = BTreeSet::new();
    walk_dir_inner(root, "", &mut files, &mut directories)?;
    Ok((files, directories))
}

fn walk_dir_inner(root: &Path, prefix: &str, files: &mut BTreeSet<String>, directories: &mut BTreeSet<String>)
    -> Result<(), Error>
{
    for maybe_entry in root.read_dir()? {
        let entry = maybe_entry?;
        // Slightly-lazy error handling: Non-decodable byte sequences on Unix
        // turn into "U+FFFD REPLACEMENT CHARACTER" here. If the user selects
        // such a file, then when we later try to read it, we will encounter a
        // file-not-found error.
        let entry_name_osstring = entry.file_name();
        let entry_name = entry_name_osstring.to_string_lossy();
        let entry_path = format!("{}{}", prefix, entry_name);

        if entry.file_type()?.is_dir() {
            directories.insert(entry_path.clone());
            let entry_prefix = format!("{}/", entry_path);
            walk_dir_inner(&entry.path(), &entry_prefix, files, directories)?;
        } else {
            files.insert(entry_path);
        }
    }
    Ok(())
}


#[derive(Debug)]
pub enum GlobError {
    Braces,
    Backslash,
    Caret,
    Absolute,
    Empty,
    PatternError(PatternError), // anything caught by the glob library

    NotFound,
}

impl ::std::error::Error for GlobError {
    fn description(&self) -> &str {
        match *self {
            GlobError::Braces => r#"Unexpected curly brace"#,
            GlobError::Backslash => r#"Unexpected backslash (\). Use forward slashes (/) as path separators instead."#,
            GlobError::Caret => r#"[^...] syntax is not supported. Use [!...] instead."#,
            GlobError::Absolute => r#"Expected relative path"#,
            GlobError::Empty => r#"Expected glob pattern, found empty string"#,
            GlobError::PatternError(_) => r#"Invalid glob syntax"#,

            GlobError::NotFound => r#"File(s) not found"#,
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match self {
            &GlobError::PatternError(ref pattern_error) => Some(pattern_error),
            _ => None,
        }
    }
}

impl fmt::Display for GlobError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}
