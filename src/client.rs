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

/// Check if a branch exists locally (refs/heads only).
///
/// Use `remote_branch_exists` to check the remote, and `fetch_branch`
/// to create a local ref from a remote-only branch.
pub fn branch_exists(branch: &str, verbose: &bool) -> Result<bool, ClientError> {
    let result = GitCommand::new(
        vec![
            "show-ref".to_string(),
            "--verify".to_string(),
            "--quiet".to_string(),
            format!("refs/heads/{}", branch),
        ],
        verbose,
    )
    .execute();

    Ok(result.is_ok())
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
    let output = GitCommand::new(
        vec![
            "-C".to_string(),
            path.to_string(),
            "status".to_string(),
            "--porcelain".to_string(),
        ],
        verbose,
    )
    .execute()?;

    Ok(!output.is_empty())
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

/// Get the current branch of a worktree at a specific path
pub fn worktree_current_branch(path: &str, verbose: &bool) -> Result<Option<String>, ClientError> {
    let output = std::process::Command::new("git")
        .args(&["-C", path, "rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|_| ClientError::Command)?;

    print_command(&vec!["-C".to_string(), path.to_string(), "rev-parse".to_string(), "--abbrev-ref".to_string(), "HEAD".to_string()], verbose);

    if !output.status.success() {
        return Err(ClientError::NonZeroExitCode);
    }

    let branch = String::from_utf8(output.stdout)
        .map_err(|_| ClientError::InvalidUtf8)?
        .trim()
        .to_string();

    // HEAD means detached state
    if branch == "HEAD" {
        return Ok(None);
    }

    Ok(Some(branch))
}

/// Check if a worktree has a merge/rebase/cherry-pick in progress
pub fn worktree_has_conflicts(path: &str, verbose: &bool) -> Result<bool, ClientError> {
    // Check for various conflict states by looking for marker files
    let git_dir_output = std::process::Command::new("git")
        .args(&["-C", path, "rev-parse", "--git-dir"])
        .output()
        .map_err(|_| ClientError::Command)?;

    print_command(&vec!["-C".to_string(), path.to_string(), "rev-parse".to_string(), "--git-dir".to_string()], verbose);

    if !git_dir_output.status.success() {
        return Err(ClientError::NonZeroExitCode);
    }

    let git_dir = String::from_utf8(git_dir_output.stdout)
        .map_err(|_| ClientError::InvalidUtf8)?
        .trim()
        .to_string();

    let git_dir_path = if std::path::Path::new(&git_dir).is_absolute() {
        std::path::PathBuf::from(&git_dir)
    } else {
        std::path::PathBuf::from(path).join(&git_dir)
    };

    // Check for merge, rebase, or cherry-pick in progress
    let merge_head = git_dir_path.join("MERGE_HEAD");
    let rebase_merge = git_dir_path.join("rebase-merge");
    let rebase_apply = git_dir_path.join("rebase-apply");
    let cherry_pick_head = git_dir_path.join("CHERRY_PICK_HEAD");

    Ok(merge_head.exists() || rebase_merge.exists() || rebase_apply.exists() || cherry_pick_head.exists())
}

/// Check if a worktree directory is valid (not broken)
pub fn worktree_is_valid(path: &str, verbose: &bool) -> Result<bool, ClientError> {
    let output = std::process::Command::new("git")
        .args(&["-C", path, "rev-parse", "--is-inside-work-tree"])
        .output()
        .map_err(|_| ClientError::Command)?;

    print_command(&vec!["-C".to_string(), path.to_string(), "rev-parse".to_string(), "--is-inside-work-tree".to_string()], verbose);

    if !output.status.success() {
        return Ok(false);
    }

    let result = String::from_utf8(output.stdout)
        .map_err(|_| ClientError::InvalidUtf8)?
        .trim()
        .to_string();

    Ok(result == "true")
}

/// Check if a branch exists on the remote
pub fn remote_branch_exists(branch: &str, verbose: &bool) -> Result<bool, ClientError> {
    let output = GitCommand::new(
        vec![
            "ls-remote".to_string(),
            "--heads".to_string(),
            "origin".to_string(),
            branch.to_string(),
        ],
        verbose,
    )
    .execute()?;

    Ok(!output.trim().is_empty())
}

/// Fetch a specific branch from origin, creating a local ref for it
pub fn fetch_branch(branch: &str, verbose: &bool) -> Result<(), ClientError> {
    GitCommand::new(
        vec![
            "fetch".to_string(),
            "origin".to_string(),
            format!("refs/heads/{}:refs/heads/{}", branch, branch),
        ],
        verbose,
    )
    .execute()
    .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::{Mutex, atomic::{AtomicU32, Ordering}};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);
    /// Mutex to serialize tests that need to change the process working directory.
    /// Mutex to serialize tests that change the process working directory.
    /// We recover from poison (a prior test panicked while holding the lock).
    static CWD_LOCK: Mutex<()> = Mutex::new(());
    fn lock_cwd() -> std::sync::MutexGuard<'static, ()> {
        CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Helper: create a temporary directory for tests (unique per call)
    fn create_temp_dir(name: &str) -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!("git-polyp-test-{}-{}-{}", name, std::process::id(), id));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    /// Helper: clean up a temporary directory
    fn cleanup_temp_dir(path: &Path) {
        let _ = std::fs::remove_dir_all(path);
    }

    /// Helper: run a git command in a specific directory
    fn git(dir: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .expect("failed to run git");
        assert!(
            output.status.success(),
            "git {} failed in {}: {}",
            args.join(" "),
            dir.display(),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    }

    /// Helper: set up a "remote" repo with an initial commit on main,
    /// and a bare clone pointing to it as origin.
    /// Returns (remote_dir, bare_dir).
    fn setup_remote_and_bare_clone() -> (PathBuf, PathBuf) {
        let remote_dir = create_temp_dir("remote");
        let bare_dir = create_temp_dir("bare");
        // remove bare_dir so git clone can create it
        std::fs::remove_dir_all(&bare_dir).unwrap();

        // Init the "remote" repo
        git(&remote_dir, &["init", "-b", "main"]);
        git(&remote_dir, &["config", "user.email", "test@test.com"]);
        git(&remote_dir, &["config", "user.name", "Test"]);
        std::fs::write(remote_dir.join("file.txt"), "hello").unwrap();
        git(&remote_dir, &["add", "."]);
        git(&remote_dir, &["commit", "-m", "initial"]);

        // Clone as bare
        git(
            &std::env::temp_dir(),
            &["clone", "--bare", remote_dir.to_str().unwrap(), bare_dir.to_str().unwrap()],
        );

        (remote_dir, bare_dir)
    }

    /// Helper: create a branch with a commit on the "remote" repo
    fn create_remote_branch(remote_dir: &Path, branch_name: &str, filename: &str) {
        git(remote_dir, &["checkout", "-b", branch_name]);
        std::fs::write(remote_dir.join(filename), format!("content for {}", branch_name)).unwrap();
        git(remote_dir, &["add", "."]);
        git(remote_dir, &["commit", "-m", &format!("add {}", branch_name)]);
        // Go back to main so subsequent calls don't stack branches
        git(remote_dir, &["checkout", "main"]);
    }

    // =========================================================================
    // branch_exists
    // =========================================================================

    #[test]
    fn test_branch_exists_returns_true_for_local_branch() {
        let _lock = lock_cwd();
        let (remote_dir, bare_dir) = setup_remote_and_bare_clone();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&bare_dir).unwrap();

        let result = branch_exists("main", &false).unwrap();
        assert!(result, "main branch should exist locally in bare clone");

        std::env::set_current_dir(&original_dir).unwrap();
        cleanup_temp_dir(&bare_dir);
        cleanup_temp_dir(&remote_dir);
    }

    #[test]
    fn test_branch_exists_returns_false_for_unknown_branch() {
        let _lock = lock_cwd();
        let (remote_dir, bare_dir) = setup_remote_and_bare_clone();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&bare_dir).unwrap();

        let result = branch_exists("does-not-exist", &false).unwrap();
        assert!(!result, "non-existent branch should not be found");

        std::env::set_current_dir(&original_dir).unwrap();
        cleanup_temp_dir(&bare_dir);
        cleanup_temp_dir(&remote_dir);
    }

    // =========================================================================
    // remote_branch_exists
    // =========================================================================

    #[test]
    fn test_remote_branch_exists_finds_branch_pushed_after_clone() {
        let _lock = lock_cwd();
        let (remote_dir, bare_dir) = setup_remote_and_bare_clone();
        let original_dir = std::env::current_dir().unwrap();

        // Create a new branch on the remote AFTER the bare clone
        create_remote_branch(&remote_dir, "feature-remote", "feature.txt");

        std::env::set_current_dir(&bare_dir).unwrap();

        // branch_exists should NOT find it (no local ref yet)
        let local = branch_exists("feature-remote", &false).unwrap();
        assert!(!local, "branch should not exist locally before fetch");

        // remote_branch_exists SHOULD find it (queries the remote via ls-remote)
        let remote = remote_branch_exists("feature-remote", &false).unwrap();
        assert!(remote, "branch should be found on the remote");

        std::env::set_current_dir(&original_dir).unwrap();
        cleanup_temp_dir(&bare_dir);
        cleanup_temp_dir(&remote_dir);
    }

    #[test]
    fn test_remote_branch_exists_returns_false_for_unknown() {
        let _lock = lock_cwd();
        let (remote_dir, bare_dir) = setup_remote_and_bare_clone();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&bare_dir).unwrap();

        let result = remote_branch_exists("no-such-branch", &false).unwrap();
        assert!(!result, "non-existent branch should not be found on remote");

        std::env::set_current_dir(&original_dir).unwrap();
        cleanup_temp_dir(&bare_dir);
        cleanup_temp_dir(&remote_dir);
    }

    // =========================================================================
    // fetch_branch
    // =========================================================================

    #[test]
    fn test_fetch_branch_makes_remote_branch_available_locally() {
        let _lock = lock_cwd();
        let (remote_dir, bare_dir) = setup_remote_and_bare_clone();
        let original_dir = std::env::current_dir().unwrap();

        // Create a new branch on the remote after the clone
        create_remote_branch(&remote_dir, "feature-fetch", "fetch.txt");

        std::env::set_current_dir(&bare_dir).unwrap();

        // Not available locally yet
        assert!(!branch_exists("feature-fetch", &false).unwrap());

        // Fetch the branch
        fetch_branch("feature-fetch", &false).unwrap();

        // Now it should be available locally
        assert!(
            branch_exists("feature-fetch", &false).unwrap(),
            "branch should exist locally after fetch"
        );

        std::env::set_current_dir(&original_dir).unwrap();
        cleanup_temp_dir(&bare_dir);
        cleanup_temp_dir(&remote_dir);
    }

    #[test]
    fn test_fetch_branch_fails_for_nonexistent_branch() {
        let _lock = lock_cwd();
        let (remote_dir, bare_dir) = setup_remote_and_bare_clone();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&bare_dir).unwrap();

        let result = fetch_branch("no-such-branch", &false);
        assert!(result.is_err(), "fetching non-existent branch should fail");

        std::env::set_current_dir(&original_dir).unwrap();
        cleanup_temp_dir(&bare_dir);
        cleanup_temp_dir(&remote_dir);
    }

    // =========================================================================
    // End-to-end: the full flow that run_add/run_switch now uses
    // =========================================================================

    #[test]
    fn test_worktree_add_works_after_fetching_remote_branch() {
        let _lock = lock_cwd();
        let (remote_dir, bare_dir) = setup_remote_and_bare_clone();
        let original_dir = std::env::current_dir().unwrap();

        // Create a new branch on the remote after the clone
        create_remote_branch(&remote_dir, "feature-worktree", "wt.txt");

        std::env::set_current_dir(&bare_dir).unwrap();

        // Simulate the fixed run_add flow:
        // 1. branch_exists → false
        assert!(!branch_exists("feature-worktree", &false).unwrap());
        // 2. remote_branch_exists → true
        assert!(remote_branch_exists("feature-worktree", &false).unwrap());
        // 3. fetch_branch
        fetch_branch("feature-worktree", &false).unwrap();
        // 4. worktree_add succeeds
        let wt_path = create_temp_dir("wt-output");
        let _ = std::fs::remove_dir_all(&wt_path); // worktree_add creates it
        let result = worktree_add(wt_path.to_str().unwrap(), "feature-worktree", &false);
        assert!(result.is_ok(), "worktree_add should succeed after fetch: {:?}", result.err());

        // Verify the worktree directory was created with the right content
        assert!(wt_path.join("wt.txt").exists(), "worktree should contain the file from the remote branch");

        // Clean up
        let _ = Command::new("git")
            .args(&["worktree", "remove", "--force", wt_path.to_str().unwrap()])
            .current_dir(&bare_dir)
            .output();
        std::env::set_current_dir(&original_dir).unwrap();
        cleanup_temp_dir(&wt_path);
        cleanup_temp_dir(&bare_dir);
        cleanup_temp_dir(&remote_dir);
    }

    #[test]
    fn test_worktree_add_without_fetch_fails_for_remote_only_branch() {
        let _lock = lock_cwd();
        let (remote_dir, bare_dir) = setup_remote_and_bare_clone();
        let original_dir = std::env::current_dir().unwrap();

        // Create a new branch on the remote after the clone
        create_remote_branch(&remote_dir, "feature-nofetch", "nofetch.txt");

        std::env::set_current_dir(&bare_dir).unwrap();

        // Without fetching, worktree_add should fail
        let wt_path = create_temp_dir("wt-nofetch");
        let _ = std::fs::remove_dir_all(&wt_path);
        let result = worktree_add(wt_path.to_str().unwrap(), "feature-nofetch", &false);
        assert!(result.is_err(), "worktree_add should fail without fetch for remote-only branch");

        std::env::set_current_dir(&original_dir).unwrap();
        cleanup_temp_dir(&wt_path);
        cleanup_temp_dir(&bare_dir);
        cleanup_temp_dir(&remote_dir);
    }
}
