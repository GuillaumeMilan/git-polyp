use crate::io;
use std::os::unix::process::ExitStatusExt;

struct GitCommand {
    args: Vec<String>,
    verbose: bool,
}

#[derive(Debug)]
pub enum ClientError {
    Command,
    NonZeroExitCode,
    InvalidUtf8,
    CanNotCreatePolypDir,
}

impl GitCommand {
    fn new(args: Vec<String>, verbose: &bool) -> Self {
        Self {
            args,
            verbose: verbose.clone(),
        }
    }

    fn execute(&self) -> Result<String, ClientError> {
        print_command(&self.args, &self.verbose);
        let output = match std::process::Command::new("git").args(&self.args).output() {
            Ok(output) => output,
            Err(_) => {
                return Err(ClientError::Command);
            }
        };

        if output.status.into_raw() != 0 {
            return Err(ClientError::NonZeroExitCode);
        }

        match String::from_utf8(output.stdout) {
            Ok(result) => Ok(result.trim().to_string()),
            Err(_) => {
                return Err(ClientError::InvalidUtf8);
            }
        }
    }
}

pub fn merge_base(upstream: &str, branch: &str, verbose: &bool) -> Result<String, ClientError> {
    GitCommand::new(
        vec![
            "merge-base".to_string(),
            upstream.to_string(),
            branch.to_string(),
        ],
        verbose,
    )
    .execute()
}

pub fn dot_git_dir(verbose: &bool) -> Result<String, ClientError> {
    GitCommand::new(
        vec!["rev-parse".to_string(), "--git-dir".to_string()],
        verbose,
    )
    .execute()
}

pub fn polyp_dir(verbose: &bool) -> Result<String, ClientError> {
    let git_dir = dot_git_dir(verbose)?;
    let polyp_dir = format!("{}/polyp", git_dir);
    if !std::path::Path::new(&polyp_dir).exists() {
        std::fs::create_dir(&polyp_dir).map_err(|_| ClientError::CanNotCreatePolypDir)?;
    };
    Ok(polyp_dir)
}

pub fn current_branch(verbose: &bool) -> Result<String, ClientError> {
    GitCommand::new(
        vec![
            "rev-parse".to_string(),
            "--abbrev-ref".to_string(),
            "HEAD".to_string(),
        ],
        verbose,
    )
    .execute()
}

pub fn rev_parse(rev: &str, verbose: &bool) -> Result<String, ClientError> {
    GitCommand::new(vec!["rev-parse".to_string(), rev.to_string()], verbose).execute()
}

pub fn is_in_repo(verbose: &bool) -> Result<bool, ClientError> {
    let args = vec!["rev-parse".to_string()];
    print_command(&args, verbose);
    let output = match std::process::Command::new("git").args(&args).output() {
        Ok(output) => output,
        Err(_) => {
            return Err(ClientError::Command);
        }
    };

    if output.status.into_raw() != 0 {
        return Ok(false);
    }
    return Ok(true);
}

pub fn rev_list(upstream: &str, branch: &str, verbose: &bool) -> Result<Vec<String>, ClientError> {
    let output = GitCommand::new(
        vec![
            "rev-list".to_string(),
            "--reverse".to_string(),
            format!("{}..{}", upstream, branch),
        ],
        verbose,
    )
    .execute()?;

    Ok(output.lines().map(|line| line.to_string()).collect())
}

pub fn commit_message(commit_hash: &str, verbose: &bool) -> Result<String, ClientError> {
    GitCommand::new(
        vec![
            "log".to_string(),
            "-1".to_string(),
            "--format=%s".to_string(),
            commit_hash.to_string(),
        ],
        verbose,
    )
    .execute()
}

pub fn branches_at(commit_hash: &str, verbose: &bool) -> Result<Vec<String>, ClientError> {
    let output = GitCommand::new(
        vec![
            "branch".to_string(),
            "--points-at".to_string(),
            commit_hash.to_string(),
            "--format=%(refname:short)".to_string(),
        ],
        verbose,
    )
    .execute()?;

    Ok(output.lines().map(|line| line.trim().to_string()).collect())
}

pub fn switch_d(revision: &str, verbose: &bool) -> Result<(), ClientError> {
    GitCommand::new(
        vec![
            "switch".to_string(),
            "--detach".to_string(),
            revision.to_string(),
        ],
        verbose,
    )
    .execute()
    .map(|_| ())
}

pub fn switch(branch: &str, verbose: &bool) -> Result<(), ClientError> {
    GitCommand::new(vec!["switch".to_string(), branch.to_string()], verbose)
        .execute()
        .map(|_| ())
}

pub fn cherry_pick(commit_a: &str, commit_b: &str, verbose: &bool) -> Result<(), ClientError> {
    let args = vec![
        "cherry-pick".to_string(),
        format!("{}^..{}", commit_a, commit_b),
    ];
    GitCommand::new(args, verbose).execute().map(|_| ())
}

pub fn cherry_pick_continue(verbose: &bool) -> Result<(), ClientError> {
    GitCommand::new(
        vec!["cherry-pick".to_string(), "--continue".to_string()],
        verbose,
    )
    .execute()
    .map(|_| ())
}

pub fn move_branche_at(commit_hash: &str, branch: &str, verbose: &bool) -> Result<(), ClientError> {
    GitCommand::new(
        vec![
            "branch".to_string(),
            "--force".to_string(),
            branch.to_string(),
            commit_hash.to_string(),
        ],
        verbose,
    )
    .execute()
    .map(|_| ())
}

pub fn push_branches(
    remote: &str,
    branches: Vec<String>,
    verbose: &bool,
) -> Result<(), ClientError> {
    let mut args = vec![
        "push".to_string(),
        "--force-with-lease".to_string(),
        remote.to_string(),
    ];

    for branch in branches {
        args.push(branch);
    }
    GitCommand::new(args, verbose).execute().map(|_| ())
}

fn print_command(args: &Vec<String>, verbose: &bool) {
    io::execute(verbose, &format!("git {}", args.join(" ")));
}

// ============================================================================
// Worktree-related functions
// ============================================================================

/// Information about a single git worktree
#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    pub path: String,
    pub head: String,
    pub branch: Option<String>,
    pub is_bare: bool,
}

/// Clone a repository as a bare repo
pub fn clone_bare(url: &str, dest: &str, verbose: &bool) -> Result<(), ClientError> {
    GitCommand::new(
        vec![
            "clone".to_string(),
            "--bare".to_string(),
            url.to_string(),
            dest.to_string(),
        ],
        verbose,
    )
    .execute()
    .map(|_| ())
}

/// List all worktrees in the repository
pub fn worktree_list(verbose: &bool) -> Result<Vec<WorktreeInfo>, ClientError> {
    let output = GitCommand::new(
        vec![
            "worktree".to_string(),
            "list".to_string(),
            "--porcelain".to_string(),
        ],
        verbose,
    )
    .execute()?;

    let mut worktrees = Vec::new();
    let mut current_path: Option<String> = None;
    let mut current_head: Option<String> = None;
    let mut current_branch: Option<String> = None;
    let mut is_bare = false;

    for line in output.lines() {
        if line.starts_with("worktree ") {
            // Save previous worktree if exists
            if let (Some(path), Some(head)) = (current_path.take(), current_head.take()) {
                worktrees.push(WorktreeInfo {
                    path,
                    head,
                    branch: current_branch.take(),
                    is_bare,
                });
                is_bare = false;
            }
            current_path = Some(line.strip_prefix("worktree ").unwrap_or("").to_string());
        } else if line.starts_with("HEAD ") {
            current_head = Some(line.strip_prefix("HEAD ").unwrap_or("").to_string());
        } else if line.starts_with("branch ") {
            let branch_ref = line.strip_prefix("branch ").unwrap_or("");
            // Convert refs/heads/branch to just branch
            current_branch = Some(
                branch_ref
                    .strip_prefix("refs/heads/")
                    .unwrap_or(branch_ref)
                    .to_string(),
            );
        } else if line == "bare" {
            is_bare = true;
        }
    }

    // Don't forget the last worktree
    if let (Some(path), Some(head)) = (current_path, current_head) {
        worktrees.push(WorktreeInfo {
            path,
            head,
            branch: current_branch,
            is_bare,
        });
    }

    Ok(worktrees)
}

/// Add a new worktree
pub fn worktree_add(path: &str, branch: &str, verbose: &bool) -> Result<(), ClientError> {
    GitCommand::new(
        vec![
            "worktree".to_string(),
            "add".to_string(),
            path.to_string(),
            branch.to_string(),
        ],
        verbose,
    )
    .execute()
    .map(|_| ())
}

/// Add a new worktree with a new branch from a base ref
pub fn worktree_add_new_branch(
    path: &str,
    branch: &str,
    base: &str,
    verbose: &bool,
) -> Result<(), ClientError> {
    GitCommand::new(
        vec![
            "worktree".to_string(),
            "add".to_string(),
            "-b".to_string(),
            branch.to_string(),
            path.to_string(),
            base.to_string(),
        ],
        verbose,
    )
    .execute()
    .map(|_| ())
}

/// Remove a worktree
pub fn worktree_remove(path: &str, force: bool, verbose: &bool) -> Result<(), ClientError> {
    let mut args = vec!["worktree".to_string(), "remove".to_string()];
    if force {
        args.push("--force".to_string());
    }
    args.push(path.to_string());

    GitCommand::new(args, verbose).execute().map(|_| ())
}

/// Check if a branch exists (locally or remotely)
pub fn branch_exists(branch: &str, verbose: &bool) -> Result<bool, ClientError> {
    // Check local branch
    let local_check = GitCommand::new(
        vec![
            "show-ref".to_string(),
            "--verify".to_string(),
            "--quiet".to_string(),
            format!("refs/heads/{}", branch),
        ],
        verbose,
    )
    .execute();

    if local_check.is_ok() {
        return Ok(true);
    }

    // Check remote branch
    let remote_check = GitCommand::new(
        vec![
            "show-ref".to_string(),
            "--verify".to_string(),
            "--quiet".to_string(),
            format!("refs/remotes/origin/{}", branch),
        ],
        verbose,
    )
    .execute();

    Ok(remote_check.is_ok())
}

/// Create a new branch from a base ref
pub fn create_branch(name: &str, from: &str, verbose: &bool) -> Result<(), ClientError> {
    GitCommand::new(
        vec![
            "branch".to_string(),
            name.to_string(),
            from.to_string(),
        ],
        verbose,
    )
    .execute()
    .map(|_| ())
}

/// Delete a branch
pub fn delete_branch(name: &str, force: bool, verbose: &bool) -> Result<(), ClientError> {
    let flag = if force { "-D" } else { "-d" };
    GitCommand::new(
        vec!["branch".to_string(), flag.to_string(), name.to_string()],
        verbose,
    )
    .execute()
    .map(|_| ())
}

/// Check if a branch is merged into another branch
pub fn is_branch_merged(branch: &str, into: &str, verbose: &bool) -> Result<bool, ClientError> {
    let output = GitCommand::new(
        vec![
            "branch".to_string(),
            "--merged".to_string(),
            into.to_string(),
        ],
        verbose,
    )
    .execute()?;

    Ok(output.lines().any(|line| line.trim().trim_start_matches("* ") == branch))
}

/// Detect the main branch name (main, master, or default)
pub fn detect_main_branch(verbose: &bool) -> Result<String, ClientError> {
    // Try common main branch names in order of preference
    for branch in &["main", "master"] {
        if branch_exists(branch, verbose)? {
            return Ok(branch.to_string());
        }
    }

    // Try to get the default branch from remote
    let output = GitCommand::new(
        vec![
            "symbolic-ref".to_string(),
            "refs/remotes/origin/HEAD".to_string(),
        ],
        verbose,
    )
    .execute();

    if let Ok(ref_output) = output {
        if let Some(branch) = ref_output.strip_prefix("refs/remotes/origin/") {
            return Ok(branch.to_string());
        }
    }

    // Fallback to HEAD
    let head_output = GitCommand::new(
        vec![
            "symbolic-ref".to_string(),
            "--short".to_string(),
            "HEAD".to_string(),
        ],
        verbose,
    )
    .execute()?;

    Ok(head_output)
}

/// Check if a worktree has uncommitted changes
pub fn worktree_is_dirty(path: &str, verbose: &bool) -> Result<bool, ClientError> {
    let output = std::process::Command::new("git")
        .args(&["-C", path, "status", "--porcelain"])
        .output()
        .map_err(|_| ClientError::Command)?;

    if !output.status.success() {
        return Err(ClientError::NonZeroExitCode);
    }

    let stdout = String::from_utf8(output.stdout).map_err(|_| ClientError::InvalidUtf8)?;
    print_command(&vec!["-C".to_string(), path.to_string(), "status".to_string(), "--porcelain".to_string()], verbose);

    Ok(!stdout.trim().is_empty())
}

/// Get ahead/behind count for a branch relative to its upstream
pub fn get_ahead_behind(branch: &str, verbose: &bool) -> Result<(u32, u32), ClientError> {
    let output = GitCommand::new(
        vec![
            "rev-list".to_string(),
            "--left-right".to_string(),
            "--count".to_string(),
            format!("{}...origin/{}", branch, branch),
        ],
        verbose,
    )
    .execute();

    match output {
        Ok(result) => {
            let parts: Vec<&str> = result.split_whitespace().collect();
            if parts.len() == 2 {
                let ahead = parts[0].parse().unwrap_or(0);
                let behind = parts[1].parse().unwrap_or(0);
                Ok((ahead, behind))
            } else {
                Ok((0, 0))
            }
        }
        Err(_) => Ok((0, 0)), // No upstream, return 0,0
    }
}

/// Get the repository root directory
pub fn repo_root(verbose: &bool) -> Result<String, ClientError> {
    GitCommand::new(
        vec![
            "rev-parse".to_string(),
            "--show-toplevel".to_string(),
        ],
        verbose,
    )
    .execute()
}
