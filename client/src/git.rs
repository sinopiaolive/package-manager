#![allow(dead_code)]

use std::path::{Path,PathBuf};
use std::fs::canonicalize;
use git2::{Repository,Tree};
use git2;
use error::Error;

pub struct GitScmProvider {
    pub relative_package_root: PathBuf, // relative to repo.workdir()
    pub repo: Repository,
}

impl GitScmProvider {
    pub fn new(package_root: &Path) -> Result<Self, Error> {
        let absolute_package_root = canonicalize(package_root)?;
        let repo = Repository::discover(&absolute_package_root)?;
        let repo_workdir = match repo.workdir() {
            None => return Err(Error::Message("Failed to get base directory of Git repository.".to_string())),
            Some(p) => p,
        }.to_path_buf();
        let relative_package_root
            = match absolute_package_root.strip_prefix(&repo_workdir) {
                Ok(p) => p,
                Err(_) => return Err(Error::Message(format!(
                        "Git repository root {} is not a parent directory of {}",
                        repo_workdir.display(), absolute_package_root.display())))
            };
        Ok(GitScmProvider {
            relative_package_root: relative_package_root.to_path_buf(),
            repo: repo,
        })
    }

    pub fn ls_files(&self) -> Result<Vec<String>, Error> {
        let head_sha = self.repo.refname_to_id("HEAD")?;
        let head = self.repo.find_commit(head_sha)?;
        let tree = head.tree()?;
        let mut files = self.ls_files_inner(tree, "")?;
        files.sort();
        Ok(files)
    }

    fn ls_files_inner(&self, tree: Tree, prefix: &str) -> Result<Vec<String>, Error> {
        // TODO: Exclude (or throw error on) submodules
        let mut files = Vec::new();
        for entry in tree.iter() {
            let name = match entry.name() {
                None => return Err(Error::Message("File name returned by Git is not valid UTF-8".to_string())),
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
                        Err(_) => return Err(Error::Message("Git returned an unexpected object type".to_string()))
                    };
                    files.extend(self.ls_files_inner(subtree, &(relative_path + "/"))?);
                }
                _ => return Err(Error::Message("Git return an unexpected object type".to_string()))
            }
        }
        Ok(files)
    }
}

pub fn test_git() {
    println!("{:?}", GitScmProvider::new(&Path::new(".")).unwrap().ls_files());
}
