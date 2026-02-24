use crate::io::Decorate;
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

pub fn checkout(revision: &str, verbose: &bool) -> Result<(), ClientError> {
    GitCommand::new(vec!["checkout".to_string(), revision.to_string()], verbose)
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
            "-f".to_string(),
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
    if *verbose {
        let command_args_str = args.iter().fold(String::new(), |acc, arg| acc + " " + arg);
        let command_str = format!("> git {}", command_args_str).deco_as_command();
        println!("[executing] {}", command_str);
    }
}
