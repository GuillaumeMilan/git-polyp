pub mod detect;
pub mod metadata;

use crate::client;
use crate::io::{Decorate, YNQuestion};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::Path;

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

/// Extract repository name from a URL or path
fn extract_repo_name(url: &str) -> Option<String> {
    // Handle URLs like https://github.com/user/repo.git or git@github.com:user/repo.git
    let path = url
        .trim_end_matches('/')
        .trim_end_matches(".git");

    // Get the last path component
    path.rsplit(&['/', ':'][..])
        .next()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn run_init(url: &str, path: Option<&str>, verbose: bool) {
    // Determine destination path
    let dest_name = match path {
        Some(p) => p.to_string(),
        None => match extract_repo_name(url) {
            Some(name) => name,
            None => {
                eprintln!("{}", "Could not extract repository name from URL. Please specify a destination path.".deco_as_error());
                std::process::exit(1);
            }
        }
    };

    let dest_path = Path::new(&dest_name);

    // Check if destination already exists
    if dest_path.exists() {
        eprintln!("{}", format!("Destination '{}' already exists.", dest_name).deco_as_error());
        std::process::exit(1);
    }

    // Create the workspace directory
    if let Err(e) = std::fs::create_dir_all(dest_path) {
        eprintln!("{}", format!("Failed to create directory '{}': {}", dest_name, e).deco_as_error());
        std::process::exit(1);
    }

    // Clone as bare repo into .bare
    let bare_path = dest_path.join(".bare");
    println!("Cloning repository into {}...", bare_path.display().to_string().bright_green());

    if let Err(e) = client::clone_bare(url, bare_path.to_str().unwrap(), &verbose) {
        // Clean up on failure
        let _ = std::fs::remove_dir_all(dest_path);
        eprintln!("{}", format!("Failed to clone repository: {:?}", e).deco_as_error());
        std::process::exit(1);
    }

    // Create .template directory
    let template_path = dest_path.join(".template");
    if let Err(e) = std::fs::create_dir(&template_path) {
        let _ = std::fs::remove_dir_all(dest_path);
        eprintln!("{}", format!("Failed to create .template directory: {}", e).deco_as_error());
        std::process::exit(1);
    }

    // Change to bare repo directory to detect main branch
    let original_dir = std::env::current_dir().unwrap();
    if let Err(e) = std::env::set_current_dir(&bare_path) {
        let _ = std::fs::remove_dir_all(dest_path);
        eprintln!("{}", format!("Failed to change to bare repo directory: {}", e).deco_as_error());
        std::process::exit(1);
    }

    // Detect main branch
    let main_branch = match client::detect_main_branch(&verbose) {
        Ok(branch) => branch,
        Err(_) => {
            // Fallback to "main"
            "main".to_string()
        }
    };

    // Change back to original directory
    let _ = std::env::set_current_dir(&original_dir);

    // Create metadata file
    let mut metadata = metadata::WorktreeMetadata::new(main_branch.clone());
    metadata.add_worktree(main_branch.clone(), main_branch.clone());

    if let Err(e) = metadata.save(dest_path) {
        let _ = std::fs::remove_dir_all(dest_path);
        eprintln!("{}", format!("Failed to create metadata file: {:?}", e).deco_as_error());
        std::process::exit(1);
    }

    // Change to bare repo directory for worktree operations
    if let Err(e) = std::env::set_current_dir(&bare_path) {
        let _ = std::fs::remove_dir_all(dest_path);
        eprintln!("{}", format!("Failed to change to workspace directory: {}", e).deco_as_error());
        std::process::exit(1);
    }

    // Create worktree for main branch
    let main_worktree_path = dest_path.join(&main_branch);
    let main_worktree_path_str = main_worktree_path.to_str().unwrap();

    println!("Creating worktree for branch '{}'...", main_branch.bright_cyan());

    if let Err(e) = client::worktree_add(main_worktree_path_str, &main_branch, &verbose) {
        let _ = std::env::set_current_dir(&original_dir);
        let _ = std::fs::remove_dir_all(dest_path);
        eprintln!("{}", format!("Failed to create worktree for main branch: {:?}", e).deco_as_error());
        std::process::exit(1);
    }

    // Change back to original directory
    let _ = std::env::set_current_dir(&original_dir);

    println!("\n{}", "Workspace created successfully!".bright_green().bold());
    println!("\nStructure:");
    println!("  {}/", dest_name.bright_blue());
    println!("    {}     (main branch worktree)", main_branch.bright_cyan());
    println!("    {}            (bare repository)", ".bare".bright_yellow());
    println!("    {}         (template files)", ".template".bright_yellow());
    println!("    {} (metadata)", metadata::METADATA_FILENAME.bright_yellow());
    println!("\nTo get started:");
    println!("  cd {}/{}", dest_name, main_branch);
}

fn run_convert(_verbose: bool) {
    eprintln!("worktree convert: not yet implemented (Phase 3)");
    std::process::exit(1);
}

fn run_add(branch: &str, base: Option<&str>, verbose: bool) {
    // Find worktree root
    let worktree_root = match detect::find_worktree_root() {
        Ok(root) => root,
        Err(_) => {
            eprintln!("{}", "Not in a git-polyp worktree workspace. Run 'git-polyp worktree init' first.".deco_as_error());
            std::process::exit(1);
        }
    };

    // Load metadata
    let mut metadata = match metadata::WorktreeMetadata::load(&worktree_root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", format!("Failed to load metadata: {:?}", e).deco_as_error());
            std::process::exit(1);
        }
    };

    // Check if worktree already exists
    if metadata.has_worktree(branch) {
        println!("Worktree for branch '{}' already exists.", branch.bright_cyan());
        std::process::exit(0);
    }

    // Get bare repo path and change to it for git operations
    let bare_path = detect::get_bare_repo_path(&worktree_root);
    let original_dir = std::env::current_dir().unwrap();

    if let Err(e) = std::env::set_current_dir(&bare_path) {
        eprintln!("{}", format!("Failed to change to bare repo directory: {}", e).deco_as_error());
        std::process::exit(1);
    }

    // Determine the base ref for new branches
    let base_ref = base.map(|s| s.to_string()).unwrap_or_else(|| metadata.main_branch.clone());

    // Check if branch exists
    let branch_exists = match client::branch_exists(branch, &verbose) {
        Ok(exists) => exists,
        Err(e) => {
            let _ = std::env::set_current_dir(&original_dir);
            eprintln!("{}", format!("Failed to check if branch exists: {:?}", e).deco_as_error());
            std::process::exit(1);
        }
    };

    // Create worktree path
    let worktree_path = worktree_root.join(branch);
    let worktree_path_str = worktree_path.to_str().unwrap();

    if branch_exists {
        // Create worktree from existing branch
        println!("Creating worktree for existing branch '{}'...", branch.bright_cyan());
        if let Err(e) = client::worktree_add(worktree_path_str, branch, &verbose) {
            let _ = std::env::set_current_dir(&original_dir);
            eprintln!("{}", format!("Failed to create worktree: {:?}", e).deco_as_error());
            std::process::exit(1);
        }
        println!("{}", format!("Created worktree for existing branch '{}'", branch).bright_green());
    } else {
        // Create new branch from base and create worktree
        println!("Creating new branch '{}' from '{}'...", branch.bright_cyan(), base_ref.bright_cyan());
        if let Err(e) = client::worktree_add_new_branch(worktree_path_str, branch, &base_ref, &verbose) {
            let _ = std::env::set_current_dir(&original_dir);
            eprintln!("{}", format!("Failed to create worktree with new branch: {:?}", e).deco_as_error());
            std::process::exit(1);
        }
        println!("{}", format!("Created new branch '{}' from '{}' and worktree", branch, base_ref).bright_green());
    }

    // Update metadata
    metadata.add_worktree(branch.to_string(), branch.to_string());
    let _ = std::env::set_current_dir(&original_dir);

    if let Err(e) = metadata.save(&worktree_root) {
        eprintln!("{}", format!("Warning: Failed to update metadata: {:?}", e).bright_yellow());
    }

    println!("\nWorktree location: {}", worktree_path_str.bright_blue());
}

fn run_clean(
    branch: &str,
    force: bool,
    delete_branch: bool,
    keep_branch: bool,
    verbose: bool,
) {
    // Find worktree root
    let worktree_root = match detect::find_worktree_root() {
        Ok(root) => root,
        Err(_) => {
            eprintln!("{}", "Not in a git-polyp worktree workspace.".deco_as_error());
            std::process::exit(1);
        }
    };

    // Load metadata
    let mut metadata = match metadata::WorktreeMetadata::load(&worktree_root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", format!("Failed to load metadata: {:?}", e).deco_as_error());
            std::process::exit(1);
        }
    };

    // Check if this is the main branch
    if branch == metadata.main_branch {
        eprintln!("{}", format!("Cannot remove the main branch worktree '{}'.", branch).deco_as_error());
        std::process::exit(1);
    }

    // Check if worktree exists in metadata
    if !metadata.has_worktree(branch) {
        eprintln!("{}", format!("Worktree '{}' not found.", branch).deco_as_error());
        std::process::exit(1);
    }

    // Get worktree path
    let worktree_path = worktree_root.join(branch);
    let worktree_path_str = worktree_path.to_str().unwrap();

    // Check for uncommitted changes
    if worktree_path.exists() {
        match client::worktree_is_dirty(worktree_path_str, &verbose) {
            Ok(true) => {
                if !force {
                    eprintln!("{}", format!("Worktree '{}' has uncommitted changes. Use --force to remove anyway.", branch).deco_as_error());
                    std::process::exit(1);
                }
                println!("{}", "Warning: Removing worktree with uncommitted changes.".bright_yellow());
            }
            Ok(false) => {}
            Err(e) => {
                eprintln!("{}", format!("Failed to check worktree status: {:?}", e).deco_as_error());
                std::process::exit(1);
            }
        }
    }

    // Change to bare repo for git operations
    let bare_path = detect::get_bare_repo_path(&worktree_root);
    let original_dir = std::env::current_dir().unwrap();

    if let Err(e) = std::env::set_current_dir(&bare_path) {
        eprintln!("{}", format!("Failed to change to bare repo directory: {}", e).deco_as_error());
        std::process::exit(1);
    }

    // Remove worktree
    println!("Removing worktree '{}'...", branch.bright_cyan());
    if let Err(e) = client::worktree_remove(worktree_path_str, force, &verbose) {
        let _ = std::env::set_current_dir(&original_dir);
        eprintln!("{}", format!("Failed to remove worktree: {:?}", e).deco_as_error());
        std::process::exit(1);
    }

    // Update metadata
    metadata.remove_worktree(branch);
    let _ = std::env::set_current_dir(&original_dir);

    if let Err(e) = metadata.save(&worktree_root) {
        eprintln!("{}", format!("Warning: Failed to update metadata: {:?}", e).bright_yellow());
    }

    println!("{}", format!("Removed worktree '{}'", branch).bright_green());

    // Handle branch deletion
    let should_delete_branch = if delete_branch {
        true
    } else if keep_branch {
        false
    } else {
        // Prompt user
        match YNQuestion::new(format!("Also delete branch '{}'?", branch)).ask() {
            Ok(answer) => answer,
            Err(_) => false,
        }
    };

    if should_delete_branch {
        // Change back to bare repo for branch deletion
        if let Err(e) = std::env::set_current_dir(&bare_path) {
            eprintln!("{}", format!("Failed to change to bare repo directory: {}", e).deco_as_error());
            std::process::exit(1);
        }

        if let Err(e) = client::delete_branch(branch, force, &verbose) {
            let _ = std::env::set_current_dir(&original_dir);
            eprintln!("{}", format!("Warning: Failed to delete branch '{}': {:?}", branch, e).bright_yellow());
        } else {
            let _ = std::env::set_current_dir(&original_dir);
            println!("{}", format!("Deleted branch '{}'", branch).bright_green());
        }
    }
}

fn run_list(verbose: bool) {
    // Find worktree root
    let worktree_root = match detect::find_worktree_root() {
        Ok(root) => root,
        Err(_) => {
            eprintln!("{}", "Not in a git-polyp worktree workspace.".deco_as_error());
            std::process::exit(1);
        }
    };

    // Load metadata
    let metadata = match metadata::WorktreeMetadata::load(&worktree_root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", format!("Failed to load metadata: {:?}", e).deco_as_error());
            std::process::exit(1);
        }
    };

    // Change to bare repo for git operations
    let bare_path = detect::get_bare_repo_path(&worktree_root);
    let original_dir = std::env::current_dir().unwrap();

    if let Err(e) = std::env::set_current_dir(&bare_path) {
        eprintln!("{}", format!("Failed to change to bare repo directory: {}", e).deco_as_error());
        std::process::exit(1);
    }

    println!("Worktrees in {}:\n", worktree_root.display().to_string().bright_blue());

    // Collect worktree names and sort them (main branch first)
    let mut worktree_names: Vec<&String> = metadata.worktrees.keys().collect();
    worktree_names.sort_by(|a, b| {
        if *a == &metadata.main_branch {
            std::cmp::Ordering::Less
        } else if *b == &metadata.main_branch {
            std::cmp::Ordering::Greater
        } else {
            a.cmp(b)
        }
    });

    for name in worktree_names {
        let entry = metadata.get_worktree(name).unwrap();
        let worktree_path = worktree_root.join(name);
        let worktree_path_str = worktree_path.to_str().unwrap();

        // Check if worktree directory exists
        let exists = worktree_path.exists();

        // Get dirty status
        let status = if !exists {
            "[missing]".bright_red().to_string()
        } else {
            match client::worktree_is_dirty(worktree_path_str, &verbose) {
                Ok(true) => "[dirty]".bright_yellow().to_string(),
                Ok(false) => "[clean]".bright_green().to_string(),
                Err(_) => "[?]".bright_red().to_string(),
            }
        };

        // Get ahead/behind info
        let ahead_behind = if exists {
            match client::get_ahead_behind(&entry.branch, &verbose) {
                Ok((ahead, behind)) => {
                    let mut parts = Vec::new();
                    if ahead > 0 {
                        parts.push(format!("↑{}", ahead).bright_cyan().to_string());
                    }
                    if behind > 0 {
                        parts.push(format!("↓{}", behind).bright_magenta().to_string());
                    }
                    if parts.is_empty() {
                        String::new()
                    } else {
                        parts.join(" ")
                    }
                }
                Err(_) => String::new(),
            }
        } else {
            String::new()
        };

        // Format output
        let name_display = format!("{:<16}", name);
        let branch_display = format!("{:<16}", entry.branch);

        print!("  {} {} {}", name_display.bright_cyan(), branch_display, status);
        if !ahead_behind.is_empty() {
            print!("  {}", ahead_behind);
        }
        println!();
    }

    let _ = std::env::set_current_dir(&original_dir);
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

    // =========================================================================
    // extract_repo_name tests
    // =========================================================================

    #[test]
    fn test_extract_repo_name_https_url() {
        let name = extract_repo_name("https://github.com/user/repo.git");
        assert_eq!(name, Some("repo".to_string()));
    }

    #[test]
    fn test_extract_repo_name_https_url_no_git_suffix() {
        let name = extract_repo_name("https://github.com/user/repo");
        assert_eq!(name, Some("repo".to_string()));
    }

    #[test]
    fn test_extract_repo_name_ssh_url() {
        let name = extract_repo_name("git@github.com:user/repo.git");
        assert_eq!(name, Some("repo".to_string()));
    }

    #[test]
    fn test_extract_repo_name_ssh_url_no_git_suffix() {
        let name = extract_repo_name("git@github.com:user/repo");
        assert_eq!(name, Some("repo".to_string()));
    }

    #[test]
    fn test_extract_repo_name_local_path() {
        let name = extract_repo_name("/path/to/local/repo");
        assert_eq!(name, Some("repo".to_string()));
    }

    #[test]
    fn test_extract_repo_name_local_path_with_trailing_slash() {
        let name = extract_repo_name("/path/to/local/repo/");
        assert_eq!(name, Some("repo".to_string()));
    }

    #[test]
    fn test_extract_repo_name_relative_path() {
        let name = extract_repo_name("../my-project");
        assert_eq!(name, Some("my-project".to_string()));
    }

    #[test]
    fn test_extract_repo_name_simple_name() {
        let name = extract_repo_name("my-repo");
        assert_eq!(name, Some("my-repo".to_string()));
    }
}
