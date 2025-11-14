#!/usr/bin/env bash
# Installation script for git-polyp shell completions

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

detect_shell() {
    # First try to detect from SHELL environment variable (user's login shell)
    # This is more reliable than checking version variables when script runs in bash
    if [ -n "$SHELL" ]; then
        case "$SHELL" in
            */bash)
                echo "bash"
                return
                ;;
            */zsh)
                echo "zsh"
                return
                ;;
            */fish)
                echo "fish"
                return
                ;;
        esac
    fi

    # Fallback: try to detect the parent process's shell
    # Get the parent process (the shell that invoked this script)
    if command -v ps >/dev/null 2>&1; then
        local parent_cmd
        parent_cmd=$(ps -o comm= -p $PPID 2>/dev/null || ps -o args= -p $PPID 2>/dev/null | awk '{print $1}')
        case "$parent_cmd" in
            *bash)
                echo "bash"
                return
                ;;
            *zsh)
                echo "zsh"
                return
                ;;
            *fish)
                echo "fish"
                return
                ;;
        esac
    fi

    # Last resort: check version variables (only works if script is sourced)
    if [ -n "$BASH_VERSION" ]; then
        echo "bash"
    elif [ -n "$ZSH_VERSION" ]; then
        echo "zsh"
    elif [ -n "$FISH_VERSION" ]; then
        echo "fish"
    else
        echo "unknown"
    fi
}

install_bash_completion() {
    print_info "Installing Bash completion..."

    local completion_file="$SCRIPT_DIR/git-polyp.bash"
    local install_dirs=(
        "/usr/local/etc/bash_completion.d"
        "/etc/bash_completion.d"
        "$HOME/.local/share/bash-completion/completions"
    )

    # Try to install to a system directory
    for dir in "${install_dirs[@]}"; do
        if [ -d "$dir" ] && [ -w "$dir" ]; then
            cp "$completion_file" "$dir/git-polyp"
            print_success "Installed to $dir/git-polyp"
            print_info "Restart your shell or run: source $dir/git-polyp"
            return 0
        fi
    done

    # Fallback: suggest manual sourcing
    print_warning "Could not find writable bash completion directory"
    print_info "Add this line to your ~/.bashrc or ~/.bash_profile:"
    echo ""
    echo "    source $completion_file"
    echo ""
}

install_zsh_completion() {
    print_info "Installing Zsh completion..."

    local completion_file="$SCRIPT_DIR/git-polyp.zsh"
    local install_dirs=(
        "/usr/local/share/zsh/site-functions"
        "$HOME/.zsh/completions"
    )

    # Try to install to a directory in fpath
    for dir in "${install_dirs[@]}"; do
        # Special handling for user completions directory
        if [ "$dir" = "$HOME/.zsh/completions" ]; then
            # Create user completions directory if it doesn't exist
            mkdir -p "$dir"
            cp "$completion_file" "$dir/_git-polyp"
            print_success "Installed to $dir/_git-polyp"

            # Check if fpath is already configured in .zshrc
            local zshrc="$HOME/.zshrc"
            if [ -f "$zshrc" ]; then
                if ! grep -q "fpath=(.*\.zsh/completions" "$zshrc" 2>/dev/null; then
                    # Add fpath configuration to .zshrc
                    # Try to add it before 'source $ZSH/oh-my-zsh.sh' if using oh-my-zsh
                    if grep -q "source.*oh-my-zsh\.sh" "$zshrc" 2>/dev/null; then
                        # Insert before oh-my-zsh is sourced
                        local temp_file=$(mktemp)
                        awk '/source.*oh-my-zsh\.sh/ && !inserted {
                            print "# Custom completions (must be before oh-my-zsh is sourced)"
                            print "fpath=(~/.zsh/completions $fpath)"
                            print ""
                            inserted=1
                        }
                        {print}' "$zshrc" > "$temp_file"
                        mv "$temp_file" "$zshrc"
                        print_success "Added fpath configuration to ~/.zshrc (before oh-my-zsh)"
                    else
                        # Add at the beginning of the file
                        local temp_file=$(mktemp)
                        echo "# Custom completions" > "$temp_file"
                        echo "fpath=(~/.zsh/completions \$fpath)" >> "$temp_file"
                        echo "autoload -Uz compinit && compinit" >> "$temp_file"
                        echo "" >> "$temp_file"
                        cat "$zshrc" >> "$temp_file"
                        mv "$temp_file" "$zshrc"
                        print_success "Added fpath configuration to ~/.zshrc"
                    fi
                    print_info "Restart your shell to activate completions"
                else
                    print_info "fpath already configured in ~/.zshrc"
                    print_info "Restart your shell or run: exec zsh"
                fi
            else
                print_warning "~/.zshrc not found"
                print_info "Add this to your ~/.zshrc:"
                echo ""
                echo "    fpath=(~/.zsh/completions \$fpath)"
                echo "    autoload -Uz compinit && compinit"
                echo ""
            fi
            return 0
        elif [ -d "$dir" ] && [ -w "$dir" ]; then
            # System-wide directory that's already in fpath
            cp "$completion_file" "$dir/_git-polyp"
            print_success "Installed to $dir/_git-polyp"
            print_info "Restart your shell or run: compinit"
            return 0
        fi
    done

    # Fallback
    print_warning "Could not find writable zsh completion directory"
    print_info "Manually copy the completion file:"
    echo ""
    echo "    mkdir -p ~/.zsh/completions"
    echo "    cp $completion_file ~/.zsh/completions/_git-polyp"
    echo ""
    print_info "Then add to your ~/.zshrc:"
    echo ""
    echo "    fpath=(~/.zsh/completions \$fpath)"
    echo "    autoload -Uz compinit && compinit"
    echo ""
}

install_fish_completion() {
    print_info "Installing Fish completion..."

    local completion_file="$SCRIPT_DIR/git-polyp.fish"
    local install_dir="$HOME/.config/fish/completions"

    # Create directory if it doesn't exist
    mkdir -p "$install_dir"

    cp "$completion_file" "$install_dir/git-polyp.fish"
    print_success "Installed to $install_dir/git-polyp.fish"
    print_info "Completion will be available in new Fish shell sessions"
}

show_usage() {
    cat << EOF
Usage: $0 [SHELL]

Install git-polyp shell completions.

Arguments:
  SHELL    Shell type: bash, zsh, or fish (auto-detected if not specified)

Options:
  -h, --help    Show this help message

Examples:
  $0              # Auto-detect shell and install
  $0 bash         # Install for Bash
  $0 zsh          # Install for Zsh
  $0 fish         # Install for Fish

EOF
}

main() {
    local target_shell=""

    # Parse arguments
    case "${1:-}" in
        -h|--help)
            show_usage
            exit 0
            ;;
        bash|zsh|fish)
            target_shell="$1"
            ;;
        "")
            target_shell=$(detect_shell)
            ;;
        *)
            print_error "Unknown shell: $1"
            show_usage
            exit 1
            ;;
    esac

    if [ "$target_shell" = "unknown" ]; then
        print_error "Could not detect shell. Please specify: bash, zsh, or fish"
        show_usage
        exit 1
    fi

    print_info "Installing completion for: $target_shell"
    echo ""

    case "$target_shell" in
        bash)
            install_bash_completion
            ;;
        zsh)
            install_zsh_completion
            ;;
        fish)
            install_fish_completion
            ;;
    esac

    echo ""
    print_success "Installation complete!"
}

main "$@"
