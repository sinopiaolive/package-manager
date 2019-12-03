#![allow(dead_code)]

use crate::git::GitScmProvider;
use failure;
use glob::{MatchOptions, Pattern, PatternError};
use std::collections::{BTreeMap, HashSet};
use std::error::Error as StdError;
use std::fmt;
use std::path::PathBuf;

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
    pub vcs_file_set: VCSFileSet,
    pub did_initialize_vcs_file_set: bool,
}

#[derive(Fail, Debug)]
enum FileSetError {
    #[fail(display = "No such file or directory")]
    NotFound,

    #[fail(display = "Only files ignored by Git were found")]
    IgnoredOnly,

    #[fail(display = "Some files are untracked or changed")]
    NotCommitted {
        changed: Vec<String>,
        untracked: Vec<String>,
    },
}

pub enum VCSFileStatus {
    Tracked,
    Changed,
    Untracked,
    Ignored,
}

pub type VCSFileSet = BTreeMap<String, VCSFileStatus>;

// Interpret "add" and "remove" commands in the `files { ... }` section.
impl FilesSectionInterpreter {
    pub fn new(root: PathBuf) -> Result<Self, failure::Error> {
        Ok(FilesSectionInterpreter {
            root,
            vcs_file_set: VCSFileSet::new(),
            did_initialize_vcs_file_set: false,
        })
    }

    // Populate `committed_files` with the results of `git ls-files`.
    pub fn initialize_vcs_file_set(&mut self) -> Result<(), failure::Error> {
        if !self.did_initialize_vcs_file_set {
            let git_scm_provider = GitScmProvider::new(&self.root)?;
            git_scm_provider.check_repo_is_pristine()?;
            self.vcs_file_set = git_scm_provider.get_files()?;
            self.did_initialize_vcs_file_set = true;
        }
        Ok(())
    }

    pub fn add_committed(
        &mut self,
        file_set: &mut FileSet,
        glob: &str,
    ) -> Result<(), failure::Error> {
        self.initialize_vcs_file_set()?;
        let cglob = CompiledGlob::new(glob)?;
        let mut did_match_ignored = false;
        let mut did_match_non_ignored = false;
        let mut changed = Vec::<String>::new();
        let mut untracked = Vec::<String>::new();
        for (file, status) in &self.vcs_file_set {
            if self.pattern_matches_path_or_ancestor(&cglob, file) {
                match status {
                    VCSFileStatus::Tracked => {
                        file_set.insert(file.clone());
                        did_match_non_ignored = true;
                    }
                    VCSFileStatus::Changed => {
                        changed.push(file.clone());
                        did_match_non_ignored = true;
                    }
                    VCSFileStatus::Untracked => {
                        untracked.push(file.clone());
                        did_match_non_ignored = true;
                    }
                    VCSFileStatus::Ignored => {
                        did_match_ignored = true;
                    }
                }
            }
        }
        if !did_match_non_ignored {
            if did_match_ignored {
                Err(failure::Error::from(FileSetError::IgnoredOnly))
            } else {
                // We may also want to provide a more helpful error message if a
                // glob only matches case-insensitively.
                Err(failure::Error::from(FileSetError::NotFound))
            }
        } else if !changed.is_empty() || !untracked.is_empty() {
            Err(failure::Error::from(FileSetError::NotCommitted {
                changed,
                untracked,
            }))
        } else {
            Ok(())
        }
    }

    pub fn add_any(&mut self, file_set: &mut FileSet, glob: &str) -> Result<(), failure::Error> {
        // This relies on Git to give us a list of all the files, including the
        // untracked ones. This works fine as long as we are in a Git repo, but
        // we really should not depend on Git here.
        self.initialize_vcs_file_set()?;
        let cglob = CompiledGlob::new(glob)?;
        let mut did_match = false;
        for file in self.vcs_file_set.keys() {
            if self.pattern_matches_path_or_ancestor(&cglob, file) {
                did_match = true;
                // We add files regardless of their VCS status here. One minor
                // deficiency is that files that have been deleted but whose
                // deletion hasn't been commmitted will end up in the file set,
                // subsequently triggering an IO error.
                file_set.insert(file.clone());
            }
        }

        if !did_match {
            Err(failure::Error::from(FileSetError::NotFound))
        } else {
            Ok(())
        }
    }

    pub fn remove(&mut self, file_set: &mut FileSet, glob: &str) -> Result<(), failure::Error> {
        let cglob = CompiledGlob::new(glob)?;
        file_set.retain(|file| !self.pattern_matches_path_or_ancestor(&cglob, file));

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
            } else if p.ends_with('/') {
                p = &p[..(p.len() - 1)];
            } else {
                match p.rfind('/') {
                    Some(i) => p = &p[..=i],
                    None => {
                        break;
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

    fn cause(&self) -> Option<&dyn StdError> {
        match self {
            GlobError::PatternError(ref pattern_error) => Some(pattern_error),
            _ => None,
        }
    }
}

impl fmt::Display for GlobError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

#[derive(Debug)]
enum CompiledGlob {
    Root,
    NonRoot(Pattern),
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
        if glob.starts_with('/') || (glob.len() >= 2 && glob.as_bytes()[1] == b':') {
            return Err(GlobError::Absolute);
        }

        if glob == "." || glob == "./" {
            Ok(CompiledGlob::Root)
        } else {
            let pattern = Pattern::new(glob).map_err(GlobError::PatternError)?;
            Ok(CompiledGlob::NonRoot(pattern))
        }
    }

    // The reason why we don't have a single "matches" function is that the glob
    // library thinks that ".*" matches ".", and "*" matches "". So we treat the
    // root case separately.
    pub fn matches_non_root(&self, p: &str) -> bool {
        match self {
            CompiledGlob::Root => false,
            CompiledGlob::NonRoot(pattern) => pattern.matches_with(p, MATCH_OPTIONS),
        }
    }

    pub fn matches_root(&self) -> bool {
        match self {
            CompiledGlob::Root => true,
            CompiledGlob::NonRoot(_) => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn make_fsi(files: &[&str]) -> FilesSectionInterpreter {
        let mut vcs_file_set = VCSFileSet::new();
        for file in files {
            vcs_file_set.insert((*file).to_string(), VCSFileStatus::Tracked);
        }
        FilesSectionInterpreter {
            root: PathBuf::from("dummy"),
            vcs_file_set,
            did_initialize_vcs_file_set: true,
        }
    }

    fn assert_file_set(file_set: &FileSet, files: &[&str]) {
        let mut file_set_v: Vec<String> = file_set.clone().into_iter().collect();
        file_set_v.sort();
        assert_eq!(
            file_set_v,
            files
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
        );
    }

    mod process_glob {
        use super::*;

        fn glob_error_for(glob: &str) -> GlobError {
            unwrap_glob_error(make_fsi(&[]).add_any(&mut FileSet::new(), glob))
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
            fsi.add_any(&mut file_set, "src/**").unwrap();
            fsi.remove(&mut file_set, "src/vendor/*").unwrap();
            fsi.add_any(&mut file_set, "src/vendor/a.rs").unwrap();

            assert_file_set(&file_set, &["src/a.rs", "src/b.rs", "src/vendor/a.rs"]);
        }

        #[test]
        fn include_directory() {
            let mut fsi = make_fsi(&["a.rs", "src/b.rs", "src/vendor/c.rs"]);
            let mut file_set = FileSet::new();
            fsi.add_any(&mut file_set, "src").unwrap();
            assert_file_set(&file_set, &["src/b.rs", "src/vendor/c.rs"]);

            let mut file_set2 = FileSet::new();
            fsi.add_any(&mut file_set2, "src/").unwrap();
            assert_file_set(&file_set2, &["src/b.rs", "src/vendor/c.rs"]);
        }

        #[test]
        fn exclude_directory() {
            let mut fsi = make_fsi(&["src/a.rs"]);
            let mut file_set = FileSet::new();
            fsi.add_any(&mut file_set, "**").unwrap();
            fsi.remove(&mut file_set, "src").unwrap();
            assert_file_set(&file_set, &[]);
        }

        #[test]
        fn exclude_directory_trailing_slash() {
            let mut fsi = make_fsi(&["src/a.rs"]);
            let mut file_set = FileSet::new();
            fsi.add_any(&mut file_set, "**").unwrap();
            fsi.remove(&mut file_set, "src/").unwrap();
            assert_file_set(&file_set, &[]);
        }
    }
}
