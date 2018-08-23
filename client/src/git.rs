#![allow(dead_code)]

use std::path::{Path,PathBuf};
use std::fs::canonicalize;
use failure;
use git2::{Repository,RepositoryState,StatusOptions,Tree};
use git2;

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
        }.to_path_buf();
        let relative_package_root
            = match absolute_package_root.strip_prefix(&repo_workdir) {
                Ok(p) => p,
                Err(_) => return Err(failure::Error::from(GitError::RepoNotParent(
                    repo_workdir.to_string_lossy().to_string(), absolute_package_root.to_string_lossy().to_string())))
            };
        Ok(GitScmProvider {
            relative_package_root: relative_package_root.to_path_buf(),
            repo: repo,
        })
    }

    pub fn check_repo_is_pristine(&self) -> Result<(), failure::Error> {
        self.check_state_is_clean()?;
        self.check_no_submodules()?;
        self.check_status_is_empty()?;
        Ok(())
    }

    pub fn check_state_is_clean(&self) -> Result<(), failure::Error> {
        if self.repo.state() == RepositoryState::Clean {
            Ok(())
        } else {
            Err(failure::Error::from(GitError::NotCleanState))
        }
    }

    pub fn check_status_is_empty(&self) -> Result<(), failure::Error> {
        let mut status_options = StatusOptions::new();
        status_options
            .include_untracked(true)
            .exclude_submodules(true)
            .pathspec(&self.relative_package_root)
            .disable_pathspec_match(true);
        let statuses = self.repo.statuses(Some(&mut status_options))?;
        if statuses.is_empty() {
            Ok(())
        } else {
            Err(failure::Error::from(GitError::NonEmptyStatus))
        }
    }

    pub fn check_no_submodules(&self) -> Result<(), failure::Error> {
        if self.repo.submodules()?.is_empty() {
            Ok(())
        } else {
            Err(failure::Error::from(GitError::SubmodulesPresent))
        }
    }

    pub fn ls_files(&self) -> Result<Vec<String>, failure::Error> {
        let head_sha = self.repo.refname_to_id("HEAD")?;
        let head = self.repo.find_commit(head_sha)?;
        let mut tree = head.tree()?;
        for component in self.relative_package_root.iter() {
            let component_str = match component.to_str() {
                Some(s) => s,
                None => return Err(failure::Error::from(GitError::Utf8))
            };
            let subtree = match tree.get_name(component_str) {
                None => return Ok(vec![]),
                Some(tree_entry) => {
                    match tree_entry.to_object(&self.repo)?.into_tree() {
                        Err(_object) => return Ok(vec![]),
                        Ok(subtree) => subtree
                    }
                }
            };
            tree = subtree;
        }
        let mut files = self.ls_files_inner(tree, "")?;
        files.sort();
        Ok(files)
    }

    fn ls_files_inner(&self, tree: Tree, prefix: &str) -> Result<Vec<String>, failure::Error> {
        let mut files = Vec::new();
        for entry in tree.iter() {
            let name = match entry.name() {
                None => return Err(failure::Error::from(GitError::Utf8)),
                Some(n) => n
            };
            let relative_path = prefix.to_string() + name;
            match entry.kind() {
                Some(git2::ObjectType::Blob) => {
                    files.push(relative_path);
                },
                Some(git2::ObjectType::Tree) => {
                    let object = entry.to_object(&self.repo)?;
                    let subtree = match object.into_tree() {
                        Ok(t) => t,
                        Err(_) => return Err(failure::Error::from(GitError::ObjectType))
                    };
                    files.extend(self.ls_files_inner(subtree, &(relative_path + "/"))?);
                }
                _ => return Err(failure::Error::from(GitError::ObjectType))
            }
        }
        Ok(files)
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
        ObjectType {
            description("Git returned an unexpected object type")
        }

        NotCleanState {
            description("A Git operation such as a merge or a rebase is in progress in your working tree")
        }
        NonEmptyStatus {
            description("The command `git status` shows modified or untracked files. Commit them to Git or add them to .gitignore before running this.")
        }
        SubmodulesPresent {
            description("Git repositories with submodules are currently unsupported")
        }
    }
}


pub fn test_git() {
    println!("{:?}", GitScmProvider::new(&Path::new(".")).unwrap().ls_files());
}
