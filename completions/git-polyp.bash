# Bash completion script for git-polyp
# Installation:
#   Source this file in your ~/.bashrc or ~/.bash_profile:
#     source /path/to/completions/git-polyp.bash
#   Or copy to your bash completion directory:
#     cp completions/git-polyp.bash /usr/local/etc/bash_completion.d/git-polyp

_git_polyp_completion() {
    local cur prev words cword

    # Use _init_completion if available (from bash-completion package)
    # Otherwise, fall back to manual initialization
    if declare -F _init_completion >/dev/null; then
        _init_completion || return
    else
        # Manual initialization for systems without bash-completion
        COMPREPLY=()
        cur="${COMP_WORDS[COMP_CWORD]}"
        prev="${COMP_WORDS[COMP_CWORD-1]}"
        words=("${COMP_WORDS[@]}")
        cword=$COMP_CWORD
    fi

    local commands="rebase-stack"
    local global_flags="--help -h --version -v"
    local rebase_flags="--continue --abort --help -h"

    # Get git branches for completion
    _git_polyp_branches() {
        git branch --format='%(refname:short)' 2>/dev/null
    }

    # Handle completion based on position
    case $cword in
        1)
            # First argument: suggest command or global flags
            COMPREPLY=($(compgen -W "$commands $global_flags" -- "$cur"))
            ;;
        *)
            # Check what command we're completing for
            if [[ "${words[1]}" == "rebase-stack" ]]; then
                # Check if we already have flags
                local has_continue=false
                local has_abort=false

                for word in "${words[@]}"; do
                    [[ "$word" == "--continue" ]] && has_continue=true
                    [[ "$word" == "--abort" ]] && has_abort=true
                done

                # If --continue or --abort is present, don't suggest anything else
                if $has_continue || $has_abort; then
                    return 0
                fi

                # Count non-flag arguments (excluding command name)
                local arg_count=0
                for word in "${words[@]:2}"; do
                    [[ "$word" != -* ]] && ((arg_count++))
                done

                # Suggest branches or flags based on context
                if [[ "$cur" == -* ]]; then
                    # User is typing a flag
                    COMPREPLY=($(compgen -W "$rebase_flags" -- "$cur"))
                else
                    # Suggest branches
                    local branches=$(_git_polyp_branches)
                    if [[ -n "$branches" ]]; then
                        COMPREPLY=($(compgen -W "$branches $rebase_flags" -- "$cur"))
                    else
                        COMPREPLY=($(compgen -W "$rebase_flags" -- "$cur"))
                    fi
                fi
            fi
            ;;
    esac
}

complete -F _git_polyp_completion git-polyp
