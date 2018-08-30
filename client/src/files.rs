#![allow(dead_code)]

use failure;
use git::GitScmProvider;
use glob::{MatchOptions, Pattern, PatternError};
use std::collections::{BTreeSet, HashSet};
use std::error::Error as StdError;
use std::fmt;
use std::path::{Path, PathBuf};

// Case insensitivity handling for Windows is currently missing; e.g. "*.rs" in
// glob but "foo.RS" on disk. For Git, we follow the casing returned by Git, but
// for uncommitted files we might want to match case insensitively or
// alternatively throw useful error messages if the casing differs.

// This needs to be a HashSet because BTreeSet doesn't have a .retain
// method. When we iterate, we need to make sure to avoid non-deterministic
// behavior due to ordering.
type FileSet = HashSet<String>;

pub struct FilesSectionInterpreter {
    pub root: PathBuf,
    pub files_on_disk: BTreeSet<String>,
    pub directories_on_disk: BTreeSet<String>,
    pub committed_files: Vec<String>,
    pub did_initialize_committed_files: bool,
}

#[derive(Fail, Debug)]
#[fail(display = "No such file or directory (in Git)")]
pub struct AddCommittedNotFoundError;

#[derive(Fail, Debug)]
#[fail(display = "No such file or directory")]
pub struct AddUncommittedNotFoundError;

// Interpret "add" and "remove" commands in the `files { ... }` section.
impl FilesSectionInterpreter {
    pub fn new(root: PathBuf) -> Result<Self, failure::Error> {
        let (files, directories) = walk_dir(&root)?;
        Ok(FilesSectionInterpreter {
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
            git_scm_provider.check_repo_is_pristine()?;
            self.committed_files = git_scm_provider.ls_files()?;
            self.did_initialize_committed_files = true;
        }
        Ok(())
    }

    pub fn add_committed(
        &mut self,
        file_set: &mut FileSet,
        glob: &str,
    ) -> Result<(), failure::Error> {
        self.initialize_committed_files()?;
        let cglob = CompiledGlob::new(glob)?;
        let mut did_match = false;
        for file in &self.committed_files {
            if self.pattern_matches_path_or_ancestor(&cglob, file) {
                file_set.insert(file.clone());
                did_match = true;
            }
        }
        if !did_match {
            Err(failure::Error::from(AddCommittedNotFoundError))
        } else {
            Ok(())
        }
    }

    pub fn add_uncommitted(
        &mut self,
        file_set: &mut FileSet,
        glob: &str,
    ) -> Result<(), failure::Error> {
        let mut did_match = false;
        let cglob = CompiledGlob::new(glob)?;
        for file in self.files_on_disk.iter() {
            if self.pattern_matches_path_or_ancestor(&cglob, &file) {
                did_match = true;
                file_set.insert(file.clone());
            }
        }

        if !did_match {
            // We may also want to provide a more helpful error message if a
            // glob only matches case-insensitively.
            Err(failure::Error::from(AddUncommittedNotFoundError))
        } else {
            Ok(())
        }
    }

    pub fn remove(&mut self, file_set: &mut FileSet, glob: &str) -> Result<(), failure::Error> {
        let cglob = CompiledGlob::new(glob)?;
        file_set.retain(|file| {
            !self.pattern_matches_path_or_ancestor(&cglob, file)
        });

        Ok(())
    }

    fn pattern_matches_path_or_ancestor(&self, cglob: &CompiledGlob, file: &str) -> bool {
        let mut p: &str = file;
        // Given foo/bar.txt, try matching cglob against
        //
        // * foo/bar.txt
        // * foo/
        // * foo
        // * <root>
        loop {
            if cglob.matches_non_root(p) {
                return true;
            } else {
                if p.ends_with('/') {
                    p = &p[..(p.len() - 1)];
                } else {
                    match p.rfind('/') {
                        Some(i) => p = &p[..(i + 1)],
                        None => {
                            break;
                        }
                    }
                }
            }
        }
        if cglob.matches_root() {
            return true;
        }
        false
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

    fn make_fsi(files: &[&str]) -> FilesSectionInterpreter {
        FilesSectionInterpreter {
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

    fn assert_file_set(file_set: &FileSet, files: &[&str]) {
        let mut file_set_v: Vec<String> = file_set.clone().into_iter().collect();
        file_set_v.sort();
        assert_eq!(
            file_set_v,
            files.iter().map(|s| s.to_string()).collect::<Vec<String>>()
        );
    }

    mod process_glob {
        use super::*;

        fn glob_error_for(glob: &str) -> GlobError {
            unwrap_glob_error(make_fsi(&[]).add_uncommitted(&mut FileSet::new(), glob))
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
        fn include_and_exclude() {
            let mut fsi = make_fsi(&[
                "src/a.rs",
                "src/b.rs",
                "src/vendor/a.rs",
                "src/vendor/b.rs",
                "test/a.rs",
            ]);
            let mut file_set = FileSet::new();
            fsi.add_uncommitted(&mut file_set, "src/**").unwrap();
            fsi.remove(&mut file_set, "src/vendor/*").unwrap();
            fsi.add_uncommitted(&mut file_set, "src/vendor/a.rs")
                .unwrap();

            assert_file_set(&file_set, &["src/a.rs", "src/b.rs", "src/vendor/a.rs"]);
        }

        #[test]
        fn include_directory() {
            let mut fsi = make_fsi(&[
                "a.rs",
                "src/b.rs",
                "src/vendor/c.rs"
            ]);
            let mut file_set = FileSet::new();
            fsi.add_uncommitted(&mut file_set, "src").unwrap();
            assert_file_set(&file_set, &["src/b.rs", "src/vendor/c.rs"]);

            let mut file_set2 = FileSet::new();
            fsi.add_uncommitted(&mut file_set2, "src/").unwrap();
            assert_file_set(&file_set2, &["src/b.rs", "src/vendor/c.rs"]);
        }

        #[test]
        fn exclude_directory() {
            let mut fsi = make_fsi(&["src/a.rs"]);
            let mut file_set = FileSet::new();
            fsi.add_uncommitted(&mut file_set, "**").unwrap();
            fsi.remove(&mut file_set, "src").unwrap();
            assert_file_set(&file_set, &[]);
        }

        #[test]
        fn exclude_directory_trailing_slash() {
            let mut fsi = make_fsi(&["src/a.rs"]);
            let mut file_set = FileSet::new();
            fsi.add_uncommitted(&mut file_set, "**").unwrap();
            fsi.remove(&mut file_set, "src/").unwrap();
            assert_file_set(&file_set, &[]);
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
        //         match fsi(&["jquery.js"], &["jQuery.js"]) {
        //             Err(GlobError::NotFound) => {},
        //             r @ _ => panic!("{:?}", r),
        //         }
        //     }
    }
}

#[derive(Debug)]
enum CompiledGlob {
    Root,
    NonRoot(Pattern)
}

static MATCH_OPTIONS: MatchOptions = MatchOptions {
    case_sensitive: true,
    // foo/*/bar should not match foo/a/b/bar
    require_literal_separator: true,
    // * should not match .dotfile
    require_literal_leading_dot: true,
};

impl CompiledGlob {
    pub fn new(glob: &str) -> Result<Self, GlobError> {
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

        if glob == "." || glob == "./" {
            Ok(CompiledGlob::Root)
        } else {
            let pattern = Pattern::new(glob).map_err(|e| GlobError::PatternError(e))?;
            Ok(CompiledGlob::NonRoot(pattern))
        }
    }

    // The reason why we don't have a single "matches" function is that the glob
    // library thinks that ".*" matches ".", and "*" matches "". So we treat the
    // root case separately.
    pub fn matches_non_root(&self, p: &str) -> bool {
        match self {
            CompiledGlob::Root => false,
            CompiledGlob::NonRoot(pattern) => pattern.matches_with(p, &MATCH_OPTIONS)
        }
    }

    pub fn matches_root(&self) -> bool {
        match self {
            CompiledGlob::Root => true,
            CompiledGlob::NonRoot(_) => false
        }
    }
}
