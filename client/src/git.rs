#![allow(dead_code)]

use failure;
use git2;
use git2::{Repository, RepositoryState, StatusOptions};
use std::fs::canonicalize;
use std::path::{Path, PathBuf};

use files::{VCSFileSet, VCSFileStatus};

pub struct GitScmProvider {
    pub relative_package_root: PathBuf, // relative to repo.workdir()
    pub repo: Repository,
}

impl GitScmProvider {
    pub fn new(package_root: &Path) -> Result<Self, failure::Error> {
        let absolute_package_root = canonicalize(package_root)?;
        let repo = Repository::discover(&absolute_package_root)?;
        let repo_workdir = match repo.workdir() {
            None => return Err(failure::Error::from(GitError::BaseDir)),
            Some(p) => p,
        }
        .to_path_buf();
        let relative_package_root = match absolute_package_root.strip_prefix(&repo_workdir) {
            Ok(p) => p,
            Err(_) => {
                return Err(failure::Error::from(GitError::RepoNotParent(
                    repo_workdir.to_string_lossy().to_string(),
                    absolute_package_root.to_string_lossy().to_string(),
                )))
            }
        };
        Ok(GitScmProvider {
            relative_package_root: relative_package_root.to_path_buf(),
            repo,
        })
    }

    pub fn check_repo_is_pristine(&self) -> Result<(), failure::Error> {
        self.check_state_is_clean()?;
        self.check_no_submodules()?;
        Ok(())
    }

    // Check that no merge (or similar) is in progress.
    pub fn check_state_is_clean(&self) -> Result<(), failure::Error> {
        if self.repo.state() == RepositoryState::Clean {
            Ok(())
        } else {
            Err(failure::Error::from(GitError::NotCleanState))
        }
    }

    pub fn check_no_submodules(&self) -> Result<(), failure::Error> {
        // We should restrict this check to submodules living under
        // self.relative_package_root.
        if self.repo.submodules()?.is_empty() {
            Ok(())
        } else {
            Err(failure::Error::from(GitError::SubmodulesPresent))
        }
    }

    pub fn get_files(&self) -> Result<VCSFileSet, failure::Error> {
        let mut status_options = StatusOptions::new();
        status_options
            .include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_unmodified(true)
            .include_ignored(true)
            .recurse_ignored_dirs(true)
            // The name "include_unreadable" would suggest that we will get the
            // WT_UNREADABLE status flag on files without read permissions, but
            // cursory testing on macOS doesn't reveal any difference with
            // regard to status flags: libgit2 still treats unreadable
            // directories as empty and reports unreadable files as WT_NEW (if
            // untracked) or returns an IO error (if tracked and mtime has
            // changed).
            .include_unreadable(true)
            .include_unreadable_as_untracked(false)
            .exclude_submodules(false)
            // Disable rename detection so we get both source and target in the list
            .renames_head_to_index(false)
            .renames_index_to_workdir(false);
        if !self.relative_package_root.as_os_str().is_empty() {
            status_options
                .pathspec(&self.relative_package_root)
                .disable_pathspec_match(true); // don't glob
        }
        let statuses = self.repo.statuses(Some(&mut status_options))?;
        let mut vcs_file_set = VCSFileSet::new();
        for status_entry in statuses.iter() {
            let file_path = status_entry
                .path()
                .ok_or_else(|| failure::Error::from(GitError::Utf8))?;
            let status = if status_entry.status().is_empty() {
                VCSFileStatus::Tracked
            } else if status_entry.status() == git2::Status::IGNORED {
                // It's unclear if the IGNORED status flag can ever co-occur
                // with other flags. Calling .ignored() would allow other flags
                // to be set, which we would not want.
                VCSFileStatus::Ignored
            } else if status_entry.status() == git2::Status::WT_NEW {
                // We don't use .wt_new() because we only want to list files
                // that do not have other flags set. E.g. `git rm tracked_file;
                // touch tracked_file` causes tracked_file to have status
                // `INDEX_DELETED | WT_NEW`. We would not consider this file
                // untracked.
                VCSFileStatus::Untracked
            } else {
                // This subsumes every other flag combination.
                VCSFileStatus::Changed
            };
            vcs_file_set.insert(file_path.to_string(), status);
        }
        Ok(vcs_file_set)
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum GitError {
        Utf8 {
            description("Git encountered a filename that is not valid UTF-8")
        }
        BaseDir {
            description("Failed to get base directory of Git repository.")
        }
        RepoNotParent(repo_root: String, package_root: String) {
            display("Git found a repo at {}, which is not a parent directory of {}", repo_root, package_root)
        }

        NotCleanState {
            description("A Git operation such as a merge or a rebase is in progress in your working tree")
        }
        SubmodulesPresent {
            description("Git repositories with submodules are currently unsupported")
        }
    }
}

pub fn test_git() {
    GitScmProvider::new(&Path::new("."))
        .unwrap()
        .get_files()
        .unwrap();
}
