#compdef git-polyp

# Zsh completion script for git-polyp
# Installation:
#   Copy this file to a directory in your $fpath with the name _git-polyp:
#     cp completions/git-polyp.zsh ~/.zsh/completions/_git-polyp
#   Or add the completions directory to your fpath in ~/.zshrc:
#     fpath=(/path/to/git_polyp/completions $fpath)
#     autoload -Uz compinit && compinit

_git-polyp() {
    local -a commands
    commands=(
        'rebase-stack:Rebase a stack of branches onto a new base'
    )

    local -a global_flags
    global_flags=(
        '(-h --help)'{-h,--help}'[Show help message]'
        '(-v --version)'{-v,--version}'[Show version information]'
    )

    local -a rebase_flags
    rebase_flags=(
        '--continue[Resume rebase after conflict resolution]'
        '--abort[Cancel the rebase operation]'
        '(-h --help)'{-h,--help}'[Show command help]'
    )

    # Get git branches
    local -a branches
    branches=(${(f)"$(git branch --format='%(refname:short)' 2>/dev/null)"})

    _arguments -C \
        '1: :->command' \
        '*:: :->args' \
        && return 0

    case $state in
        command)
            _describe -t commands 'git-polyp commands' commands
            _describe -t flags 'global flags' global_flags
            ;;
        args)
            case $words[1] in
                rebase-stack)
                    # Check for --continue or --abort
                    if (( ${words[(I)--continue]} || ${words[(I)--abort]} )); then
                        # Don't suggest anything else if these flags are present
                        return 0
                    fi

                    # Count non-flag arguments
                    local -a args_only
                    args_only=(${words:#-*})
                    local arg_count=${#args_only}

                    if [[ $arg_count -le 2 ]]; then
                        # First positional arg (base-branch)
                        _describe -t branches 'base branch' branches
                        _describe -t flags 'rebase-stack flags' rebase_flags
                    elif [[ $arg_count -eq 3 ]]; then
                        # Second positional arg (target-branch)
                        _describe -t branches 'target branch' branches
                        _describe -t flags 'rebase-stack flags' rebase_flags
                    else
                        # Only flags after both positional args
                        _describe -t flags 'rebase-stack flags' rebase_flags
                    fi
                    ;;
            esac
            ;;
    esac
}

_git-polyp "$@"
