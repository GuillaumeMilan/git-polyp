# Fish completion script for git-polyp
# Installation:
#   Copy this file to your Fish completions directory:
#     cp completions/git-polyp.fish ~/.config/fish/completions/
#   Or create a symlink:
#     ln -s /path/to/git_polyp/completions/git-polyp.fish ~/.config/fish/completions/

# Helper function to check if a flag is already present
function __fish_git_polyp_has_flag
    set -l cmd (commandline -opc)
    return (contains -- $argv[1] $cmd)
end

# Helper function to get the number of non-flag arguments
function __fish_git_polyp_arg_count
    set -l cmd (commandline -opc)
    set -l count 0
    for arg in $cmd[2..-1]
        if not string match -q -- '-*' $arg
            set count (math $count + 1)
        end
    end
    echo $count
end

# Helper function to check if we're completing for rebase-stack command
function __fish_git_polyp_using_command
    set -l cmd (commandline -opc)
    return (contains -- $argv[1] $cmd)
end

# Get git branches for completion
function __fish_git_polyp_branches
    git branch --format='%(refname:short)' 2>/dev/null
end

# Global flags (only show if no command yet)
complete -c git-polyp -n "not __fish_seen_subcommand_from rebase-stack" -s h -l help -d "Show help message"
complete -c git-polyp -n "not __fish_seen_subcommand_from rebase-stack" -s v -l version -d "Show version information"

# Commands
complete -c git-polyp -n "not __fish_seen_subcommand_from rebase-stack" -a "rebase-stack" -d "Rebase a stack of branches onto a new base"

# rebase-stack flags
complete -c git-polyp -n "__fish_seen_subcommand_from rebase-stack" -l continue -d "Resume rebase after conflict resolution"
complete -c git-polyp -n "__fish_seen_subcommand_from rebase-stack" -l abort -d "Cancel the rebase operation"
complete -c git-polyp -n "__fish_seen_subcommand_from rebase-stack" -s h -l help -d "Show command help"

# Branch name completions for rebase-stack
# Only suggest branches if --continue or --abort are not present
complete -c git-polyp -n "__fish_seen_subcommand_from rebase-stack; and not __fish_git_polyp_has_flag --continue; and not __fish_git_polyp_has_flag --abort" -a "(__fish_git_polyp_branches)" -d "Branch"
