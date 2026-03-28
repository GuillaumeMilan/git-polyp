use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

use crate::GitPolyp;

pub fn run(shell: Shell) {
    let mut cmd = GitPolyp::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut io::stdout());
}

/// Generate completions to a buffer (used for testing)
pub fn generate_to_buffer(shell: Shell) -> Vec<u8> {
    let mut cmd = GitPolyp::command();
    let name = cmd.get_name().to_string();
    let mut buf = Vec::new();
    generate(shell, &mut cmd, name, &mut buf);
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_generate_bash_completions() {
        let output = generate_to_buffer(Shell::Bash);
        let content = String::from_utf8(output).expect("Valid UTF-8");

        assert!(!content.is_empty(), "Bash completions should not be empty");
        assert!(
            content.contains("_git-polyp"),
            "Bash completions should contain function name"
        );
        assert!(
            content.contains("worktree"),
            "Bash completions should contain worktree command"
        );
        assert!(
            content.contains("completions"),
            "Bash completions should contain completions command"
        );
    }

    #[test]
    fn test_generate_zsh_completions() {
        let output = generate_to_buffer(Shell::Zsh);
        let content = String::from_utf8(output).expect("Valid UTF-8");

        assert!(!content.is_empty(), "Zsh completions should not be empty");
        assert!(
            content.contains("#compdef git-polyp"),
            "Zsh completions should contain compdef directive"
        );
        assert!(
            content.contains("worktree"),
            "Zsh completions should contain worktree command"
        );
        assert!(
            content.contains("completions"),
            "Zsh completions should contain completions command"
        );
    }

    #[test]
    fn test_generate_fish_completions() {
        let output = generate_to_buffer(Shell::Fish);
        let content = String::from_utf8(output).expect("Valid UTF-8");

        assert!(!content.is_empty(), "Fish completions should not be empty");
        assert!(
            content.contains("complete -c git-polyp"),
            "Fish completions should contain complete command"
        );
        assert!(
            content.contains("worktree"),
            "Fish completions should contain worktree command"
        );
        assert!(
            content.contains("completions"),
            "Fish completions should contain completions command"
        );
    }

    #[test]
    fn test_generate_powershell_completions() {
        let output = generate_to_buffer(Shell::PowerShell);
        let content = String::from_utf8(output).expect("Valid UTF-8");

        assert!(
            !content.is_empty(),
            "PowerShell completions should not be empty"
        );
        assert!(
            content.contains("git-polyp"),
            "PowerShell completions should contain command name"
        );
    }

    #[test]
    fn test_generate_elvish_completions() {
        let output = generate_to_buffer(Shell::Elvish);
        let content = String::from_utf8(output).expect("Valid UTF-8");

        assert!(
            !content.is_empty(),
            "Elvish completions should not be empty"
        );
        assert!(
            content.contains("git-polyp"),
            "Elvish completions should contain command name"
        );
    }

    #[test]
    fn test_completions_contain_worktree_subcommands() {
        let output = generate_to_buffer(Shell::Bash);
        let content = String::from_utf8(output).expect("Valid UTF-8");

        // Verify worktree subcommands are present
        let subcommands = ["init", "convert", "add", "clean", "list", "switch", "prune", "check"];
        for subcmd in subcommands {
            assert!(
                content.contains(subcmd),
                "Completions should contain worktree subcommand: {}",
                subcmd
            );
        }
    }

    #[test]
    fn test_completions_contain_rebase_flags() {
        let output = generate_to_buffer(Shell::Bash);
        let content = String::from_utf8(output).expect("Valid UTF-8");

        // Verify rebase flags are present
        assert!(
            content.contains("--abort"),
            "Completions should contain --abort flag"
        );
        assert!(
            content.contains("--continue"),
            "Completions should contain --continue flag"
        );
        assert!(
            content.contains("--verbose"),
            "Completions should contain --verbose flag"
        );
    }

    #[test]
    fn test_parse_completions_command_bash() {
        let args = crate::GitPolyp::try_parse_from(["git-polyp", "completions", "bash"]);
        assert!(args.is_ok(), "Should parse 'completions bash'");
    }

    #[test]
    fn test_parse_completions_command_zsh() {
        let args = crate::GitPolyp::try_parse_from(["git-polyp", "completions", "zsh"]);
        assert!(args.is_ok(), "Should parse 'completions zsh'");
    }

    #[test]
    fn test_parse_completions_command_fish() {
        let args = crate::GitPolyp::try_parse_from(["git-polyp", "completions", "fish"]);
        assert!(args.is_ok(), "Should parse 'completions fish'");
    }

    #[test]
    fn test_parse_completions_command_powershell() {
        let args = crate::GitPolyp::try_parse_from(["git-polyp", "completions", "powershell"]);
        assert!(args.is_ok(), "Should parse 'completions powershell'");
    }

    #[test]
    fn test_parse_completions_command_elvish() {
        let args = crate::GitPolyp::try_parse_from(["git-polyp", "completions", "elvish"]);
        assert!(args.is_ok(), "Should parse 'completions elvish'");
    }

    #[test]
    fn test_parse_completions_missing_shell() {
        let args = crate::GitPolyp::try_parse_from(["git-polyp", "completions"]);
        assert!(args.is_err(), "Should fail without shell argument");
    }

    #[test]
    fn test_parse_completions_invalid_shell() {
        let args = crate::GitPolyp::try_parse_from(["git-polyp", "completions", "invalid"]);
        assert!(args.is_err(), "Should fail with invalid shell");
    }
}
