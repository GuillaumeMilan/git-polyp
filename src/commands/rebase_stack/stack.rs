use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::ResultExt;
use crate::client;

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
    destination_ref: String,
    entries: Vec<StackEntry>,
}

const STACK_FILE: &str = "rebase_stack.json";

impl Stack {
    pub fn new(
        base_ref: &str,
        top_ref: &str,
        destination_ref: &str,
    ) -> Result<Self, client::ClientError> {
        let entries = build_stack(base_ref, top_ref)?;
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

    pub fn exists() -> Result<bool, StackError> {
        let polyp_dir = client::polyp_dir().map_err(|_| StackError::CannotFindPolypDir)?;
        let stack_path = format!("{}/{}", polyp_dir, STACK_FILE);
        Ok(std::path::Path::new(&stack_path).exists())
    }

    pub fn persist(&self) -> Result<(), StackError> {
        let stack_data =
            serde_json::to_string(&self).map_err(|_| StackError::SerializationError)?;
        let polyp_dir = client::polyp_dir().map_err(|_| StackError::CannotFindPolypDir)?;
        let stack_path = format!("{}/{}", polyp_dir, STACK_FILE);
        fs::write(stack_path, stack_data).map_err(|_| StackError::CannotWriteToFile)?;
        return Ok(());
    }

    pub fn load() -> Result<Self, StackError> {
        let polyp_dir = client::polyp_dir().map_err(|_| StackError::CannotFindPolypDir)?;
        let stack_path = format!("{}/{}", polyp_dir, STACK_FILE);
        let stack_data =
            fs::read_to_string(stack_path).map_err(|_| StackError::CannotReadFromFile)?;
        let stack: Stack =
            serde_json::from_str(&stack_data).map_err(|_| StackError::SerializationError)?;
        Ok(stack)
    }

    pub fn clean() -> Result<(), StackError> {
        let polyp_dir = client::polyp_dir().map_err(|_| StackError::CannotFindPolypDir)?;
        let stack_path = format!("{}/{}", polyp_dir, STACK_FILE);
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

    pub fn apply(&self) -> Result<(), StackError> {
        for entry in self.entries.iter() {
            for branch in entry.branches.iter() {
                client::move_branche_at(&entry.commit, &branch)
                    .map_err(|_| StackError::CannotApplyStack)?;
            }
        }
        Ok(())
    }

    pub fn apply_branches_from(&self, other: &Stack) -> Result<Stack, StackError> {
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

    pub fn branches(&self) -> Vec<String> {
        self.entries
            .iter()
            .flat_map(|entry| entry.branches.clone())
            .collect()
    }
}

fn build_stack(upstream: &str, branch: &str) -> Result<Vec<StackEntry>, client::ClientError> {
    match client::rev_list(upstream, branch) {
        Ok(commits) => {
            let mut stack = Vec::new();
            for commit in commits {
                let branches = client::branches_at(&commit)
                    .unwrap_or_exit("Failed to get branches containing the commit.");
                let message = client::commit_message(&commit)
                    .unwrap_or_exit("Failed to get the commit message.");
                stack.push(StackEntry::new(commit, branches, message));
            }
            Ok(stack)
        }
        Err(e) => Err(e),
    }
}
