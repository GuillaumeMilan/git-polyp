use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::ResultExt;
use crate::client;
use crate::io;
use crate::io::Decorate;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StackEntry {
    commit: String,
    branches: Vec<String>,
    message: String,
}

impl StackEntry {
    pub fn new(commit: String, branches: Vec<String>, message: String) -> Self {
        StackEntry {
            commit,
            branches,
            message,
        }
    }

    pub fn format(&self) -> String {
        let short_commit = self.commit.chars().take(8).collect::<String>().red();
        let branches = if self.branches.len() == 0 {
            "".to_string()
        } else {
            format!("({})", self.branches.join(", "))
        };
        format!(
            "* {} - {} {}",
            short_commit,
            branches.yellow(),
            self.message
        )
    }
}

pub enum StackError {
    SerializationError,
    CannotFindPolypDir,
    CannotWriteToFile,
    CannotReadFromFile,
    CannotApplyStack,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stack {
    pub destination_ref: String,
    entries: Vec<StackEntry>,
}

const STACK_FILE: &str = "rebase_stack.json";

impl Stack {
    pub fn new(
        base_ref: &str,
        top_ref: &str,
        destination_ref: &str,
        verbose: &bool,
    ) -> Result<Self, client::ClientError> {
        let entries = build_stack(base_ref, top_ref, verbose)?;
        Ok(Stack {
            destination_ref: destination_ref.to_string(),
            entries,
        })
    }

    pub fn format(&self) -> String {
        self.entries
            .iter()
            .rev()
            .map(|entry| entry.format())
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn format_with_title(&self, title: &str) -> String {
        format!("{}\n\n{}", title.bold().underline(), self.format())
    }

    pub fn exists(verbose: &bool) -> Result<bool, StackError> {
        io::explain(
            verbose,
            "Finding the git-polyp directory to check if a rebase stack file exists.",
        );
        let polyp_dir = client::polyp_dir(verbose).map_err(|_| StackError::CannotFindPolypDir)?;
        let stack_path = format!("{}/{}", polyp_dir, STACK_FILE);
        io::explain(
            verbose,
            &format!(
                "Checking if a rebase stack already exists by reading the cached stack file at {}.",
                stack_path
            ),
        );
        let stack_exists = std::path::Path::new(&stack_path).exists();
        if stack_exists {
            io::explain(verbose, "A rebase stack file already exists.");
        } else {
            io::explain(verbose, "No rebase stack file found.");
        }
        Ok(stack_exists)
    }

    pub fn persist(&self, verbose: &bool) -> Result<(), StackError> {
        let stack_data =
            serde_json::to_string(&self).map_err(|_| StackError::SerializationError)?;
        io::explain(
            verbose,
            "Finding the git-polyp directory to persist the rebase stack.",
        );
        let polyp_dir = client::polyp_dir(verbose).map_err(|_| StackError::CannotFindPolypDir)?;
        let stack_path = format!("{}/{}", polyp_dir, STACK_FILE);
        io::explain(
            verbose,
            &format!(
                "Persisting the rebase stack to the cached stack file at {}.",
                stack_path
            ),
        );
        fs::write(stack_path, stack_data).map_err(|_| StackError::CannotWriteToFile)?;
        io::explain(verbose, "Rebase stack persisted successfully.");
        return Ok(());
    }

    pub fn load(verbose: &bool) -> Result<Self, StackError> {
        io::explain(
            verbose,
            "Finding the git-polyp directory to load the rebase stack.",
        );
        let polyp_dir = client::polyp_dir(verbose).map_err(|_| StackError::CannotFindPolypDir)?;
        let stack_path = format!("{}/{}", polyp_dir, STACK_FILE);
        io::explain(
            verbose,
            &format!(
                "Loading the rebase stack from the cached stack file at {}.",
                stack_path
            ),
        );
        let stack_data =
            fs::read_to_string(stack_path).map_err(|_| StackError::CannotReadFromFile)?;
        let stack: Stack =
            serde_json::from_str(&stack_data).map_err(|_| StackError::SerializationError)?;
        Ok(stack)
    }

    pub fn clean(verbose: &bool) -> Result<(), StackError> {
        io::explain(
            verbose,
            "Finding the git-polyp directory to clean any existing rebase stack.",
        );
        let polyp_dir = client::polyp_dir(verbose).map_err(|_| StackError::CannotFindPolypDir)?;
        let stack_path = format!("{}/{}", polyp_dir, STACK_FILE);
        io::explain(
            verbose,
            &format!(
                "Cleaning the rebase stack by removing the cached stack file at {}.",
                stack_path
            ),
        );
        if std::path::Path::new(&stack_path).exists() {
            match fs::remove_file(stack_path) {
                Ok(()) => Ok(()),
                Err(err) => match err.kind() {
                    std::io::ErrorKind::NotFound => Ok(()),
                    _ => Err(StackError::CannotWriteToFile),
                },
            }
        } else {
            Ok(())
        }
    }

    pub fn apply(&self, verbose: &bool) -> Result<(), StackError> {
        io::explain(
            verbose,
            "Applying the rebase stack by moving the branches to their corresponding commits in the provided stack.",
        );
        for entry in self.entries.iter() {
            for branch in entry.branches.iter() {
                client::move_branche_at(&entry.commit, &branch, verbose)
                    .map_err(|_| StackError::CannotApplyStack)?;
            }
        }
        io::explain(verbose, "Rebase stack applied successfully.");
        Ok(())
    }

    pub fn apply_branches_from(&self, other: &Stack, verbose: &bool) -> Result<Stack, StackError> {
        io::explain(
            verbose,
            "Applying the branches from the provided stack to the current stack by matching the commit messages of their entries.",
        );
        let mut new_stack = self.clone();
        // Create a dictionnary mapping commit messages to branches for the other stack
        let branches_message_map = other
            .entries
            .iter()
            .map(|entry| (entry.message.clone(), entry.branches.clone()))
            .collect::<std::collections::HashMap<String, Vec<String>>>();

        for entry in new_stack.entries.iter_mut() {
            match branches_message_map.get(&entry.message) {
                Some(branches) => entry.branches = branches.clone(),
                None => entry.branches = Vec::new(),
            }
        }
        Ok(new_stack)
    }

    pub fn base_ref(&self) -> &str {
        self.entries
            .first()
            .map(|entry| &entry.commit)
            .expect("Stack should have at least one entry")
    }

    pub fn top_ref(&self) -> &str {
        self.entries
            .last()
            .map(|entry| &entry.commit)
            .expect("Stack should have at least one entry")
    }

    pub fn top_branch(&self) -> Option<String> {
        self.entries
            .last()
            .and_then(|entry| entry.branches.first().cloned())
    }

    pub fn branches(&self) -> Vec<String> {
        self.entries
            .iter()
            .flat_map(|entry| entry.branches.clone())
            .collect()
    }
}

fn build_stack(
    upstream: &str,
    branch: &str,
    verbose: &bool,
) -> Result<Vec<StackEntry>, client::ClientError> {
    let explain_message = format!(
        "Building the stack of commits to rebase from `{}` to `{}`",
        upstream.deco_as_command(),
        branch.deco_as_command()
    );
    io::explain(verbose, &explain_message);
    match client::rev_list(upstream, branch, verbose) {
        Ok(commits) => {
            let mut stack = Vec::new();
            for commit in commits {
                let branches = client::branches_at(&commit, verbose)
                    .unwrap_or_exit("Failed to get branches containing the commit.");
                let message = client::commit_message(&commit, verbose)
                    .unwrap_or_exit("Failed to get the commit message.");
                stack.push(StackEntry::new(commit, branches, message));
            }
            io::explain(
                verbose,
                &format!("Stack built with {} commits.", stack.len()),
            );
            Ok(stack)
        }
        Err(e) => Err(e),
    }
}
