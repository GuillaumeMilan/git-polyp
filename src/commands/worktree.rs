pub mod detect;
pub mod metadata;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct WorktreeArgs {
    #[command(subcommand)]
    pub command: WorktreeCommands,
}

#[derive(Subcommand, Debug)]
pub enum WorktreeCommands {
    /// Clone a repo into a git-polyp worktree workspace
    Init {
        /// Git repository URL or local path to clone from
        url: String,
        /// Destination path (defaults to repo name from URL)
        path: Option<String>,
    },
    /// Convert an existing repo into a worktree workspace
    Convert,
    /// Add a new worktree for a branch
    Add {
        /// Name of the branch
        branch: String,
        /// Base ref to create branch from (default: main branch)
        #[arg(long)]
        base: Option<String>,
    },
    /// Remove a worktree entry
    Clean {
        /// Name of the branch/worktree to remove
        branch: String,
        /// Force removal even if worktree has uncommitted changes
        #[arg(long)]
        force: bool,
        /// Also delete the branch without prompting
        #[arg(long, conflicts_with = "keep_branch")]
        delete_branch: bool,
        /// Keep the branch without prompting
        #[arg(long, conflicts_with = "delete_branch")]
        keep_branch: bool,
    },
    /// Show all worktrees with status
    List,
    /// Output path to a worktree for cd navigation
    Switch {
        /// Name of the branch
        branch: String,
    },
    /// Clean up worktrees for merged/deleted branches
    Prune,
    /// Verify health of all worktrees
    Check,
}

pub fn run(args: WorktreeArgs, verbose: bool) {
    match args.command {
        WorktreeCommands::Init { url, path } => {
            run_init(&url, path.as_deref(), verbose);
        }
        WorktreeCommands::Convert => {
            run_convert(verbose);
        }
        WorktreeCommands::Add { branch, base } => {
            run_add(&branch, base.as_deref(), verbose);
        }
        WorktreeCommands::Clean {
            branch,
            force,
            delete_branch,
            keep_branch,
        } => {
            run_clean(&branch, force, delete_branch, keep_branch, verbose);
        }
        WorktreeCommands::List => {
            run_list(verbose);
        }
        WorktreeCommands::Switch { branch } => {
            run_switch(&branch, verbose);
        }
        WorktreeCommands::Prune => {
            run_prune(verbose);
        }
        WorktreeCommands::Check => {
            run_check(verbose);
        }
    }
}

fn run_init(_url: &str, _path: Option<&str>, _verbose: bool) {
    eprintln!("worktree init: not yet implemented (Phase 2)");
    std::process::exit(1);
}

fn run_convert(_verbose: bool) {
    eprintln!("worktree convert: not yet implemented (Phase 3)");
    std::process::exit(1);
}

fn run_add(_branch: &str, _base: Option<&str>, _verbose: bool) {
    eprintln!("worktree add: not yet implemented (Phase 2)");
    std::process::exit(1);
}

fn run_clean(
    _branch: &str,
    _force: bool,
    _delete_branch: bool,
    _keep_branch: bool,
    _verbose: bool,
) {
    eprintln!("worktree clean: not yet implemented (Phase 2)");
    std::process::exit(1);
}

fn run_list(_verbose: bool) {
    eprintln!("worktree list: not yet implemented (Phase 2)");
    std::process::exit(1);
}

fn run_switch(_branch: &str, _verbose: bool) {
    eprintln!("worktree switch: not yet implemented (Phase 3)");
    std::process::exit(1);
}

fn run_prune(_verbose: bool) {
    eprintln!("worktree prune: not yet implemented (Phase 3)");
    std::process::exit(1);
}

fn run_check(_verbose: bool) {
    eprintln!("worktree check: not yet implemented (Phase 3)");
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    // Helper to parse worktree args from command line style input
    fn parse_args(args: &[&str]) -> Result<WorktreeArgs, clap::Error> {
        WorktreeArgs::try_parse_from(args)
    }

    // =========================================================================
    // Init command tests
    // =========================================================================

    #[test]
    fn test_parse_init_with_url_only() {
        let args = parse_args(&["worktree", "init", "https://github.com/user/repo.git"]).unwrap();
        match args.command {
            WorktreeCommands::Init { url, path } => {
                assert_eq!(url, "https://github.com/user/repo.git");
                assert!(path.is_none());
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_parse_init_with_url_and_path() {
        let args = parse_args(&["worktree", "init", "https://github.com/user/repo.git", "my-repo"]).unwrap();
        match args.command {
            WorktreeCommands::Init { url, path } => {
                assert_eq!(url, "https://github.com/user/repo.git");
                assert_eq!(path, Some("my-repo".to_string()));
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_parse_init_with_local_path() {
        let args = parse_args(&["worktree", "init", "/path/to/local/repo"]).unwrap();
        match args.command {
            WorktreeCommands::Init { url, path } => {
                assert_eq!(url, "/path/to/local/repo");
                assert!(path.is_none());
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_parse_init_missing_url() {
        let result = parse_args(&["worktree", "init"]);
        assert!(result.is_err());
    }

    // =========================================================================
    // Convert command tests
    // =========================================================================

    #[test]
    fn test_parse_convert() {
        let args = parse_args(&["worktree", "convert"]).unwrap();
        assert!(matches!(args.command, WorktreeCommands::Convert));
    }

    // =========================================================================
    // Add command tests
    // =========================================================================

    #[test]
    fn test_parse_add_branch_only() {
        let args = parse_args(&["worktree", "add", "feature-x"]).unwrap();
        match args.command {
            WorktreeCommands::Add { branch, base } => {
                assert_eq!(branch, "feature-x");
                assert!(base.is_none());
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_parse_add_with_base() {
        let args = parse_args(&["worktree", "add", "feature-x", "--base", "develop"]).unwrap();
        match args.command {
            WorktreeCommands::Add { branch, base } => {
                assert_eq!(branch, "feature-x");
                assert_eq!(base, Some("develop".to_string()));
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_parse_add_missing_branch() {
        let result = parse_args(&["worktree", "add"]);
        assert!(result.is_err());
    }

    // =========================================================================
    // Clean command tests
    // =========================================================================

    #[test]
    fn test_parse_clean_branch_only() {
        let args = parse_args(&["worktree", "clean", "feature-x"]).unwrap();
        match args.command {
            WorktreeCommands::Clean {
                branch,
                force,
                delete_branch,
                keep_branch,
            } => {
                assert_eq!(branch, "feature-x");
                assert!(!force);
                assert!(!delete_branch);
                assert!(!keep_branch);
            }
            _ => panic!("Expected Clean command"),
        }
    }

    #[test]
    fn test_parse_clean_with_force() {
        let args = parse_args(&["worktree", "clean", "feature-x", "--force"]).unwrap();
        match args.command {
            WorktreeCommands::Clean { force, .. } => {
                assert!(force);
            }
            _ => panic!("Expected Clean command"),
        }
    }

    #[test]
    fn test_parse_clean_with_delete_branch() {
        let args = parse_args(&["worktree", "clean", "feature-x", "--delete-branch"]).unwrap();
        match args.command {
            WorktreeCommands::Clean { delete_branch, keep_branch, .. } => {
                assert!(delete_branch);
                assert!(!keep_branch);
            }
            _ => panic!("Expected Clean command"),
        }
    }

    #[test]
    fn test_parse_clean_with_keep_branch() {
        let args = parse_args(&["worktree", "clean", "feature-x", "--keep-branch"]).unwrap();
        match args.command {
            WorktreeCommands::Clean { delete_branch, keep_branch, .. } => {
                assert!(!delete_branch);
                assert!(keep_branch);
            }
            _ => panic!("Expected Clean command"),
        }
    }

    #[test]
    fn test_parse_clean_delete_and_keep_conflict() {
        // --delete-branch and --keep-branch are mutually exclusive
        let result = parse_args(&["worktree", "clean", "feature-x", "--delete-branch", "--keep-branch"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_clean_all_flags() {
        let args = parse_args(&["worktree", "clean", "feature-x", "--force", "--delete-branch"]).unwrap();
        match args.command {
            WorktreeCommands::Clean {
                branch,
                force,
                delete_branch,
                keep_branch,
            } => {
                assert_eq!(branch, "feature-x");
                assert!(force);
                assert!(delete_branch);
                assert!(!keep_branch);
            }
            _ => panic!("Expected Clean command"),
        }
    }

    // =========================================================================
    // List command tests
    // =========================================================================

    #[test]
    fn test_parse_list() {
        let args = parse_args(&["worktree", "list"]).unwrap();
        assert!(matches!(args.command, WorktreeCommands::List));
    }

    // =========================================================================
    // Switch command tests
    // =========================================================================

    #[test]
    fn test_parse_switch() {
        let args = parse_args(&["worktree", "switch", "feature-x"]).unwrap();
        match args.command {
            WorktreeCommands::Switch { branch } => {
                assert_eq!(branch, "feature-x");
            }
            _ => panic!("Expected Switch command"),
        }
    }

    #[test]
    fn test_parse_switch_missing_branch() {
        let result = parse_args(&["worktree", "switch"]);
        assert!(result.is_err());
    }

    // =========================================================================
    // Prune command tests
    // =========================================================================

    #[test]
    fn test_parse_prune() {
        let args = parse_args(&["worktree", "prune"]).unwrap();
        assert!(matches!(args.command, WorktreeCommands::Prune));
    }

    // =========================================================================
    // Check command tests
    // =========================================================================

    #[test]
    fn test_parse_check() {
        let args = parse_args(&["worktree", "check"]).unwrap();
        assert!(matches!(args.command, WorktreeCommands::Check));
    }

    // =========================================================================
    // Error cases
    // =========================================================================

    #[test]
    fn test_parse_unknown_subcommand() {
        let result = parse_args(&["worktree", "unknown"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_no_subcommand() {
        let result = parse_args(&["worktree"]);
        assert!(result.is_err());
    }
}
