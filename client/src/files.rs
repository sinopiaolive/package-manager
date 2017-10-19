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
    pub fn add_file(&mut self, file: String) -> Result<(), Error> {
        if self.files_on_disk.contains(&file) {
            self.selected_files.insert(file);
            Ok(())
        } else {
            Err(Error::Message(format!("File is committed to SCM but missing in working tree: {}", &file)))
        }
    }

    pub fn parse_glob(&self, glob: &str)
        -> Result<Pattern, GlobError>
    {
        // The various checks here are purely to provide better error messages
        // than "file not found" for various syntax errors.
        if glob == "" {
            return Err(GlobError::Empty);
        }
        if glob.starts_with('!') {
            return Err(GlobError::ExclamationPoint);
        }
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
            Ok(pattern) => Ok(pattern)
        }
    }

    pub fn match_options() -> MatchOptions {
        MatchOptions {
            case_sensitive: true,
            // foo/*/bar should not match foo/a/b/bar
            require_literal_separator: true,
            // * should not match .dotfile
            require_literal_leading_dot: true,
        }
    }

    pub fn add(&mut self, glob: &str) -> Result<(), GlobError> {
        let mut did_match = false;
        let pattern = self.parse_glob(glob)?;
        let match_options = FileCollection::match_options();
        for file in self.files_on_disk.iter() {
            if pattern.matches_with(&file, &match_options) {
                did_match = true;
                self.selected_files.insert(file.to_string());
            }
        }

        if !did_match {
            // Produce a helpful error if the glob matches a directory (but no
            // files).
            for dir in self.directories_on_disk.iter() {
                if pattern.matches_with(&dir, &match_options) ||
                    pattern.matches_with(&format!("{}/", dir), &match_options)
                {
                    return Err(GlobError::Directory);
                }
            }

            // We may also want to provide a more helpful error message if a
            // glob only matches case-insensitively.

            return Err(GlobError::NotFound);
        }

        Ok(())
    }

    pub fn remove(&mut self, glob: &str) -> Result<(), GlobError> {
        let pattern = self.parse_glob(glob)?;
        let match_options = FileCollection::match_options();
        self.selected_files.retain(|file| {
            // Given file x/y, check if the glob matches x/y, x/ or x
            // to enable excluding entire directories.
            let mut slice: &str = &file;
            loop {
                if pattern.matches_with(slice, &match_options) {
                    break false;
                } else {
                    if *slice.as_bytes().last().unwrap() == b'/' {
                        slice = &slice[..(slice.len()-1)];
                    } else {
                        match slice.rfind('/') {
                            Some(i) => {
                                slice = &slice[..(i+1)]
                            }
                            None => {
                                break true;
                            }
                        }
                    }
                }
            }
        });

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
    Empty,
    ExclamationPoint,
    Braces,
    Backslash,
    Caret,
    Absolute,
    PatternError(PatternError), // anything caught by the glob library

    Directory,
    NotFound,
}

impl StdError for GlobError {
    fn description(&self) -> &str {
        match *self {
            GlobError::Empty => r#"Expected file path or glob pattern"#,
            GlobError::ExclamationPoint => r#"Unexpected `!`"#,
            GlobError::Braces => r#"Unexpected curly brace"#,
            GlobError::Backslash => r#"Unexpected backslash (\). Use forward slashes (/) as path separators instead."#,
            GlobError::Caret => r#"[^...] syntax is not supported. Use [!...] instead."#,
            GlobError::Absolute => r#"Expected relative path"#,
            GlobError::PatternError(_) => r#"Invalid glob syntax"#,

            GlobError::Directory => r#"Expected file, found directory"#,
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


#[cfg(test)]
mod test {
    use super::*;

    fn make_fc(files: &[&str]) -> FileCollection {
        FileCollection {
            root: PathBuf::from("dummy"),
            files_on_disk: files.iter().map(|s| s.to_string()).collect(),
            directories_on_disk: generate_directories(files),
            selected_files: HashSet::new(),
        }
    }

    fn generate_directories(files: &[&str]) -> BTreeSet<String> {
        let mut directories = BTreeSet::new();
        for file in files {
            let mut slice: &str = file;
            loop {
                match slice.rfind('/') {
                    Some(i) => {
                        slice = &slice[..i];
                    }
                    None => {
                        break;
                    }
                }
                directories.insert(slice.to_string());
            }
        }
        directories
    }

    fn assert_selected(fc: &FileCollection, files: &[&str]) {
        assert_eq!(
            fc.get_selected_files(),
            files.iter().map(|s| s.to_string()).collect::<Vec<String>>()
        );
    }

    mod process_glob {
        use super::*;

        #[test]
        fn invalid_globs() {
            match make_fc(&[]).add("") {
                Err(GlobError::Empty) => {},
                r @ _ => panic!("{:?}", r),
            }
            match make_fc(&[]).add("foo\\bar") {
                Err(GlobError::Backslash) => {},
                r @ _ => panic!("{:?}", r),
            }
            match make_fc(&[]).add("/foo") {
                Err(GlobError::Absolute) => {},
                r @ _ => panic!("{:?}", r),
            }
            match make_fc(&[]).add("c:/foo") {
                Err(GlobError::Absolute) => {},
                r @ _ => panic!("{:?}", r),
            }

            // We're reserving these in case we want to make them significant
            // glob syntax in the future.
            match make_fc(&[]).add("!foo") {
                Err(GlobError::ExclamationPoint) => {},
                r @ _ => panic!("{:?}", r),
            }
            match make_fc(&[]).add("foo{") {
                Err(GlobError::Braces) => {},
                r @ _ => panic!("{:?}", r),
            }
            match make_fc(&[]).add("foo}") {
                Err(GlobError::Braces) => {},
                r @ _ => panic!("{:?}", r),
            }
            match make_fc(&[]).add("fo[^o]") {
                Err(GlobError::Caret) => {},
                r @ _ => panic!("{:?}", r),
            }

            match make_fc(&[]).add("***") {
                Err(GlobError::PatternError(_)) => {},
                r @ _ => panic!("{:?}", r),
            }
        }

        #[test]
        fn not_found() {
            match make_fc(&[]).add("missing") {
                Err(GlobError::NotFound) => {},
                r @ _ => panic!("{:?}", r),
            }
        }

        #[test]
        fn include_and_exclude() {
            let mut fc = make_fc(&["src/a.rs", "src/b.rs", "src/vendor/a.rs", "src/vendor/b.rs", "test/a.rs"]);
            fc.add("src/**").unwrap();
            fc.remove("src/vendor/*").unwrap();
            fc.add("src/vendor/a.rs").unwrap();

            assert_selected(&fc, &["src/a.rs", "src/b.rs", "src/vendor/a.rs"]);
        }

        #[test]
        fn include_directory() {
            match make_fc(&["src/foo.rs"]).add("src") {
                Err(GlobError::Directory) => {},
                r @ _ => panic!("{:?}", r),
            }

            match make_fc(&["src/foo.rs"]).add("src/") {
                Err(GlobError::Directory) => {},
                r @ _ => panic!("{:?}", r),
            }
        }

        #[test]
        fn exclude_directory() {
            let mut fc = make_fc(&["src/a.rs"]);
            fc.add("**").unwrap();
            fc.remove("src").unwrap();
            assert_selected(&fc, &[]);
        }

        #[test]
        fn exclude_directory_trailing_slash() {
            let mut fc = make_fc(&["src/a.rs"]);
            fc.add("**").unwrap();
            fc.remove("src/").unwrap();
            assert_selected(&fc, &[]);
        }

    //     mod star_behavior {
    //         use super::*;

    //         #[test]
    //         fn single_star() {
    //             assert_matches(
    //                 &["a/*/foo.rs"],
    //                 &["a/b/foo.rs"],
    //                 &["a/b/c/foo.rs", "a/foo.rs", "a/.dot/foo.rs"]
    //             );
    //         }

    //         #[test]
    //         fn double_star() {
    //             assert_matches(
    //                 &["a/**/foo.rs"],
    //                 &["a/b/c/foo.rs", "a/b/foo.rs", "a/foo.rs"],
    //                 &["a/.dot/foo.rs", "a/b/.dot/foo.rs"]
    //             );
    //         }

    //         #[test]
    //         fn trailing_double_star() {
    //             assert_matches(
    //                 &["**"],
    //                 &["a/b/foo.rs", "foo.rs"],
    //                 &[".foo.rs", ".a/b/foo.rs", "a/b/.foo.rs"]
    //             );
    //         }

    //         #[test]
    //         fn negated_trailing_double_star() {
    //             // !a/** excludes a/.dot because it matches the a/ directory,
    //             // excluding it entirely. People may come to rely on this so we
    //             // test it explicitly here.
    //             assert_matches(
    //                 &["a/*", "a/.dot/.foo.rs", "!a/**"],
    //                 &[],
    //                 &["a/foo.rs", "a/.dot/.foo.rs"]
    //             );
    //         }

    //         #[test]
    //         fn negated_double_star() {
    //             // Edge case: !** does not exclude the root directory.
    //             assert_matches(
    //                 &[".dot", "foo", "!**"],
    //                 &[".dot"],
    //                 &["foo"]
    //             );
    //         }
    //     }

    //     #[test]
    //     fn test_case_sensitive() {
    //         match fc(&["jquery.js"], &["jQuery.js"]) {
    //             Err(GlobError::NotFound) => {},
    //             r @ _ => panic!("{:?}", r),
    //         }
    //     }
    }
}
