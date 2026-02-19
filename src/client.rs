use std::os::unix::process::ExitStatusExt;

struct GitCommand {
    args: Vec<String>,
}

#[derive(Debug)]
pub enum ClientError {
    Command,
    NonZeroExitCode,
    InvalidUtf8,
    CanNotCreatePolypDir,
}

impl GitCommand {
    fn new(args: Vec<String>) -> Self {
        Self { args }
    }

    fn execute(&self) -> Result<String, ClientError> {
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

pub fn merge_base(upstream: &str, branch: &str) -> Result<String, ClientError> {
    GitCommand::new(vec![
        "merge-base".to_string(),
        upstream.to_string(),
        branch.to_string(),
    ])
    .execute()
}

pub fn dot_git_dir() -> Result<String, ClientError> {
    GitCommand::new(vec!["rev-parse".to_string(), "--git-dir".to_string()]).execute()
}

pub fn polyp_dir() -> Result<String, ClientError> {
    let git_dir = dot_git_dir()?;
    let polyp_dir = format!("{}/polyp", git_dir);
    if !std::path::Path::new(&polyp_dir).exists() {
        std::fs::create_dir(&polyp_dir).map_err(|_| ClientError::CanNotCreatePolypDir)?;
    };
    Ok(polyp_dir)
}

pub fn current_branch() -> Result<String, ClientError> {
    GitCommand::new(vec![
        "rev-parse".to_string(),
        "--abbrev-ref".to_string(),
        "HEAD".to_string(),
    ])
    .execute()
}

pub fn rev_parse(rev: &str) -> Result<String, ClientError> {
    GitCommand::new(vec!["rev-parse".to_string(), rev.to_string()]).execute()
}

pub fn is_in_repo() -> Result<bool, ClientError> {
    let output = match std::process::Command::new("git")
        .args(["rev-parse"])
        .output()
    {
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

pub fn rev_list(upstream: &str, branch: &str) -> Result<Vec<String>, ClientError> {
    let output = GitCommand::new(vec![
        "rev-list".to_string(),
        "--reverse".to_string(),
        format!("{}..{}", upstream, branch),
    ])
    .execute()?;

    Ok(output.lines().map(|line| line.to_string()).collect())
}

pub fn commit_message(commit_hash: &str) -> Result<String, ClientError> {
    GitCommand::new(vec![
        "log".to_string(),
        "-1".to_string(),
        "--format=%s".to_string(),
        commit_hash.to_string(),
    ])
    .execute()
}

pub fn branches_at(commit_hash: &str) -> Result<Vec<String>, ClientError> {
    let output = GitCommand::new(vec![
        "branch".to_string(),
        "--points-at".to_string(),
        commit_hash.to_string(),
        "--format=%(refname:short)".to_string(),
    ])
    .execute()?;

    Ok(output.lines().map(|line| line.trim().to_string()).collect())
}

pub fn checkout(revision: &str) -> Result<(), ClientError> {
    GitCommand::new(vec!["checkout".to_string(), revision.to_string()])
        .execute()
        .map(|_| ())
}

pub fn cherry_pick(commit_a: &str, commit_b: &str) -> Result<(), ClientError> {
    let args = vec![
        "cherry-pick".to_string(),
        format!("{}^..{}", commit_a, commit_b),
    ];
    GitCommand::new(args).execute().map(|_| ())
}

pub fn move_branche_at(commit_hash: &str, branch: &str) -> Result<(), ClientError> {
    GitCommand::new(vec![
        "branch".to_string(),
        "-f".to_string(),
        branch.to_string(),
        commit_hash.to_string(),
    ])
    .execute()
    .map(|_| ())
}

pub fn push_branches(remote: &str, branches: Vec<String>) -> Result<(), ClientError> {
    let mut args = vec!["push".to_string(), remote.to_string()];

    for branch in branches {
        args.push(branch);
    }
    GitCommand::new(args).execute().map(|_| ())
}
