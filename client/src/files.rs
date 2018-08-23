#![allow(dead_code)]

use failure;
use git::GitScmProvider;
use glob::{MatchOptions, Pattern, PatternError};
use std::collections::{BTreeSet, HashSet};
use std::error::Error as StdError;
use std::fmt;
use std::path::{Path, PathBuf};

// This needs to be a HashSet because BTreeSet doesn't have a .retain
// method. When we iterate, we need to make sure to avoid non-deterministic
// behavior due to ordering.
type FileSet = HashSet<String>;

pub struct FileCollection {
    pub root: PathBuf,
    pub files_on_disk: BTreeSet<String>,
    pub directories_on_disk: BTreeSet<String>,
    pub committed_files: Vec<String>,
    pub did_initialize_committed_files: bool,
}

#[derive(Fail, Debug)]
#[fail(display = "No such file or directory")]
pub struct AddNotFoundError;

#[derive(Fail, Debug)]
#[fail(display = "No such file or directory in version control system")]
pub struct AddCommittedNotFoundError;

#[derive(Fail, Debug)]
#[fail(display = "Version control system returned 0 committed files")]
pub struct AddAllCommittedNotFoundError;

impl FileCollection {
    pub fn new(root: PathBuf) -> Result<Self, failure::Error> {
        let (files, directories) = walk_dir(&root)?;
        Ok(FileCollection {
            root: root,
            files_on_disk: files,
            directories_on_disk: directories,
            committed_files: Vec::new(),
            did_initialize_committed_files: false,
        })
    }

    // Populate `committed_files` with the results of `git ls-files`.
    pub fn initialize_committed_files(&mut self) -> Result<(), failure::Error> {
        if !self.did_initialize_committed_files {
            let git_scm_provider = GitScmProvider::new(&self.root)?;
            self.committed_files = git_scm_provider.ls_files()?;
            self.did_initialize_committed_files = true;
        }
        Ok(())
    }

    pub fn add_committed(
        &mut self,
        selected_files: &mut FileSet,
        glob: &str,
    ) -> Result<(), failure::Error> {
        self.initialize_committed_files()?;
        let pattern = self.parse_glob(glob)?;
        let mut did_match = false;
        for file in &self.committed_files {
            if FileCollection::pattern_matches(&pattern, file) {
                if !self.files_on_disk.contains(file) {
                    return Err(format_err!(
                        "File is committed to SCM but missing in working tree: {}",
                        file
                    ));
                }
                selected_files.insert(file.clone());
                did_match = true;
            }
        }
        if !did_match {
            Err(failure::Error::from(AddCommittedNotFoundError))
        } else {
            Ok(())
        }
    }

    pub fn parse_glob(&self, glob: &str) -> Result<Pattern, GlobError> {
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
        if glob.starts_with("/") || (glob.len() >= 2 && glob.as_bytes()[1] == b':') {
            return Err(GlobError::Absolute);
        }
        Pattern::new(glob).map_err(|pattern_error| GlobError::PatternError(pattern_error))
    }

    pub fn pattern_matches(pattern: &Pattern, file: &str) -> bool {
        pattern.matches_with(&file, &FileCollection::match_options())
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

    pub fn add(&mut self, selected_files: &mut FileSet, glob: &str) -> Result<(), failure::Error> {
        let mut did_match = false;
        let pattern = self.parse_glob(glob)?;
        for file in self.files_on_disk.iter() {
            if FileCollection::pattern_matches(&pattern, &file) {
                did_match = true;
                selected_files.insert(file.to_string());
            }
        }

        if !did_match {
            // Produce a helpful error if the glob matches a directory (but no
            // files).
            for dir in self.directories_on_disk.iter() {
                if FileCollection::pattern_matches(&pattern, &dir)
                    || FileCollection::pattern_matches(&pattern, &format!("{}/", dir))
                {
                    return Err(failure::Error::from(GlobError::Directory));
                }
            }

            // We may also want to provide a more helpful error message if a
            // glob only matches case-insensitively.

            return Err(failure::Error::from(GlobError::NotFound));
        }

        Ok(())
    }

    pub fn remove(
        &mut self,
        selected_files: &mut FileSet,
        glob: &str,
    ) -> Result<(), failure::Error> {
        let pattern = self.parse_glob(glob)?;
        let match_options = FileCollection::match_options();
        selected_files.retain(|file| {
            // Given file x/y, check if the glob matches x/y, x/ or x
            // to enable excluding entire directories.
            let mut slice: &str = &file;
            loop {
                if pattern.matches_with(slice, &match_options) {
                    break false;
                } else {
                    if *slice.as_bytes().last().unwrap() == b'/' {
                        slice = &slice[..(slice.len() - 1)];
                    } else {
                        match slice.rfind('/') {
                            Some(i) => slice = &slice[..(i + 1)],
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
}

fn walk_dir(root: &Path) -> Result<(BTreeSet<String>, BTreeSet<String>), failure::Error> {
    let mut files = BTreeSet::new();
    let mut directories = BTreeSet::new();
    walk_dir_inner(root, "", &mut files, &mut directories)?;
    Ok((files, directories))
}

fn walk_dir_inner(
    root: &Path,
    prefix: &str,
    files: &mut BTreeSet<String>,
    directories: &mut BTreeSet<String>,
) -> Result<(), failure::Error> {
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
            GlobError::Backslash => {
                r#"Unexpected backslash (\). Use forward slashes (/) as path separators instead."#
            }
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
            committed_files: Vec::new(),
            did_initialize_committed_files: true,
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

    fn assert_selected(selected_files: &FileSet, files: &[&str]) {
        let mut selected_files_v: Vec<String> = selected_files.clone().into_iter().collect();
        selected_files_v.sort();
        assert_eq!(
            selected_files_v,
            files.iter().map(|s| s.to_string()).collect::<Vec<String>>()
        );
    }

    mod process_glob {
        use super::*;

        fn glob_error_for(glob: &str) -> GlobError {
            unwrap_glob_error(make_fc(&[]).add(&mut FileSet::new(), glob))
        }

        fn unwrap_glob_error(glob_result: Result<(), ::failure::Error>) -> GlobError {
            glob_result.unwrap_err().downcast::<GlobError>().unwrap()
        }

        #[test]
        fn invalid_globs() {
            assert_matches!(glob_error_for(""), GlobError::Empty);
            assert_matches!(glob_error_for("foo\\bar"), GlobError::Backslash);
            assert_matches!(glob_error_for("/foo"), GlobError::Absolute);
            assert_matches!(glob_error_for("c:/foo"), GlobError::Absolute);
            // We're reserving these in case we want to make them significant
            // glob syntax in the future.
            assert_matches!(glob_error_for("!foo"), GlobError::ExclamationPoint);
            assert_matches!(glob_error_for("foo{"), GlobError::Braces);
            assert_matches!(glob_error_for("foo}"), GlobError::Braces);
            assert_matches!(glob_error_for("fo[^o]"), GlobError::Caret);
            assert_matches!(glob_error_for(""), GlobError::Empty);
            // Rejected by the globbing library.
            assert_matches!(glob_error_for("***"), GlobError::PatternError(_));
        }

        #[test]
        fn not_found() {
            assert_matches!(glob_error_for("missing"), GlobError::NotFound);
        }

        #[test]
        fn include_and_exclude() {
            let mut fc = make_fc(&[
                "src/a.rs",
                "src/b.rs",
                "src/vendor/a.rs",
                "src/vendor/b.rs",
                "test/a.rs",
            ]);
            let mut file_set = FileSet::new();
            fc.add(&mut file_set, "src/**").unwrap();
            fc.remove(&mut file_set, "src/vendor/*").unwrap();
            fc.add(&mut file_set, "src/vendor/a.rs").unwrap();

            assert_selected(&file_set, &["src/a.rs", "src/b.rs", "src/vendor/a.rs"]);
        }

        #[test]
        fn include_directory() {
            assert_matches!(
                unwrap_glob_error(make_fc(&["src/foo.rs"]).add(&mut FileSet::new(), "src")),
                GlobError::Directory
            );
            assert_matches!(
                unwrap_glob_error(make_fc(&["src/foo.rs"]).add(&mut FileSet::new(), "src/")),
                GlobError::Directory
            );
        }

        #[test]
        fn exclude_directory() {
            let mut fc = make_fc(&["src/a.rs"]);
            let mut file_set = FileSet::new();
            fc.add(&mut file_set, "**").unwrap();
            fc.remove(&mut file_set, "src").unwrap();
            assert_selected(&file_set, &[]);
        }

        #[test]
        fn exclude_directory_trailing_slash() {
            let mut fc = make_fc(&["src/a.rs"]);
            let mut file_set = FileSet::new();
            fc.add(&mut file_set, "**").unwrap();
            fc.remove(&mut file_set, "src/").unwrap();
            assert_selected(&file_set, &[]);
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
